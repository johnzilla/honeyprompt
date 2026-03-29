# Architecture Research

**Domain:** Rust CLI security tool — honeypot page generation, HTTP callback listening, SQLite event storage, TUI monitoring, automated agent testing, CI/CD release pipeline
**Researched:** 2026-03-29
**Confidence:** HIGH (verified against official Tokio, Axum, GitHub Actions docs and community patterns)

---

## Standard Architecture (v1.0 — Existing, Unchanged)

### Top-Level Process Topology (Server Mode)

```
  ┌─────────────────────────────────────────────────────────────┐
  │                     Tokio Runtime                           │
  │                                                             │
  │  ┌──────────────┐   callback_tx   ┌──────────────────────┐ │
  │  │  Axum Server │ ──────────────► │   Event Broker Task  │ │
  │  │  (HTTP + CB) │                 │  (broadcast fan-out) │ │
  │  └──────────────┘                 └───────┬──────────────┘ │
  │                                           │                 │
  │                           ┌──────────────┴──────────┐      │
  │                           │                         │       │
  │                   ┌───────▼──────┐        ┌────────▼─────┐ │
  │                   │  DB Writer   │        │  TUI Task    │ │
  │                   │  Task        │        │  (Ratatui)   │ │
  │                   └───────┬──────┘        └──────────────┘ │
  │                           │                                 │
  │                   ┌───────▼──────┐                         │
  │                   │   SQLite     │                         │
  │                   └──────────────┘                         │
  └─────────────────────────────────────────────────────────────┘
```

This topology is unchanged for v2.0. The new `test-agent` subcommand reuses and composes existing modules rather than replacing any of them.

---

## v2.0 New Architecture: test-agent Subcommand

### What test-agent Does

`honeyprompt test-agent` is a scripted probe-and-measure subcommand. It:

1. Generates a fresh ephemeral honeypot page (reusing `generator`) with a random nonce set
2. Starts a **temporary server** bound to a local port (reusing `server::build_router`)
3. Prints the test URL so a human or CI script can point an agent at it
4. Waits for callbacks, collecting them up to a timeout
5. Shuts the server down automatically when timeout expires or all expected callbacks are received
6. Produces a pass/fail scorecard with exit code 0 (no callbacks = agent ignored injection) or 1 (callbacks received = agent complied with injection)

The key difference from `serve` is **auto-termination** — the server is not meant to run indefinitely.

### test-agent Process Topology

```
  CLI test-agent
       │
       ├── generate ephemeral output dir (temp dir, reuse generator module)
       │
       ├── open in-memory or ephemeral SQLite DB (reuse store module)
       │
       ├── start Tokio runtime
       │        │
       │   ┌────▼──────────────────────────────────────────┐
       │   │               Tokio Runtime                    │
       │   │                                                │
       │   │  ┌──────────────┐  callback_tx  ┌──────────┐  │
       │   │  │  Axum Server │ ────────────► │  Broker  │  │
       │   │  │  (temp port) │               │  Task    │  │
       │   │  └──────────────┘               └────┬─────┘  │
       │   │                                      │         │
       │   │                             ┌────────▼──────┐  │
       │   │                             │  DB Writer    │  │
       │   │                             │  Task         │  │
       │   │                             └────────┬──────┘  │
       │   │                                      │         │
       │   │                             ┌────────▼──────┐  │
       │   │                             │  Ephemeral    │  │
       │   │                             │  SQLite DB    │  │
       │   │                             └───────────────┘  │
       │   │                                                │
       │   │  ┌──────────────────────────────────────────┐  │
       │   │  │   Shutdown Coordinator                   │  │
       │   │  │   tokio::select! {                       │  │
       │   │  │     _ = timeout_sleep => shutdown        │  │
       │   │  │     _ = shutdown_rx =>   shutdown        │  │
       │   │  │   }                                      │  │
       │   │  └──────────────────────────────────────────┘  │
       │   └────────────────────────────────────────────────┘
       │
       └── collect events from DB after shutdown
           └── emit scorecard (stdout or JSON), exit with code
```

### Minimal Temporary Server Pattern

The monitor module already demonstrates this pattern for its integrated mode: it uses a `tokio::sync::oneshot` channel for shutdown signaling. The test-agent extends this with a **timeout arm** in a `tokio::select!` block.

