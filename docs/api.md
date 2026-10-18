# API Reference

Public routes of DropX. Admin routes are in [admin.md](admin.md). Limits and retention rules are in [limits.md](limits.md).

Examples use `https://dropx.click`. Timestamps (`created_at`, `expires_at`, `X-Expires-At`) are Unix time in seconds.

## Routes

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/` | Landing page (browsers) or a one-line text (other clients) |
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
| `Max-Downloads` | no | Delete the file after this many downloads. Must be a positive integer; any other value is ignored. |
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

The delete URL is also sent in the `X-Url-Delete` response header. The upload response is the only place the delete token is shown; `/:id/info` does not return it. The URLs always use `https` and the request host without the port.

### Errors

| Status | Cause | Body |
|--------|-------|------|
| 400 | `Max-Age-Days` is 0 or negative, or above the tier maximum | JSON, `error` is `Invalid retention period` |
| 401 | `X-API-Key` is unknown or revoked | JSON, `error` is `Invalid API key` |
| 413 | File is larger than the tier allows | JSON, `error` is `File too large`, with `max_size` in bytes |
| 429 | Hourly or daily upload limit reached | Plain text `Rate limit exceeded` |

## Download

`GET /:id`, `GET /get/:id`, `GET /inline/:id`. The three routes differ in `Content-Disposition` and in the order of the download-limit check (see below):

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

Every download request adds one to the counter. When the counter reaches `Max-Downloads`, the file is deleted.

`/get/:id` and `/inline/:id` delete the file before they read it. The download that reaches the limit therefore returns `404` with `File not found in storage`. `/:id` reads the file first, so the last download succeeds.

Expiry is enforced by the daily cleanup job, not on each request. A file stays downloadable until the job removes it. See [architecture.md](architecture.md#cleanup-job).

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

| Status | Body |
|--------|------|
| 200 | `{"success": true, "message": "File deleted successfully"}` |
| 404 | `error` is `NotFound` (no such file) |
| 404 | `error` is `Unauthorized` (wrong token) |

## Landing page

`GET /` returns the HTML page when the `Accept` header contains `text/html`. Other clients get the text `DropX - File Transfer Service`. The response has `Vary: Accept`.

## Error format

Errors from the handlers are JSON:

```json
{ "error": "File not found", "message": "The requested file does not exist or has expired" }
```

Exceptions: the `429` response is plain text; a path that exists for another method returns plain-text `405 Method Not Allowed`; unknown routes return plain-text `404 Not Found`; internal failures (database, storage, missing `ADMIN_TOKEN`) return plain-text `500`.
