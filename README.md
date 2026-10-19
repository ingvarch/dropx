# DropX - File Transfer Service

[![CI](https://github.com/ingvarch/dropx/actions/workflows/ci.yml/badge.svg)](https://github.com/ingvarch/dropx/actions/workflows/ci.yml)
[![Security](https://github.com/ingvarch/dropx/actions/workflows/security.yml/badge.svg)](https://github.com/ingvarch/dropx/actions/workflows/security.yml)
[![Release](https://img.shields.io/github/v/release/ingvarch/dropx)](https://github.com/ingvarch/dropx/releases)
[![License](https://img.shields.io/github/license/ingvarch/dropx)](LICENSE)
[![dropx.click](https://img.shields.io/website?url=https%3A%2F%2Fdropx.click&label=dropx.click)](https://dropx.click)
[![Rust](https://img.shields.io/badge/Rust-dea584?logo=rust&logoColor=black)](Cargo.toml)
[![Cloudflare Workers](https://img.shields.io/badge/Cloudflare%20Workers-f38020?logo=cloudflare&logoColor=white)](https://developers.cloudflare.com/workers/)

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

You need Rust with the `wasm32-unknown-unknown` target, Bun and Node.js, see [docs/development.md](docs/development.md#prerequisites).

```bash
bun install
cp .dev.vars.example .dev.vars
bunx wrangler d1 migrations apply dropx-db --local
bunx wrangler dev
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