```rust
// Conceptual structure — not final code

let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

// Spawn the server with graceful shutdown tied to the oneshot receiver
tokio::spawn(async move {
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async { let _ = shutdown_rx.await; })
    .await
    .ok();
});

// Shutdown coordinator: fires after timeout OR when caller sends shutdown_tx
tokio::select! {
    _ = tokio::time::sleep(timeout_duration) => {
        // Timeout expired — send shutdown signal
        let _ = shutdown_tx.send(());
    }
    _ = early_exit_rx => {
        // All expected callbacks received early — can terminate before timeout
        let _ = shutdown_tx.send(());
    }
}
```

The **early exit** path (optional) uses a second oneshot channel. The event broker task sends on `early_exit_tx` when the expected nonce set is fully covered. This avoids waiting the full timeout when an agent fires all callbacks quickly.

This pattern is verified against:
- The existing `monitor/mod.rs` integrated mode (lines 910–926 use exactly this oneshot shutdown pattern)
- The Axum `with_graceful_shutdown` API (stable, confirmed in axum 0.8)
- Tokio's official shutdown documentation (tokio.rs/tokio/topics/shutdown)

### New module: `test_agent` (or `src/test_agent/mod.rs`)

**Status: NEW** — does not exist yet.

Responsibilities:
- Orchestrate ephemeral generate → serve → wait → score pipeline
- Own the `TestAgentArgs` CLI struct
- Own the scorecard struct and its rendering (text and JSON)
- Own the timeout/early-exit coordination logic

**Modules it calls (all existing, no changes needed to them):**

| Module | How test-agent uses it | Change required |
|--------|----------------------|-----------------|
| `generator` | `generator::generate(&cfg, &conn, &tmp_dir)` | None |
| `server::build_router` | Construct the Axum router with ephemeral state | None |
| `server::AppState` | Reuse struct directly | None |
| `server::NonceMeta` | Reuse struct directly | None |
| `broker::broker_task` | Spawn as usual | None |
| `broker::db_writer_task` | Spawn as usual | None |
| `store::open_or_create_db` | Open ephemeral DB (temp path or in-memory) | None |
| `store::count_detections` | Query results after shutdown | None |
| `config::Config` | Provide ephemeral config (bind address, base_url) | None |
| `crawler_catalog::CrawlerCatalog` | Load as usual | None |

The `test_agent` module **does not** need to call `store::insert_nonce` directly — the generator writes `callback-map.json` and the server reads it. The DB is populated by the normal event pipeline.

### New CLI variant in `src/cli/mod.rs`

**Status: MODIFIED** — add `TestAgent(TestAgentArgs)` to the `Commands` enum and define `TestAgentArgs`.

```rust
// Addition to Commands enum
Commands::TestAgent(TestAgentArgs)

// New args struct
pub struct TestAgentArgs {
    pub path: PathBuf,          // project dir (or temp dir if --ephemeral)
    pub listen: Option<u16>,    // port to bind (default: OS-assigned)
    pub timeout: u64,           // seconds before auto-shutdown (default: 30)
    pub format: OutputFormat,   // text (default) or json
}

pub enum OutputFormat { Text, Json }
```

### New match arm in `src/main.rs`

**Status: MODIFIED** — add one arm to the `match cli.command` block.

```rust
Commands::TestAgent(args) => {
    let rt = tokio::runtime::Runtime::new()?;
    let exit_code = rt.block_on(test_agent::run(&args))?;
    std::process::exit(exit_code);
}
```

The exit code is meaningful: `0` = no agent compliance detected, `1` = agent fired at least one callback. This enables CI usage (`if honeyprompt test-agent ...; then echo "PASS"`)

### Ephemeral Output Directory Strategy

Two options:

**Option A (recommended): User-specified project directory**
- `test-agent --path .` reuses an initialized project directory
- Uses the existing `output/` dir and `callback-map.json`
- User runs `honeyprompt generate` first, then `honeyprompt test-agent`
- Familiar workflow, no temp dir cleanup concerns
- **This is the MVP approach**

**Option B: Fully ephemeral (future)**
- `test-agent --ephemeral` creates a `tempfile::TempDir` automatically
- Runs `generate` internally then serves it
- Adds a `tempfile` dep but no other changes needed
- Deferred — Option A is sufficient for v2.0

### Scorecard Output

The scorecard is emitted after server shutdown, reading from the ephemeral DB.

