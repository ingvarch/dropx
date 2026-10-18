# AGENTS.md

DropX is a file transfer service: a Rust Cloudflare Worker on worker-rs 0.8, built with worker-build. It uses R2 (files), D1 (metadata, API keys), a Durable Object (`RateLimiter`) and a daily cron trigger (cleanup). This is not a standard Rust application: it compiles to wasm32.

## Docs

Read the doc that matches your task before changing code. Update it in the same change when behavior changes. Plain language, no emoji.

- `docs/api.md`: public routes.
- `docs/admin.md`: admin routes.
- `docs/architecture.md`: components and request flows.
- `docs/limits.md`: size, expiry and rate limits.
- `docs/development.md`: local setup, checks, [project structure](docs/development.md#project-structure).
- `docs/deployment.md`: running on a Cloudflare account.
- `CONTRIBUTING.md`: workflow for contributors.

## Commands

The checks are in [docs/development.md](docs/development.md#checks). CI runs the same ones. Run all of them after every code change; the output must have no warnings and no failures.

```bash
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo check
cargo test
bun run test:e2e
```

- `bun install` installs wrangler. Run it with `bunx wrangler`, not a global install. wrangler itself runs on Node.js 22 or newer.
- `bun run test:e2e` starts its own `wrangler dev` on fresh local state. It does not touch `.wrangler/state` or `.dev.vars`.

## Production

Never run `bun run deploy`, `wrangler` with `--remote`, `wrangler deploy` or `wrangler secret` unless the maintainer asks.

## Tests

Write the failing test first, and confirm it fails for the right reason.

- Rust unit tests are for pure functions. They must call project code: no assertions on constants, no copies of the logic.
- `console_log!` and `worker::Headers` need the JS runtime and fail in `cargo test`. Extract a pure function and test that. Examples: `decide_limit` in `src/durable_objects/rate_limiter.rs`, `parse_check_response` in `src/services/rate_limiter.rs`, `remove_object_then_row` in `src/services/file_removal.rs` (run with `Waker::noop`).
- HTTP behavior goes in `tests/e2e/*.test.mjs` (`bun test`, written with `node:test` and `node:assert`), using `tests/e2e/helpers.mjs`:
  - Every request gets a random `CF-Connecting-IP` (`newClientIp`), so tests do not share rate limit counters.
  - `setExpiresAt`, `d1Raw` and `listR2ObjectSizes` read and write D1 and R2 through the Local Explorer.
  - `runScheduledCleanup` triggers the cron job.
  - To make a D1 insert fail, create a SQLite trigger that raises an abort, then drop it (see `files.test.mjs`). Limit it with `WHEN` to one file name and drop it in `finally`, so other uploads are not affected.
- `bun test` runs the files one after another in one process. All tests share one Worker and one database: undo what a test changes (triggers) and do not rely on rows from other tests.

## worker-rs

- Check APIs in the registry source of the locked version (`~/.cargo/registry/src/*/worker-<version>/src`), not in memory or older docs.
- Handlers return `Result<Response>`. Wrap responses in `Ok(...)`.
- Read params from `ctx` before moving `ctx.env`.
- Durable Object `storage.get` returns `Option`.
- `strip = true` in `Cargo.toml` breaks the build; see the comment there and issue #12.

## Code

- Constants live in `src/utils/constants.rs`. No magic numbers or strings.
- Error responses use the helpers in `src/utils/response.rs`.
- No `unwrap` or `expect` in production paths.
- Comments are short and say only what the code does not.
- DRY, YAGNI: extract repeated logic, build only what the task needs.

## Security

- Store only hashes of API keys and delete tokens (`hash_secret`), and hash client IPs with the `IP_HASH_KEY` secret (`client_id`, `hash_ip`) in `src/utils/auth.rs`.
- Compare secrets in constant time (`secrets_match`).
- Validate all input.
- Never log secrets.

## Git and pull requests

- Branch: `<type>/<short-name>`, for example `fix/inline-download-limit`.
- Commit: `type(scope): description`, imperative, lower case. Types and scopes are listed in [CONTRIBUTING.md](CONTRIBUTING.md#make-a-change). Each commit passes the checks.
- Pull request: fill in `.github/pull_request_template.md`, link the issue with `Closes #N`, add labels from the repository (a type label such as `bug`, `enhancement`, `documentation` or `chore`, plus area labels such as `api` or `storage`).
- Merge with rebase or squash.
- Commits and pull requests carry no AI attribution.
