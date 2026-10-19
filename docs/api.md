# API Reference

Public routes of DropX. Admin routes are in [admin.md](admin.md). Limits and retention rules are in [limits.md](limits.md).

Examples use `https://dropx.click`. Timestamps (`created_at`, `expires_at`, `X-Expires-At`) are Unix time in seconds.

## Routes

| Method | Path | Purpose |
|--------|------|---------|
| GET, HEAD | `/` | Landing page (browsers) or a one-line text (other clients) |
| POST | `/` | Upload, file name is `file` |
| PUT | `/:filename` | Upload with a file name |
| GET | `/:id` | Download, shown in the browser when the type allows it |
| GET | `/get/:id` | Download, always as an attachment |
| GET | `/inline/:id` | Download, always shown in the browser |
| GET | `/:id/info` | File metadata as JSON |
| DELETE | `/:id/:token` | Delete a file with its delete token |

`PUT /` is not a route. `curl --upload-file ./file.txt https://dropx.click/` works because curl appends the file name to a URL that ends with `/`. A plain `PUT /` request returns `405`.

## Upload

`POST /` or `PUT /:filename`. The request body is the file. `:filename` is a single path segment; without it the name is `file`.

```bash
curl --upload-file ./big.zip -H "X-API-Key: tfr_xxxxx" https://dropx.click/big.zip
```

### Request headers

| Header | Required | Meaning |
|--------|----------|---------|
| `X-API-Key` | no | API key. Raises the limits to the key's tier. An unknown or revoked key gives `401`. |
| `Max-Downloads` | no | Stop serving the file after this many downloads. The cleanup job deletes it within an hour. Must be a positive integer; any other value is ignored. |
| `Max-Age-Days` | no | Keep the file this many days instead of the tier default. Rules in [limits.md](limits.md#retention). |

The `Content-Type` of the request is ignored. The type is detected from the first bytes of the file (PDF, JPEG, PNG, GIF, ZIP, RAR, gzip, XML, JSON) and then from the file name extension. Unknown files get `application/octet-stream`.

### Response

Status `200`, `Content-Type: application/json`:

```json
{
  "url": "https://dropx.click/<id>",
  "id": "<id>",
  "filename": "file.txt",
  "size": 1234,
  "mime_type": "text/plain",
  "expires_in": "3d",
  "expires_at": 1760000000,
  "delete_url": "https://dropx.click/<id>/<delete_token>"
}
```

The delete URL is also sent in the `X-Url-Delete` response header. The upload response is the only place the delete token is shown; `/:id/info` does not return it. The URLs use the `Host` of the request: `https` for public hosts, `http` and the port for `localhost` and `127.0.0.1`.

### Errors

| Status | Cause | Body |
|--------|-------|------|
| 400 | `Max-Age-Days` is 0 or negative, or above the tier maximum | JSON, `error` is `Invalid retention period` |
| 401 | `X-API-Key` is unknown or revoked | JSON, `error` is `Invalid API key` |
| 413 | File is larger than the tier allows | JSON, `error` is `File too large`, with `max_size` in bytes |
| 429 | Hourly or daily upload limit reached | JSON, `error` is `Rate limit exceeded`, `message` names the window (`Hourly rate limit exceeded` or `Daily rate limit exceeded`). The `Retry-After` header holds the seconds until the window resets |

## Download

`GET /:id`, `GET /get/:id`, `GET /inline/:id`. The three routes differ only in `Content-Disposition`:

- `/:id` sends `inline` for images (JPEG, PNG, GIF, WebP, SVG), PDF, plain text, HTML, CSS, JavaScript, MP4, WebM, MP3, OGG and WAV, and `attachment` for everything else.
- `/get/:id` always sends `attachment`.
- `/inline/:id` always sends `inline`.

### Response headers

| Header | Value |
|--------|-------|
| `Content-Type` | The detected type |
| `Content-Disposition` | `inline` or `attachment`, with the original file name |
| `Content-Length` | File size in bytes |
| `X-Download-Count` | Downloads before this one (the first download shows `0`) |
| `X-Expires-At` | Expiry time |

Every download request adds one to the counter. When the counter reaches `Max-Downloads`, the file is no longer served: the download that reaches the limit succeeds, and the next request returns `404`. The cleanup job deletes the file within an hour.

A file is available while the current time is before `expires_at`. After that, the download routes and `/:id/info` return `404`, `File not found`, as for an unknown ID. [Delete](#delete) still works until the hourly cleanup job removes the file. See [architecture.md](architecture.md#cleanup-job).

### Errors

| Status | Cause | `error` |
|--------|-------|---------|
| 404 | No such file | `File not found` |
| 404 | The database has the file but storage does not | `File not found in storage` |

## File info

`GET /:id/info` returns JSON and does not count as a download:

```json
{
  "id": "<id>",
  "filename": "file.txt",
  "size": 1234,
  "mime_type": "text/plain",
  "created_at": 1759740800,
  "expires_at": 1760000000,
  "download_count": 0
}
```

```bash
curl https://dropx.click/<id>/info
```

Unknown ID: `404`, `error` is `File not found`.

## Delete

`DELETE /:id/:token` needs no API key. The token is the last part of `delete_url`.

Delete also works on a file past `expires_at` that the cleanup job has not removed yet.

| Status | Body |
|--------|------|
| 200 | `{"success": true, "message": "File deleted successfully"}` |
| 404 | `error` is `NotFound` (no such file) |
| 404 | `error` is `Unauthorized` (wrong token) |
| 500 | Plain text (see [Error format](#error-format)). The object could not be deleted from storage; the file is kept, retry the request |

## Landing page

`GET /` returns the HTML page when the `Accept` header contains `text/html`. Other clients get the text `DropX - File Transfer Service`. The response has `Vary: Accept`. `HEAD /` returns the same status and headers without a body, so uptime checks such as the README badge work.

## Error format

Errors from the handlers are JSON:

```json
{ "error": "File not found", "message": "The requested file does not exist or has expired" }
```

Exceptions: a path that exists for another method returns plain-text `405 Method Not Allowed`; unknown routes return plain-text `404 Not Found`; internal failures (database, storage, rate limiter, missing `ADMIN_TOKEN`, missing `IP_HASH_KEY` for anonymous uploads) return plain-text `500`.
