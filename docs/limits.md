# Limits

The limits are compile-time constants in `src/utils/constants.rs`. Changing them means changing the code and deploying again. There are no environment variables for them.

## Tiers

| Tier | Uploads per hour | Uploads per day | Max file size | Default retention |
|------|------------------|-----------------|---------------|-------------------|
| Anonymous (no key) | 10 | 25 | 25 MB | 3 days |
| Standard (API key) | 100 | 500 | 100 MB | 7 days |
| Premium (API key) | 1000 | 5000 | 100 MB | 30 days |

A client is anonymous when the request has no `X-API-Key` header. API keys are created with the [admin API](admin.md#create-an-api-key).

## Rate limits

Only uploads count. Downloads, info and delete requests are not limited.

- Who is counted: an API key is counted by its SHA-256 hash. An anonymous client is counted by a keyed hash (HMAC) of its network, taken from `CF-Connecting-IP`: the IPv4 address, or the /64 prefix for IPv6 (an IPv4-mapped IPv6 address counts as its IPv4 address). An IPv6 client usually owns a whole /64 and can switch addresses inside it, so counting each address would give it fresh limits every time. Requests without that header share one anonymous counter.
- Windows: each client has an hourly and a daily counter. A counter starts when the first upload arrives after the previous window ended, and resets one hour (or one day) later. The windows are not aligned to the clock.
- Both counters are checked. A rejected request does not add to the counters.
- The headers and the `Content-Length` size are checked before the rate limit. An upload rejected with `400` or with `413` from its `Content-Length` does not use the counters. An upload without `Content-Length` that turns out too large while the body is read (`413` or `411`) still uses one upload from each counter.

When a limit is hit, the upload returns `429` with a `Retry-After` header and a JSON body. `Retry-After` is the number of whole seconds until the upload can succeed again (at least 1). `message` names the full window, `Hourly rate limit exceeded` or `Daily rate limit exceeded`. If both windows are full, `Retry-After` counts to the later of the two resets and `message` names that window:

```json
{ "error": "Rate limit exceeded", "message": "Hourly rate limit exceeded" }
```

An admin can clear the counters of one client: see [admin.md](admin.md#reset-a-rate-limit).

## File size

With a `Content-Length` header, the size is checked before the rate limit and before the body is read. A file over the tier limit returns `413`:

```json
{ "error": "File too large", "message": "Maximum file size is 25 MB", "max_size": 26214400 }
```

Within the limit, the body goes straight from the request to R2 without being held in the Worker's memory. The stored size is the one R2 reports.

Without `Content-Length` (a chunked upload, for example `cat file | curl -T - ...`), the Worker reads the body into memory, up to 25 MB. It stops reading as soon as the limit is passed:

- Over the tier limit when the tier limit is 25 MB or smaller: `413`, as above.
- Over 25 MB with a tier that allows more: `411`, `Length required`, with a message to send `Content-Length`:

```json
{ "error": "Length required", "message": "Send a Content-Length header for files over 25 MB" }
```

`curl --upload-file ./file` sends `Content-Length` by itself, so only streamed input is affected.

Cloudflare also limits the request body size by the zone plan: 100 MB on Free and Pro, 200 MB on Business, more on Enterprise. Larger requests are rejected by Cloudflare with its own HTML `413` page before they reach the Worker. dropx.click is on the Pro plan; measured on production, a body of 104,857,600 bytes (100 MB as the tiers count it) reaches the Worker and one more byte does not. The standard and premium limits are set to exactly that size, so a file within a tier limit always gets past Cloudflare. A body over it gets Cloudflare's HTML `413` on every tier, not the JSON one. A larger premium limit needs the Business or Enterprise plan. See [Cloudflare Workers limits](https://developers.cloudflare.com/workers/platform/limits/) and [deployment.md](deployment.md#cloudflare-limits-and-pricing).

## Retention

Each upload gets an expiry time (`expires_at`) when it is stored.

- Without `Max-Age-Days`, the file gets the tier default from the table above.
- With `Max-Age-Days: N`, the file gets `N` days. `N` can only shorten the retention:
  - `N` above the tier default: `400`, `Invalid retention period`, with the maximum for the tier in the message.
  - `N` of 0 or negative: `400`, `Retention period must be at least 1 day`.
  - A value that is not an integer is ignored, and the default applies.

After `expires_at`, the download routes and `/:id/info` return `404`. Delete with the token still works until the hourly cleanup job removes the file. See [api.md](api.md#delete) and [architecture.md](architecture.md#cleanup-job).

## Max-Downloads

`Max-Downloads: N` stops serving the file after `N` downloads. The cleanup job deletes it within an hour. There is no tier limit on `N`. A value that is not a positive integer is ignored, and the file has no download limit.

`Max-Downloads` and `Max-Age-Days` work together: the file stops being served at whichever comes first, the download count reaching `N` or the expiry time passing.

Details of the download count are in [api.md](api.md#download).