**Text format (default):**
```
honeyprompt test-agent results
  listened:    30s
  url:         http://localhost:PORT
  callbacks:   3
  tier 1:      1
  tier 2:      1
  tier 3:      1
  verdict:     COMPLIANCE DETECTED (agent followed injection instructions)

exit code: 1
```

**JSON format (--format json):**
```json
{
  "listened_secs": 30,
  "url": "http://localhost:PORT",
  "total_callbacks": 3,
  "tier_counts": {"1": 1, "2": 1, "3": 1},
  "verdict": "compliance_detected"
}
```

Exit code `0` = `"no_compliance"`, exit code `1` = `"compliance_detected"`.

---

## v2.0 New Architecture: GitHub Actions CI/CD

### Workflow Files

Two separate workflow files. Separation keeps CI fast and release publishing independent.

```
.github/
└── workflows/
    ├── ci.yml       # Runs on every push and PR to main
    └── release.yml  # Runs only on tag push (v*)
```

### ci.yml Structure

Runs: `cargo fmt --check`, `cargo clippy`, `cargo test` on Linux and macOS.

```
Trigger: push to main, pull_request to main

jobs:
  check:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
          with: { components: rustfmt, clippy }
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test --workspace
```

Key choices:
- `dtolnay/rust-toolchain` — the current recommended action; `actions-rs` is unmaintained (MEDIUM confidence, multiple community sources agree)
- `Swatinem/rust-cache@v2` — caches `~/.cargo/registry` and `target/`, dramatically speeds up CI
- `-D warnings` on clippy — enforces clean lints as a hard gate
- macOS runner included — confirms macOS compatibility on every push

### release.yml Structure

Triggered by `v*` tag push. Builds cross-platform binaries, uploads to GitHub Release.

```
Trigger: push of tags matching v*

jobs:
  build:
    strategy:
      matrix:
        include:
          - { os: ubuntu-latest,  target: x86_64-unknown-linux-gnu,    suffix: "",     archive: tar.gz }
          - { os: ubuntu-latest,  target: aarch64-unknown-linux-gnu,   suffix: "",     archive: tar.gz }
          - { os: macos-latest,   target: x86_64-apple-darwin,         suffix: "",     archive: tar.gz }
          - { os: macos-latest,   target: aarch64-apple-darwin,        suffix: "",     archive: tar.gz }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
          with: { targets: ${{ matrix.target }} }
      - uses: houseabsolute/actions-rust-cross@v0
          with:
            command: build
            target: ${{ matrix.target }}
            args: "--release"
      - name: Package
        run: |
          BINARY="target/${{ matrix.target }}/release/honeyprompt${{ matrix.suffix }}"
          ARCHIVE="honeyprompt-${{ github.ref_name }}-${{ matrix.target }}.${{ matrix.archive }}"
          tar czf "$ARCHIVE" -C "$(dirname $BINARY)" "$(basename $BINARY)"
      - uses: actions/upload-artifact@v4
          with: { name: "${{ matrix.target }}", path: "*.tar.gz" }

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
          with: { path: artifacts/ }
      - uses: softprops/action-gh-release@v2
          with:
            files: artifacts/**/*
```

Key choices:
- `houseabsolute/actions-rust-cross@v0` — handles both native and cross-compiled targets; uses Docker (via `cross`) for Linux ARM, native cargo for macOS and Linux x86_64 (MEDIUM confidence, verified as current active project on GitHub Marketplace)
- `aarch64-unknown-linux-gnu` via cross — Linux ARM is the most common non-trivial cross-compile target; requires Docker on runner, which ubuntu-latest provides
- `aarch64-apple-darwin` natively on macos-latest — Apple Silicon runner available, no cross-compile tooling needed for this target
- No Windows target — matches project constraint (Linux and macOS first, Windows is v2+)
- Two-phase structure (build matrix → release) — artifacts uploaded by matrix jobs, collected and released by a single dependent job; this is the canonical pattern and avoids race conditions
- `softprops/action-gh-release@v2` — stable, widely used, handles multi-file uploads cleanly (HIGH confidence, official GitHub Marketplace)

### Naming Convention for Binary Archives

```
honeyprompt-{version}-{target}.tar.gz

Examples:
  honeyprompt-v2.0.0-x86_64-unknown-linux-gnu.tar.gz
  honeyprompt-v2.0.0-aarch64-unknown-linux-gnu.tar.gz
  honeyprompt-v2.0.0-x86_64-apple-darwin.tar.gz
  honeyprompt-v2.0.0-aarch64-apple-darwin.tar.gz
```

