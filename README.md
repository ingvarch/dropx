# DropX - File Transfer Service

A file transfer service built with Rust on Cloudflare Workers, inspired by transfer.sh. Upload a file with one curl command and share the link.

## Features

- Upload with `curl`, no signup
- Files expire automatically
- Optional download limit (`Max-Downloads`) and shorter retention (`Max-Age-Days`)
- Delete link returned with every upload
- API keys with higher limits (standard and premium tiers)
- Per-client rate limiting
- Files in R2, metadata in D1, rate limit counters in Durable Objects

## Usage

```bash
# Upload (the response is JSON with the download URL and the delete URL)
curl --upload-file ./file.txt https://dropx.click/file.txt

# Download
curl https://dropx.click/<id> -o file.txt

# Delete
curl -X DELETE https://dropx.click/<id>/<delete_token>
```

Limits per tier are in [docs/limits.md](docs/limits.md). Public routes and options are in [docs/api.md](docs/api.md), admin routes in [docs/admin.md](docs/admin.md).

## Quick Start

You need Rust, Node.js with `wrangler`, and the setup from [docs/development.md](docs/development.md#prerequisites).

```bash
cp .dev.vars.example .dev.vars
wrangler d1 migrations apply dropx-db --local
wrangler dev
curl --upload-file ./file.txt http://localhost:8787/file.txt
```

## Documentation

- [API reference](docs/api.md): public routes, headers, responses, errors
- [Limits](docs/limits.md): tiers, rate limits, file size, retention
- [Admin API](docs/admin.md): API keys, stats, rate limit reset
- [Development](docs/development.md): local setup, checks, project structure
- [Deployment](docs/deployment.md): Cloudflare resources, secrets, deploy, logs
- [Architecture](docs/architecture.md): components, data flow, database, cleanup, security

## License

[MIT](LICENSE)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Security problems go through [SECURITY.md](SECURITY.md).
