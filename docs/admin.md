# Admin API

Admin routes manage API keys, show statistics and clear rate limits. They are not shown on the landing page.

Every admin request needs the header `X-Admin-Token` with the value of the `ADMIN_TOKEN` secret. A missing or wrong token returns `401`:

```json
{ "error": "Unauthorized", "message": "Invalid or missing admin token" }
```

If `ADMIN_TOKEN` is not set on the Worker, admin requests fail with a server error instead of `401`.

## Setting ADMIN_TOKEN

`ADMIN_TOKEN` is the only runtime setting the code reads. Everything else the Worker uses comes from the bindings in `wrangler.toml`.

- Production: `npx wrangler secret put ADMIN_TOKEN` (see [deployment.md](deployment.md#secrets)).
- Local: set `ADMIN_TOKEN` in `.dev.vars` (see [development.md](development.md#local-secrets)).

## Create an API key

`POST /admin/api-keys`

```bash
curl -X POST https://dropx.click/admin/api-keys \
  -H "X-Admin-Token: $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "tier": "standard"}'
```

Both fields are required. `tier` is `standard` or `premium` ([limits](limits.md#tiers)). A body that does not parse returns `400`, and `error` is `Invalid request body`.

Response, status `201`:

```json
{
  "key": "tfr_<32 hex characters>",
  "tier": "standard",
  "created_at": 1759740800,
  "limits": { "hourly": 100, "daily": 500 }
}
```

The client sends the key in the `X-API-Key` header on uploads.

## Revoke an API key

`DELETE /admin/api-keys/:key`

```bash
curl -X DELETE https://dropx.click/admin/api-keys/tfr_xxxxx \
  -H "X-Admin-Token: $ADMIN_TOKEN"
```

Response, status `200`: `{"message": "API key revoked", "key": "tfr_xxxxx"}`. The key is marked inactive, and uploads with it return `401`. Revoking a key that is already revoked also returns `200`. A key that does not exist returns `404` with `error` set to `NotFound`.

## Statistics

`GET /admin/stats`

```bash
curl https://dropx.click/admin/stats -H "X-Admin-Token: $ADMIN_TOKEN"
```

```json
{
  "total_files": 120,
  "total_size_bytes": 53687091,
  "active_files": 118,
  "total_downloads": 430,
  "api_keys_count": 5
}
```

- `total_files`: rows in the `files` table, including expired files the cleanup job has not removed yet.
- `active_files`: files whose expiry time is in the future.
- `api_keys_count`: API keys that are not revoked.

## Reset a rate limit

`POST /admin/rate-limit/reset/:user_id`

```bash
curl -X POST https://dropx.click/admin/rate-limit/reset/tfr_xxxxx \
  -H "X-Admin-Token: $ADMIN_TOKEN"
```

`user_id` is the identity the limiter counts, see [limits.md](limits.md#rate-limits):

- For an API key client: the key itself.
- For an anonymous client: the hash of the IP address (SHA-256 of the IP, first 16 bytes, as 32 hex characters). The hash is stored in the `uploaded_by` column of that client's files, so it can be read from the database.

Response, status `200`: `{"message": "Rate limit reset successfully", "user_id": "..."}`.