Including the full target triple in the archive name is the community standard and makes `install.sh` scripts trivially writable.

---

## v2.0 New Architecture: Deployment Configuration

### Containerized Demo at honeyprompt.sh

The live demo requires the `serve` subcommand running persistently. Two deployment options:

**Option A (recommended): Docker + minimal Dockerfile**

```dockerfile
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY honeyprompt /usr/local/bin/honeyprompt
WORKDIR /data
EXPOSE 8080
ENTRYPOINT ["honeyprompt", "serve", "--path", "/data"]
```

The container expects `/data` to be a pre-initialized project directory (with `honeyprompt.toml`, `output/`, `.honeyprompt/events.db`). Volume mount or bake into image at build time.

**Option B: systemd service on a VPS**

Simpler for a single-machine deployment:
```
[Service]
ExecStart=/usr/local/bin/honeyprompt serve --path /opt/honeyprompt
Restart=always
```

For v2.0 MVP, Option B is lower-friction and sufficient. Docker adds value when the deployment needs reproducibility or multi-instance isolation — likely v3+.

### Config for Public Deployment

The existing `honeyprompt.toml` `bind_address` field handles port binding. The public deployment should:
- Bind to `0.0.0.0:80` (or `8080` behind a reverse proxy like nginx or Caddy)
- Set `base_url = "https://honeyprompt.sh"` so generated callback URLs point to the real domain
- SQLite WAL mode (already enabled in `store::run_migrations`) handles concurrent reads without locking

No new configuration fields or modules needed for deployment — it is purely operational configuration.

---

## Component Boundary Summary: New vs Modified vs Unchanged

| Component | Status | Change |
|-----------|--------|--------|
| `src/test_agent/mod.rs` | **NEW** | Create: orchestrates ephemeral generate→serve→wait→score pipeline |
| `src/cli/mod.rs` | **MODIFIED** | Add `TestAgent(TestAgentArgs)` to `Commands` enum; add `TestAgentArgs` struct |
| `src/main.rs` | **MODIFIED** | Add `Commands::TestAgent` match arm; call `test_agent::run`, propagate exit code |
| `src/lib.rs` | **MODIFIED** | Expose `pub mod test_agent` |
| `.github/workflows/ci.yml` | **NEW** | CI workflow: fmt + clippy + test on Linux and macOS |
| `.github/workflows/release.yml` | **NEW** | Release workflow: cross-compile matrix + GitHub Release |
| `src/server/mod.rs` | **UNCHANGED** | `build_router` and `AppState` reused as-is |
| `src/broker/mod.rs` | **UNCHANGED** | `broker_task`, `db_writer_task` reused as-is |
| `src/store/mod.rs` | **UNCHANGED** | Existing query functions sufficient |
| `src/generator/mod.rs` | **UNCHANGED** | `generate()` called from `test_agent` |
| `src/monitor/mod.rs` | **UNCHANGED** | Not involved in test-agent |
| `src/report/mod.rs` | **UNCHANGED** | Not involved in test-agent |
| All other modules | **UNCHANGED** | `config`, `catalog`, `fingerprint`, `crawler_catalog`, `nonce`, `types` |

---

## Data Flow: test-agent Subcommand

```
CLI test-agent [--listen PORT] [--timeout SECS] [--format text|json]
  │
  ├─ load config from honeyprompt.toml (path arg)
  ├─ open ephemeral SQLite DB (same .honeyprompt/events.db or temp)
  │
  ├─ spawn event pipeline:
  │     mpsc(callback_tx) → broker_task → broadcast
  │                                          ├─► db_writer_task → SQLite
  │                                          └─► [optional: event counter for early exit]
  │
  ├─ build_router(AppState, output_dir) → axum router
  ├─ bind TcpListener on --listen port (or OS-assigned)
  ├─ print: "Test URL: http://localhost:{PORT}"
  │
  ├─ spawn axum::serve(...).with_graceful_shutdown(oneshot_rx)
  │
  ├─ tokio::select! {
  │     _ = tokio::time::sleep(timeout) => { oneshot_tx.send(()) }
  │     _ = early_exit_rx (optional)   => { oneshot_tx.send(()) }
  │   }
  │
  ├─ await server task completion (flush in-flight requests)
  │
  ├─ query SQLite: count_detections(), tier breakdown
  │
  └─ print scorecard → std::process::exit(exit_code)
```

