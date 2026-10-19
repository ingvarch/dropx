# Architecture

## Components

| Component | Role |
|-----------|------|
| Cloudflare Worker (Rust, `worker` crate) | Routes requests and runs the logic |
| R2 bucket (binding `TRANSFER_BUCKET`) | Stores file contents |
| D1 database (binding `DB`) | Stores file metadata and API keys |
| Durable Object `RateLimiter` (binding `RATE_LIMITER`) | Counts uploads per client |
| Cron trigger | Starts the cleanup job every hour |

Code layout is in [development.md](development.md#project-structure). The router is in `src/lib.rs`. Handlers call services (`database`, `storage`, `rate_limiter`), and services talk to the bindings.

## Upload flow

1. If `X-API-Key` is present, hash it with SHA-256 and look the hash up in D1. An unknown or revoked key returns `401`. Otherwise the client is identified by an HMAC of the IPv4 address in `CF-Connecting-IP`, or of its IPv6 /64 prefix.
2. Check `Max-Age-Days` against the tier default and parse `Max-Downloads`. An invalid retention returns `400`.
3. Check `Content-Length`. A valid value over the tier limit returns `413`. A missing or invalid value means the body is buffered later, with a limit of 25 MB or the tier limit if that is smaller.
4. Ask the client's `RateLimiter` Durable Object to count the upload. Over the limit returns `429`. A rate limiter failure returns `500`.
5. Generate the file ID and the delete token ([IDs](#ids-and-tokens)).
6. Write the body to R2. With a valid `Content-Length`, the request stream goes to R2 without entering Worker memory. Without it, the Worker reads the body chunk by chunk and stops when the limit is passed: `413` over the tier limit, `411` over 25 MB when the tier allows more. R2 reports the stored size.
7. Read the first bytes back from R2 and detect the MIME type from them, then from the file name. An empty file skips the read. If the read fails, delete the object just written and return the error (`500`).
8. Compute `expires_at`, insert the metadata row into D1 and return the download URL as plain text, or the JSON response when `Accept` contains `application/json`. The `400`, `401`, `411`, `413` and `429` answers follow the same rule; a `500` does not. If the insert fails, delete the object it just wrote to R2 and return the insert error (`500`). A failed delete is logged and does not replace that error.

A request rejected in steps 2 to 4, or in step 6 by the size check, stores no file: nothing is written to R2 and no row is added to `files`.

## Download flow

`GET /:id`, `GET /get/:id` and `GET /inline/:id` share one code path. They differ in `Content-Disposition`, and `/inline/:id` changes `Content-Type` for HTML, SVG and XML ([api.md](api.md#download)). The decision is in `content_headers` in `src/utils/content_disposition.rs`. The shared code path:

1. Read the metadata row from D1. No row, a row with `expires_at` in the past, or a row whose `download_count` has reached `max_downloads`, returns `404`.
2. Open the object in R2. No object returns `404`.
3. Add one to `download_count` in D1.
4. Stream the object body from R2 into the response with the headers listed in [api.md](api.md#download). The Worker does not read the file into memory, so the size of a file is not limited by the Worker memory.

The download never deletes a file. When the new count reaches `max_downloads`, step 1 returns `404` for the next request, and the cleanup job removes the file.

## Delete flow

`DELETE /:id/:token`:

1. Read the stored token hash by ID. No row returns `404` `NotFound`.
2. Compare it with the hash of the presented token in constant time. A mismatch returns `404` `Unauthorized`.
3. Delete the object from R2, then the row from D1. If the object delete fails, the row stays and the request returns `500`. The file can be deleted again, and the cleanup job removes it after it expires.
4. Return `200`.

Every removal (delete and the cleanup job) goes through `remove_file` in `src/services/file_removal.rs`.

## Storage path

R2 keys are sharded by the file ID, lowercased:

```
{first 2 characters}/{characters 3-4}/{file ID}
```

For example, the ID `aBcD1234xyz` is stored as `ab/cd/aBcD1234xyz`. The ID keeps its original case in the last part.

## Database

Schema from `migrations/`.

### files

| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT, primary key | File ID |
| `original_filename` | TEXT | |
| `size` | INTEGER | Bytes |
| `mime_type` | TEXT | |
| `created_at` | INTEGER | Unix seconds |
| `expires_at` | INTEGER | Unix seconds |
| `download_count` | INTEGER | Default 0 |
| `uploaded_by` | TEXT | SHA-256 of the API key, or the IP HMAC for anonymous uploads |
| `delete_token` | TEXT | SHA-256 of the delete token, 64 hex characters. Added in `0002` |
| `max_downloads` | INTEGER | Added in `0003`, null means no limit |

Indexes: `expires_at`, `uploaded_by`, `created_at`, `delete_token`, and `max_downloads` (only rows where it is set).

### api_keys

| Column | Type | Notes |
|--------|------|-------|
| `key` | TEXT, primary key | SHA-256 of the API key, 64 hex characters |
| `user_email` | TEXT | |
| `created_at` | INTEGER | Unix seconds |
| `last_used_at` | INTEGER | Updated on each upload with the key |
| `is_active` | INTEGER | 1 active, 0 revoked |
| `tier` | TEXT | `standard` or `premium`. Any other value is an error: the key lookup fails and the upload returns `500` |

Indexes: `is_active`, `user_email`.

## Rate limiter

One Durable Object per client, named by the client identity (the SHA-256 of the API key, or the IP HMAC). It stores the hourly and daily counters and the time each window resets. The Worker calls it with the limits for the client's tier (or the anonymous limits). A check without numeric `hourly` and `daily` values returns `400`, and the upload fails with `500`. Windows and `429` behavior are in [limits.md](limits.md#rate-limits).

## Cleanup job

The cron trigger runs `cleanup::run_cleanup` every hour, at minute 0:

1. Select the IDs of rows with `expires_at` at or before now, and rows whose `download_count` has reached `max_downloads`: the same rows the download routes no longer serve.
2. For each file, delete the object from R2, then the row from D1. A failure is logged.
3. A row whose R2 delete failed stays, and the next run retries it.

Expired files are already unavailable before this job runs: the download and info routes skip rows with `expires_at` in the past, and only delete still finds them. The job frees their storage. A file that used up its `Max-Downloads` stays in storage until the next run, at most one hour.

## IDs and tokens

- File ID: the first 8 bytes of a random UUID v4, base58-encoded.
- Delete token: the 16 bytes of a random UUID v4, base58-encoded.
- API key: `tfr_` plus the 16 bytes of a random UUID v4 in hex.

## Security

What the code does:

- API keys and delete tokens are random UUID v4 values (122 random bits). D1 stores only their SHA-256 (`api_keys.key`, `files.delete_token`). The create and upload responses show the plain value once, and it cannot be recovered. A delete or an upload with a key hashes the presented value and compares it in SQL.
- IPv4 addresses and IPv6 /64 prefixes (`2001:db8:1:2::/64`) are hashed with HMAC-SHA-256 keyed with the `IP_HASH_KEY` secret (first 16 bytes, hex, 32 characters) before they are used as the rate limit key or stored in `uploaded_by`. Without the key the hash cannot be reversed by trying all IPv4 addresses. A missing `IP_HASH_KEY` makes anonymous uploads fail with `500`. Changing the key gives every client a new identity, so rate limit counters start over.
- Admin routes need `X-Admin-Token` equal to the `ADMIN_TOKEN` secret. Both values are hashed with SHA-256 and the digests are compared in constant time, so response time does not reveal how much of a guess is right or how long the secret is.
- HTML, SVG and XML files are never served as pages: `/:id` sends them as attachments, `/inline/:id` sends them as `text/plain`, and every download sends `X-Content-Type-Options: nosniff`.
- Deleting needs the delete token that only the uploader receives.
- Anyone who knows a file ID can download it. There is no per-file access control.
- Files are stored as uploaded. The service does not encrypt them. The landing page shows how to encrypt with GPG or OpenSSL before upload.
- Files are removed on expiry, on delete, or after `Max-Downloads` is reached (the cleanup job deletes it within an hour).
