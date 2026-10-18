# Deployment

Steps to run DropX on your own Cloudflare account. For local work, see [development.md](development.md).

The prerequisites are the same as for development. Log in first:

```bash
bunx wrangler login
```

## Resources

The names must match `wrangler.toml`.

### R2 bucket

The binding `TRANSFER_BUCKET` points at the bucket `assets-dropx-click`:

```bash
bunx wrangler r2 bucket create assets-dropx-click
```

To use another name, change `bucket_name` in `wrangler.toml`.

### D1 database

The binding `DB` points at the database `dropx-db`:

```bash
bunx wrangler d1 create dropx-db
```

Copy the `database_id` from the output into the `[[d1_databases]]` block of `wrangler.toml`.

### Migrations

The `migrations/` folder holds `0001_initial.sql`, `0002_add_delete_token.sql`, `0003_add_max_downloads.sql` and `0004_hash_stored_secrets.sql`. For a new database, apply all of them:

```bash
bunx wrangler d1 migrations apply dropx-db --remote
```

wrangler records applied files in the `d1_migrations` table and skips them next time.

A database set up earlier with `wrangler d1 execute --file` has no history in `d1_migrations`. On such a database apply only the new migration files, with `bunx wrangler d1 execute dropx-db --remote --file=migrations/<file>.sql`, then deploy with `bunx wrangler deploy`. `bun run deploy` would try to apply every migration again and stop at `0002`.

Migration `0004` replaces the stored delete tokens and API keys with random values and deactivates all API keys, because SQLite cannot hash them. After it runs, existing delete links stop working, existing API keys return `401` and must be created again, and `uploaded_by` of existing files no longer matches any client. Files stay downloadable until they expire. The migration does not change the schema, so the previous Worker version still runs on the migrated database.

Apply `0004` right before you deploy the version that hashes secrets; `bun run deploy` does both, in that order. The migration keeps values that are already 64-character SHA-256 hex: delete tokens, API keys and the `uploaded_by` of keyed uploads written by the new version survive either order. Anonymous `uploaded_by` values look the same in both versions, so all of them are replaced. Between the two steps, or after a `wrangler rollback`, the previous version stores plain delete tokens and API keys: those delete links fail and those keys return `401` on the new version, and the plain values stay in D1 until the files expire. To clear them, run the `0004` SQL again with `bunx wrangler d1 execute dropx-db --remote --file=migrations/0004_hash_stored_secrets.sql`: it replaces every value that is not 64 characters long.

### Durable Object

`wrangler.toml` declares the `RATE_LIMITER` binding and a migration (`tag = "v1"`) that creates the SQLite-backed `RateLimiter` class. Nothing to run: `wrangler deploy` applies it.

### Cron trigger

The cleanup job runs daily at 02:00 UTC (`crons = ["0 2 * * *"]` in `wrangler.toml`). `wrangler deploy` registers it. What the job does is in [architecture.md](architecture.md#cleanup-job).

## Secrets

```bash
bunx wrangler secret put ADMIN_TOKEN
```

Enter a long random value when prompted. It is used for the [admin API](admin.md).

```bash
openssl rand -hex 32 | bunx wrangler secret put IP_HASH_KEY
```

`IP_HASH_KEY` is the key for the IP hashes. Set it before you deploy this version: without it anonymous uploads fail with `500`. Keep the value. If it changes, every client gets a new rate limit identity.

```bash
bunx wrangler secret list
bunx wrangler secret delete ADMIN_TOKEN
```

## Deploy

```bash
bun run deploy
```

It builds the Worker, applies the migrations the remote database does not have yet, and runs `wrangler deploy`. The migrations are applied without a confirmation prompt, and a failed build or migration stops the deploy. Building first keeps the time between the migration and the new version short. To see the pending migrations first:

```bash
bunx wrangler d1 migrations list dropx-db --remote
```

`wrangler.toml` has `workers_dev = true`, so the Worker is served on a `workers.dev` address. A custom domain is not configured in `wrangler.toml`.

## Logs

```bash
bunx wrangler tail
```

The Worker writes log lines with `console_log!`, mostly from the cleanup job and from failed deletes. The `[observability.logs]` block in `wrangler.toml` has `enabled = false`, so logs are not stored in the Cloudflare dashboard.

## Cloudflare limits and pricing

Platform limits and prices depend on your plan and change over time. Check the official pages:

- Workers: [limits](https://developers.cloudflare.com/workers/platform/limits/), [pricing](https://developers.cloudflare.com/workers/platform/pricing/)
- R2: [limits](https://developers.cloudflare.com/r2/platform/limits/), [pricing](https://developers.cloudflare.com/r2/pricing/)
- D1: [limits](https://developers.cloudflare.com/d1/platform/limits/), [pricing](https://developers.cloudflare.com/d1/platform/pricing/)
- Durable Objects: [limits](https://developers.cloudflare.com/durable-objects/platform/limits/), [pricing](https://developers.cloudflare.com/durable-objects/platform/pricing/)

The upload limits of the service itself are in [limits.md](limits.md).