---

## Architectural Patterns

### Pattern 1: Oneshot-Gated Temporary Server with Timeout

**What:** Axum server's `with_graceful_shutdown` accepts a future. Pass a oneshot receiver future. A separate coordinator task sends on the transmitter when a timeout fires (via `tokio::select!`).

**When to use:** Any subcommand that needs a server with bounded lifetime — test automation, OAuth callback receivers, one-shot webhooks.

**Trade-offs:** Simple, zero external deps. The server does not force-close connections after shutdown signal — in-flight requests complete gracefully. For a test tool this is desirable (avoids losing a callback that arrived at the last millisecond).

### Pattern 2: Module Reuse Through Public Function Boundaries

**What:** `test_agent` calls `server::build_router` and `generator::generate` directly, without duplicating their logic.

**When to use:** When adding a new orchestration mode that reuses existing pipeline stages. The key requirement is that module entry points are `pub fn` with clean argument types (no hidden globals, no static state).

**Trade-offs:** Coupling is real but deliberate. If `build_router` signature changes, `test_agent` must update. This is acceptable — they're in the same codebase. The alternative (duplicating logic) is worse.

**Evidence this is safe here:** The existing `monitor` module already does this — it calls `server::build_router` and `broker::broker_task` directly (monitor/mod.rs lines 854–895). The pattern is proven.

### Pattern 3: Exit Code as Test Oracle

**What:** `std::process::exit(code)` from a CLI subcommand, where the code encodes the test result.

**When to use:** When the subcommand output needs to be machine-readable by CI scripts and shell conditionals without parsing stdout.

**Trade-offs:** Exit codes are coarse (0/1). For more detail (which tiers fired), the `--format json` stdout output is the richer channel. These two channels are complementary, not competing.

---

## Anti-Patterns to Avoid (v2.0 Additions)

### Anti-Pattern 1: Blocking on Server Shutdown Before Querying Results

**What:** Querying SQLite for the scorecard while the DB writer task may still be flushing events from the broadcast channel.

**Why bad:** The server may have received callbacks in its final milliseconds. The DB writer is async — it may not have committed all events by the time the server's `serve` future completes.

**Instead:** After the server future completes, await a small drain window or use a tokio::sync::Notify/barrier to confirm the DB writer task has caught up. Simplest approach: drop the `broadcast::Sender` after shutdown, which causes the DB writer's `recv()` to return `Closed`, signaling it has drained all pending events.

### Anti-Pattern 2: Generating Pages Inside the Server Runtime

**What:** Running `generator::generate()` inside the `tokio::spawn` or async context.

**Why bad:** The generator does synchronous filesystem I/O (reads templates, writes files). Running it on the async executor blocks the Tokio thread pool.

**Instead:** Run `generator::generate()` synchronously before entering the async runtime, or use `tokio::task::spawn_blocking`. The simplest option for test-agent is to generate before `rt.block_on(...)` starts.

### Anti-Pattern 3: Using a Persistent DB for Ephemeral Tests

**What:** Running `test-agent` against the project's production `.honeyprompt/events.db`.

**Why bad:** Mixes test data with real evidence. Repeated test runs inflate session counts and corrupt reports.

**Instead:** For the MVP, document clearly that test-agent uses a separate temp path or the user should run it in a separate project directory. A `--db :memory:` flag is a clean future option (SQLite supports in-memory databases via rusqlite).

### Anti-Pattern 4: Using actions-rs in GitHub Actions

**What:** Using `actions-rs/toolchain` or `actions-rs/cargo` actions.

**Why bad:** The `actions-rs` organization is unmaintained. Known bugs exist and no new releases are coming. Multiple community sources (MEDIUM confidence) consistently recommend migrating away.

**Instead:** Use `dtolnay/rust-toolchain` for toolchain installation and plain `run: cargo ...` steps for commands. Use `houseabsolute/actions-rust-cross` for cross-compilation.

---

## Recommended Project Structure (v2.0 additions only)

