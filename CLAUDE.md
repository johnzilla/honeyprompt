## Project

**HoneyPrompt** is a terminal-first security tool that detects unsafe behavior by AI browsing agents. It generates honeypot pages with visible human warnings and hidden prompt-injection canaries, then records HTTP callbacks that prove graduated levels (Tiers 1–5) of agent compliance. Solo project, no external users yet: breaking changes are fine.

Backlog and parked ideas live in `TODOS.md`. Releases are tagged `vX.Y.Z`; there are no milestone or phase numbers.

### Constraints

- Rust, single static binary; Linux and macOS
- Ethics: every generated page carries a visible human warning; payloads are auditable; never collect real secrets, credentials or cookies
- Fast startup, low memory

## Stack

- Clap (derive) CLI, Axum HTTP, Ratatui TUI, dialoguer prompts
- SQLite via rusqlite (sync) + tokio-rusqlite (async), WAL mode
- rust-embed for bundled assets (`assets/templates`, `assets/catalog`, `assets/crawlers`)
- tokio runtime
- GitHub Actions: `ci.yml` (fmt, clippy `-D warnings`, test) and `release.yml` (4-target binaries on `v*` tags)
- Docker + Caddy deploy (`deploy/`); live demo honeyprompt.sh runs `serve /landing --domain honeyprompt.sh` (the image ships no config); honeyprompt.dev is GitHub Pages from `docs/`

## Conventions

- Async entry points use `tokio::runtime::Runtime::new()` in `main.rs`, not `#[tokio::main]`
- Event pipeline: server `try_send` (drops counted in `DropCounters`) → mpsc(256) → broker → mpsc(256) → DB writer → broadcast(1024) of persisted events (real replay/fire count) → TUI / stdout logger. Shutdown drains the stages in order
- `build_router()` is the reusable Axum router constructor; peer address via `ConnectInfo<SocketAddr>` + `into_make_service_with_connect_info`
- Config precedence: CLI flags > config file > defaults (`config_with_overrides`)
- Unit tests in `#[cfg(test)] mod tests` per module; integration tests in `tests/`
- GitHub Actions are SHA-pinned with version comments
- Before committing: `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test`

## Architecture (`src/`)

- `cli/`, `main.rs`: Commands enum (Init, Generate, Serve, Monitor, Report, Setup, TestAgent). `serve` takes `--domain`/`--bind`/`--tiers` for zero-config mode
- `setup/`: interactive wizard, writes `honeyprompt.toml`
- `generator/`: loads payload catalog, assigns nonces, renders index.html, robots.txt, ai.txt, security.txt
- `server/`: static honeypot + callbacks `/cb/v1/{nonce}` (T1–T3), `/cb/v4/{nonce}/{b64_list}`, `/cb/v5/{nonce}/{proof}`, and `/stats` JSON with CORS
- `broker/`: enriches raw callbacks with fingerprint/classification, fans out to DB writer and stdout logger
- `fingerprint/`, `crawler_catalog/`: request fingerprinting and known-crawler classification
- `store/`: SQLite schema + migrations, replay detection, session grouping, per-tier queries
- `monitor/`: Ratatui TUI, integrated-server or DB-attach mode
- `report/`: Markdown disclosure report
- `test_agent/`: ephemeral tempdir generate→serve→score pipeline with per-tier scorecard and CI exit codes
