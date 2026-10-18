# Development

Local setup, checks and project layout. To put the service on Cloudflare, see [deployment.md](deployment.md).

## Prerequisites

- Rust stable 1.91 or newer, with the WebAssembly target:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup target add wasm32-unknown-unknown
  ```

- Node.js and the `wrangler` CLI:

  ```bash
  npm install -g wrangler
  ```

- `worker-build`. The build command in `wrangler.toml` installs it and builds the Worker, so `wrangler dev` and `wrangler deploy` need no extra step.

## Local secrets

The only secret is `ADMIN_TOKEN`. Copy the example file and set your own value:

```bash
cp .dev.vars.example .dev.vars
```

`.dev.vars` is read by `wrangler dev` and is ignored by git.

## Local database

Apply the migrations from `migrations/` to the local D1 database:

```bash
wrangler d1 migrations apply dropx-db --local
```

Local R2, D1 and Durable Object data lives in `.wrangler/state`, which is ignored by git.

## Run

```bash
wrangler dev
```

The Worker listens on `http://localhost:8787`.

```bash
curl --upload-file ./file.txt http://localhost:8787/file.txt
curl http://localhost:8787/<id> -o file.txt
```

The `url` and `delete_url` in the upload response always start with `https://` and have no port, so replace them with `http://localhost:8787` when you use them locally.

To run the cleanup job locally, with `wrangler dev` running:

```bash
curl "http://localhost:8787/cdn-cgi/local/scheduled"
```

## Checks

Run these after every code change. The output must have no warnings and no failures.

```bash
cargo clippy && cargo check && cargo test
```

The rules for code, tests and commits are in [CLAUDE.md](../CLAUDE.md).

## Project structure

```
dropx/
├── src/
│   ├── lib.rs                  # Router, fetch and scheduled entry points
│   ├── cleanup.rs              # Cron job: remove expired files
│   ├── handlers/
│   │   ├── landing.rs          # GET /
│   │   ├── upload.rs           # POST /, PUT /:filename
│   │   ├── download.rs         # GET /:id
│   │   ├── get.rs              # GET /get/:id
│   │   ├── inline.rs           # GET /inline/:id
│   │   ├── info.rs             # GET /:id/info
│   │   ├── delete.rs           # DELETE /:id/:token
│   │   └── admin.rs            # /admin/* routes
│   ├── services/
│   │   ├── database.rs         # D1 queries
│   │   ├── storage.rs          # R2 operations
│   │   └── rate_limiter.rs     # Client for the Durable Object
│   ├── durable_objects/
│   │   └── rate_limiter.rs     # RateLimiter Durable Object
│   ├── models/
│   │   ├── file.rs             # File metadata and response types
│   │   └── api_key.rs          # API keys and tiers
│   ├── utils/
│   │   ├── constants.rs        # Limits, header names, status codes
│   │   ├── auth.rs             # IP hashing, API key generation, admin token check
│   │   ├── id_generator.rs     # File IDs and delete tokens
│   │   ├── storage_path.rs     # R2 object paths
│   │   ├── mime.rs             # MIME type detection
│   │   ├── content_disposition.rs
│   │   ├── content_negotiation.rs
│   │   ├── response.rs         # Error response helpers
│   │   ├── time.rs
│   │   ├── validation.rs
│   │   └── templates.rs        # Landing page rendering
│   └── templates/
│       ├── index.html          # Landing page
│       └── styles.css
├── migrations/                 # D1 migrations
├── docs/                       # Documentation
├── Cargo.toml
├── wrangler.toml               # Bindings, cron trigger, build command
├── .dev.vars.example           # Template for local secrets
└── CLAUDE.md                   # Development guidelines
```

How the parts fit together is in [architecture.md](architecture.md).
