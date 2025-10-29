# DropX - File Transfer Service

A fast, secure, and scalable file transfer service built with Rust on Cloudflare Workers, inspired by transfer.sh.

## Features

- Fast file uploads and downloads
- Automatic file expiration (7-30 days)
- Rate limiting with Durable Objects
- API key management for authenticated users
- R2 object storage integration
- D1 SQLite database for metadata
- Automated cleanup with cron jobs
- Admin dashboard for statistics

## Architecture

- **Cloudflare Workers** - Serverless compute with Rust
- **R2** - Object storage for files
- **D1** - SQLite database for metadata
- **Durable Objects** - Real-time rate limiting
- **Cron Triggers** - Automated cleanup

## Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wrangler CLI
npm install -g wrangler

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Install worker-build
cargo install worker-build
```

### Setup Cloudflare Resources

```bash
# Login to Cloudflare
wrangler login

# Create R2 bucket
wrangler r2 bucket create dropx-files

# Create D1 database
wrangler d1 create dropx-db

# Note the database_id from the output and update wrangler.toml

# Apply migrations
wrangler d1 execute dropx-db --file=migrations/0001_initial.sql
```

### Development

```bash
# Run locally
wrangler dev

# Test upload
curl --upload-file test.txt http://localhost:8787/

# Test download
curl http://localhost:8787/abc123 -O
```

### Deployment

```bash
# Deploy to production
wrangler deploy

# View logs
wrangler tail
```

## API Usage

### Upload File

```bash
# Simple upload
curl --upload-file photo.jpg https://dropx.yourdomain.workers.dev/

# With filename
curl --upload-file photo.jpg https://dropx.yourdomain.workers.dev/vacation.jpg

# With API key
curl --upload-file large.zip https://dropx.yourdomain.workers.dev/ \
  -H "X-API-Key: tfr_xxxxx"
```

### Download File

```bash
curl https://dropx.yourdomain.workers.dev/abc123 -O
```

### Get File Info

```bash
curl https://dropx.yourdomain.workers.dev/abc123/info
```

### Admin Endpoints

```bash
# Create API key
curl -X POST https://dropx.yourdomain.workers.dev/admin/api-keys \
  -H "X-Admin-Token: your-admin-token" \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "tier": "standard"}'

# Revoke API key
curl -X DELETE https://dropx.yourdomain.workers.dev/admin/api-keys/tfr_xxxxx \
  -H "X-Admin-Token: your-admin-token"

# Get statistics
curl https://dropx.yourdomain.workers.dev/admin/stats \
  -H "X-Admin-Token: your-admin-token"
```

## Rate Limits

| Tier | Hourly Limit | Daily Limit | Max File Size | Retention |
|------|--------------|-------------|---------------|-----------|
| Anonymous | 10 uploads | 50 uploads | 100 MB | 7 days |
| Standard API | 100 uploads | 500 uploads | 100 MB | 7 days |
| Premium API | 1000 uploads | 5000 uploads | 500 MB | 30 days |

## Configuration

Edit `wrangler.toml` to customize:

- `ADMIN_TOKEN` - Admin API authentication token
- `MAX_FILE_SIZE_MB` - Maximum file size in megabytes
- `DEFAULT_RETENTION_DAYS` - Default file retention period
- Rate limit values for different tiers

## Project Structure

```
dropx/
├── src/
│   ├── lib.rs              # Main router
│   ├── handlers/           # HTTP handlers
│   ├── services/           # Business logic
│   ├── models/             # Data structures
│   ├── utils/              # Helper functions
│   ├── durable_objects/    # Durable Objects
│   └── cleanup.rs          # Cron job
├── migrations/             # Database migrations
├── Cargo.toml              # Rust dependencies
├── wrangler.toml           # Cloudflare configuration
└── CLAUDE.md               # Development guidelines
```

## Development Guidelines

See `CLAUDE.md` for detailed Rust best practices and development guidelines.

## Cloudflare Limits (Free Tier)

- Workers: 100,000 requests/day
- R2: 10 GB storage, unlimited egress
- D1: 5 GB storage
- Cron: 3 triggers/day

## Security

- IP addresses are hashed, never stored in plain text
- Admin endpoints require token authentication
- Rate limiting prevents abuse
- Automatic file expiration ensures privacy
- No personal information collected

## License

MIT

## Support

For issues and feature requests, please open an issue on GitHub.
