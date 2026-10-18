# Development

Local setup, checks and project layout. To put the service on Cloudflare, see [deployment.md](deployment.md).

## Prerequisites

- Rust stable 1.91 or newer, with the WebAssembly target:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup target add wasm32-unknown-unknown
  ```

- [Bun](https://bun.sh) for the dependencies, the scripts and the e2e tests. `wrangler` is a dev dependency of the project; install it with:

  ```bash
  bun install
  ```

  The commands below run it with `bunx wrangler`.

- Node.js 22 or newer: `wrangler` runs on it.

- `worker-build`. The build command in `wrangler.toml` installs it and builds the Worker, so `wrangler dev` and `wrangler deploy` need no extra step.

## Local secrets

The secrets are `ADMIN_TOKEN` and `IP_HASH_KEY` (a random value, for example from `openssl rand -hex 32`). Copy the example file and set your own values:

```bash
cp .dev.vars.example .dev.vars
```

`.dev.vars` is read by `wrangler dev` and is ignored by git.

## Local database

Apply the migrations from `migrations/` to the local D1 database:

```bash
bunx wrangler d1 migrations apply dropx-db --local
```

Local R2, D1 and Durable Object data lives in `.wrangler/state`, which is ignored by git.

## Run

```bash
bunx wrangler dev
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
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo check
cargo test
bun run test:e2e
```

`cargo test` covers the pure helpers. Unit tests must call project code: do not assert constants or a copy of the logic. `bun run test:e2e` covers the HTTP behavior: it applies the migrations to a fresh temporary state, starts the Worker with `wrangler dev` on a free port, runs `tests/e2e/*.test.mjs` with `bun test` and stops the Worker. It generates a random `ADMIN_TOKEN` and `IP_HASH_KEY` for the run and does not touch `.wrangler/state` or `.dev.vars`. Each test sends requests from its own random `CF-Connecting-IP`, so tests do not share rate limit counters.

GitHub Actions runs the same checks on every pull request and on `main` (`.github/workflows/ci.yml`).

The rules for code, tests and commits are in [AGENTS.md](../AGENTS.md).

## Project structure

```
dropx/
├── src/
│   ├── lib.rs                  # Router, fetch and scheduled entry points
│   ├── cleanup.rs              # Cron job: remove expired files
│   ├── handlers/
│   │   ├── landing.rs          # GET /
│   │   ├── upload.rs           # POST /, PUT /:filename
│   │   ├── download.rs         # GET /:id, /get/:id, /inline/:id
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
│   │   ├── auth.rs             # secret and IP hashing, API key generation, admin token check
│   │   ├── id_generator.rs     # File IDs and delete tokens
│   │   ├── storage_path.rs     # R2 object paths
│   │   ├── mime.rs             # MIME type detection
│   │   ├── content_disposition.rs
│   │   ├── content_negotiation.rs
│   │   ├── response.rs         # Error response helpers
│   │   ├── time.rs
│   │   └── templates.rs        # Landing page rendering
│   └── templates/
│       ├── index.html          # Landing page
│       └── styles.css
├── tests/e2e/                  # End-to-end tests against wrangler dev
│   ├── run.mjs                 # Starts the Worker, runs the tests, stops it
│   └── helpers.mjs             # Request helpers shared by the tests
├── migrations/                 # D1 migrations
├── docs/                       # Documentation
├── .github/workflows/ci.yml    # Checks on pull requests and main
├── Cargo.toml
├── package.json                # wrangler, the e2e test and deploy scripts
├── bun.lock                    # Locked dev dependency versions
├── wrangler.toml               # Bindings, cron trigger, build command
├── .dev.vars.example           # Template for local secrets
└── AGENTS.md                   # Instructions for coding agents
```

How the parts fit together is in [architecture.md](architecture.md).
