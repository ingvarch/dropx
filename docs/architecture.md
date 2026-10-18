# Architecture

## Components

| Component | Role |
|-----------|------|
| Cloudflare Worker (Rust, `worker` crate) | Routes requests and runs the logic |
| R2 bucket (binding `TRANSFER_BUCKET`) | Stores file contents |
| D1 database (binding `DB`) | Stores file metadata and API keys |
| Durable Object `RateLimiter` (binding `RATE_LIMITER`) | Counts uploads per client |
| Cron trigger | Starts the cleanup job once a day |

Code layout is in [development.md](development.md#project-structure). The router is in `src/lib.rs`. Handlers call services (`database`, `storage`, `rate_limiter`), and services talk to the bindings.

## Upload flow

1. If `X-API-Key` is present, look the key up in D1. An unknown or revoked key returns `401`. Otherwise the client is identified by a hash of `CF-Connecting-IP`.
2. Ask the client's `RateLimiter` Durable Object to count the upload. Over the limit returns `429`.
3. Read the whole body into memory and check the size against the tier. Too large returns `413`.
4. Generate the file ID and the delete token ([IDs](#ids-and-tokens)).
5. Detect the MIME type from the first bytes, then from the file name.
6. Write the body to R2.
7. Check `Max-Age-Days` and `Max-Downloads`, compute `expires_at`.
8. Insert the metadata row into D1 and return the JSON response.

The retention check (step 7) runs after the R2 write. A request rejected there leaves an object in R2 without a database row. The cleanup job does not find such objects.

## Download flow

`GET /:id`, `GET /get/:id` and `GET /inline/:id` share one code path and differ only in `Content-Disposition`:

1. Read the metadata row from D1. No row returns `404`.
2. Read the object from R2. No object returns `404`.
3. Add one to `download_count` in D1.
4. If `max_downloads` is set and the new count has reached it, delete the object from R2 and the row from D1.
5. Return the bytes with the headers listed in [api.md](api.md#download).

The expiry time is not checked on download.

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
| `uploaded_by` | TEXT | API key, or IP hash for anonymous uploads |
| `delete_token` | TEXT | Added in `0002` |
| `max_downloads` | INTEGER | Added in `0003`, null means no limit |

Indexes: `expires_at`, `uploaded_by`, `created_at`, `delete_token`, and `max_downloads` (only rows where it is set).

### api_keys

| Column | Type | Notes |
|--------|------|-------|
| `key` | TEXT, primary key | `tfr_` followed by 32 hex characters |
| `user_email` | TEXT | |
| `created_at` | INTEGER | Unix seconds |
| `last_used_at` | INTEGER | Updated on each upload with the key |
| `is_active` | INTEGER | 1 active, 0 revoked |
| `tier` | TEXT | `standard` or `premium` |

Indexes: `is_active`, `user_email`.

## Rate limiter

One Durable Object per client, named by the client identity (the API key, or the IP hash). It stores the hourly and daily counters and the time each window resets. The Worker calls it with the limits for the client's tier. Windows and `429` behavior are in [limits.md](limits.md#rate-limits).

## Cleanup job

The cron trigger runs `cleanup::run_cleanup` daily at 02:00 UTC:

1. Select the IDs of rows with `expires_at` in the past, then delete those rows from D1.
2. Delete each object from R2. A failure is logged and not retried: the row is already gone, so the object stays in R2.

Expired files stay downloadable until this job runs.

## IDs and tokens

- File ID: the first 8 bytes of a random UUID v4, base58-encoded.
- Delete token: the 16 bytes of a random UUID v4, base58-encoded.
- API key: `tfr_` plus the 16 bytes of a random UUID v4 in hex.

## Security

What the code does:

- IP addresses are hashed with SHA-256 (first 16 bytes, hex) before they are used as the rate limit key or stored in `uploaded_by`. The hash is not salted.
- Admin routes need `X-Admin-Token` equal to the `ADMIN_TOKEN` secret.
- Deleting needs the delete token that only the uploader receives.
- Anyone who knows a file ID can download it. There is no per-file access control.
- Files are stored as uploaded. The service does not encrypt them. The landing page shows how to encrypt with GPG or OpenSSL before upload.
- API keys and delete tokens are stored as plain text in D1.
- Files are removed on expiry, on delete, or when `Max-Downloads` is reached.