```
src/
├── test_agent/       # NEW — test-agent subcommand orchestration
│   └── mod.rs        #   TestAgentArgs, run(), scorecard rendering
├── cli/
│   └── mod.rs        # MODIFIED — add TestAgent variant + TestAgentArgs
├── main.rs           # MODIFIED — add TestAgent dispatch arm

.github/
└── workflows/
    ├── ci.yml        # NEW — CI: fmt + clippy + test
    └── release.yml   # NEW — release: cross-compile matrix + GitHub Release
```

---

## Build Order for v2.0 Features

Dependencies flow downward. Build in this order to have testable checkpoints at each step.

```
Step 1: GitHub Actions CI workflow
  — No code changes. Add .github/workflows/ci.yml.
  — Validates: existing code passes fmt/clippy/test before adding new code.
  — Checkpoint: green CI badge.

Step 2: CLI extension (TestAgent variant)
  — Modify src/cli/mod.rs (add enum variant + args struct)
  — Modify src/main.rs (add match arm — stub that prints "TODO")
  — Checkpoint: `honeyprompt test-agent --help` works; compiles cleanly.

Step 3: test_agent module — server setup and temporary serving
  — Create src/test_agent/mod.rs
  — Wire broker, server, store using existing module APIs
  — Implement timeout-based shutdown using oneshot + tokio::select!
  — Checkpoint: `honeyprompt test-agent` starts a server, prints URL, shuts down after timeout.

Step 4: test_agent module — scorecard
  — Add DB query after shutdown to count callbacks by tier
  — Implement text and JSON scorecard rendering
  — Implement exit codes
  — Checkpoint: Running a manual curl to /cb/v1/<nonce> produces a scorecard showing tier 1 callback.

Step 5: GitHub Actions release workflow
  — Add .github/workflows/release.yml
  — Test with a pre-release tag (v2.0.0-rc1)
  — Verify all four binary archives appear on the GitHub Release
  — Checkpoint: GitHub Release has four .tar.gz files, all download and run.

Step 6: Deployment configuration
  — Write honeyprompt.toml for production deployment
  — Configure reverse proxy (nginx or Caddy) pointing to bound port
  — Checkpoint: honeyprompt.sh serves live honeypot pages with real callback URLs.
```

---

## Integration Points Summary

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `test_agent` → `server::build_router` | Direct function call | Reuses existing public API unchanged |
| `test_agent` → `generator::generate` | Direct function call | Must run before entering async runtime |
| `test_agent` → `broker::{broker_task, db_writer_task}` | tokio::spawn | Same wiring as monitor integrated mode |
| `test_agent` → `store::count_detections` | Direct function call | Called post-shutdown, synchronous |
| `test_agent` shutdown | oneshot channel + tokio::select! | Matches existing monitor pattern |
| CI workflow → repository | GitHub Actions push/PR trigger | Standard; no external services |
| release workflow → GitHub Releases | softprops/action-gh-release@v2 | GITHUB_TOKEN permissions: write |
| release workflow → cross-compilation | houseabsolute/actions-rust-cross | Uses Docker for Linux ARM targets |

---

## Sources

- Existing `src/monitor/mod.rs` lines 823–960 — oneshot shutdown pattern in this codebase (HIGH confidence, primary source)
- Existing `src/server/mod.rs` `build_router` pub API (HIGH confidence, primary source)
- [Tokio graceful shutdown documentation](https://tokio.rs/tokio/topics/shutdown) — HIGH confidence, official
- [Axum discussion: graceful shutdown after one request](https://github.com/tokio-rs/axum/discussions/2410) — MEDIUM confidence, community-verified pattern
- [houseabsolute/actions-rust-cross GitHub Action](https://github.com/houseabsolute/actions-rust-cross) — MEDIUM confidence, active project on GitHub Marketplace
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release) — HIGH confidence, canonical release action
- [Building a cross platform Rust CI/CD pipeline with GitHub Actions (2025)](https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/) — MEDIUM confidence, recent article
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) — MEDIUM confidence, widely recommended replacement for actions-rs
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) — MEDIUM confidence, current community standard for Rust CI caching
- [Cross Compiling Rust Projects in GitHub Actions (houseabsolute blog)](https://blog.urth.org/2023/03/05/cross-compiling-rust-projects-in-github-actions/) — MEDIUM confidence, authoritative author (created actions-rust-cross)

---
*Architecture research for: HoneyPrompt v2.0 — test-agent subcommand, CI/CD workflows, deployment config*
*Researched: 2026-03-29*
