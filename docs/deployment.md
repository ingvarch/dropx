# Deployment

Steps to run DropX on your own Cloudflare account. For local work, see [development.md](development.md).

The prerequisites are the same as for development. Log in first:

```bash
wrangler login
```

## Resources

The names must match `wrangler.toml`.

### R2 bucket

The binding `TRANSFER_BUCKET` points at the bucket `assets-dropx-click`:

```bash
wrangler r2 bucket create assets-dropx-click
```

To use another name, change `bucket_name` in `wrangler.toml`.

### D1 database

The binding `DB` points at the database `dropx-db`:

```bash
wrangler d1 create dropx-db
```

Copy the `database_id` from the output into the `[[d1_databases]]` block of `wrangler.toml`.

### Migrations

The `migrations/` folder holds three files, `0001_initial.sql`, `0002_add_delete_token.sql` and `0003_add_max_downloads.sql`. For a new database, apply all of them:

```bash
wrangler d1 migrations apply dropx-db --remote
```

wrangler records applied files in the `d1_migrations` table and skips them next time.

A database set up earlier with `wrangler d1 execute --file` has no history in `d1_migrations`. On such a database apply only the new migration files, with `wrangler d1 execute dropx-db --remote --file=migrations/<file>.sql`.

### Durable Object

`wrangler.toml` declares the `RATE_LIMITER` binding and a migration (`tag = "v1"`) that creates the SQLite-backed `RateLimiter` class. Nothing to run: `wrangler deploy` applies it.

### Cron trigger

The cleanup job runs daily at 02:00 UTC (`crons = ["0 2 * * *"]` in `wrangler.toml`). `wrangler deploy` registers it. What the job does is in [architecture.md](architecture.md#cleanup-job).

## Secrets

```bash
wrangler secret put ADMIN_TOKEN
```

Enter a long random value when prompted. It is used for the [admin API](admin.md).

```bash
wrangler secret list
wrangler secret delete ADMIN_TOKEN
```

## Deploy

```bash
wrangler deploy
```

`wrangler.toml` has `workers_dev = true`, so the Worker is served on a `workers.dev` address. A custom domain is not configured in `wrangler.toml`.

## Logs

```bash
wrangler tail
```

The Worker writes log lines with `console_log!`, mostly from the cleanup job and from failed deletes. The `[observability.logs]` block in `wrangler.toml` has `enabled = false`, so logs are not stored in the Cloudflare dashboard.

## Cloudflare limits and pricing

Platform limits and prices depend on your plan and change over time. Check the official pages:

- Workers: [limits](https://developers.cloudflare.com/workers/platform/limits/), [pricing](https://developers.cloudflare.com/workers/platform/pricing/)
- R2: [limits](https://developers.cloudflare.com/r2/platform/limits/), [pricing](https://developers.cloudflare.com/r2/pricing/)
- D1: [limits](https://developers.cloudflare.com/d1/platform/limits/), [pricing](https://developers.cloudflare.com/d1/platform/pricing/)
- Durable Objects: [limits](https://developers.cloudflare.com/durable-objects/platform/limits/), [pricing](https://developers.cloudflare.com/durable-objects/platform/pricing/)

The upload limits of the service itself are in [limits.md](limits.md).
