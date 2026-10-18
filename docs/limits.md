# Limits

The limits are compile-time constants in `src/utils/constants.rs`. Changing them means changing the code and deploying again. There are no environment variables for them.

## Tiers

| Tier | Uploads per hour | Uploads per day | Max file size | Default retention |
|------|------------------|-----------------|---------------|-------------------|
| Anonymous (no key) | 10 | 25 | 25 MB | 3 days |
| Standard (API key) | 100 | 500 | 100 MB | 7 days |
| Premium (API key) | 1000 | 5000 | 500 MB | 30 days |

A client is anonymous when the request has no `X-API-Key` header. API keys are created with the [admin API](admin.md#create-an-api-key).

## Rate limits

Only uploads count. Downloads, info and delete requests are not limited.

- Who is counted: an API key is counted by the key. An anonymous client is counted by a hash of its IP address (`CF-Connecting-IP`). Requests without that header share one anonymous counter.
- Windows: each client has an hourly and a daily counter. A counter starts when the first upload arrives after the previous window ended, and resets one hour (or one day) later. The windows are not aligned to the clock.
- Both counters are checked. A rejected request does not add to the counters.
- The check runs before the file is validated. An upload that later fails with `400` or `413` still uses one upload from each counter.

When a limit is hit, the upload returns `429` with a `Retry-After` header and a JSON body. `Retry-After` is the number of whole seconds until the upload can succeed again (at least 1). `message` names the full window, `Hourly rate limit exceeded` or `Daily rate limit exceeded`. If both windows are full, `Retry-After` counts to the later of the two resets and `message` names that window:

```json
{ "error": "Rate limit exceeded", "message": "Hourly rate limit exceeded" }
```

An admin can clear the counters of one client: see [admin.md](admin.md#reset-a-rate-limit).

## File size

The size is checked after the whole body has been read. A file over the limit returns `413`:

```json
{ "error": "File too large", "message": "Maximum file size is 25 MB", "max_size": 26214400 }
```

Cloudflare also limits the request body size by plan. See [Cloudflare limits and pricing](deployment.md#cloudflare-limits-and-pricing).

## Retention

Each upload gets an expiry time (`expires_at`) when it is stored.

- Without `Max-Age-Days`, the file gets the tier default from the table above.
- With `Max-Age-Days: N`, the file gets `N` days. `N` can only shorten the retention:
  - `N` above the tier default: `400`, `Invalid retention period`, with the maximum for the tier in the message.
  - `N` of 0 or negative: `400`, `Retention period must be at least 1 day`.
  - A value that is not an integer is ignored, and the default applies.

An expired file is removed by the daily cleanup job, not at the exact expiry time. See [architecture.md](architecture.md#cleanup-job).

## Max-Downloads

`Max-Downloads: N` deletes the file after `N` downloads. There is no tier limit on `N`. A value that is not a positive integer is ignored, and the file has no download limit.

`Max-Downloads` and `Max-Age-Days` work together: the file is removed by whichever comes first, the download count reaching `N` or the expiry time passing.

Details of the download count are in [api.md](api.md#download).
