# Phase 5: test-agent Subcommand - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Add `honeyprompt test-agent` CLI subcommand that spins up a temporary honeypot server, waits for AI agent callbacks within a configurable timeout, then prints a per-tier compliance scorecard and exits with a CI-friendly exit code. Also add a GitHub Actions CI workflow (test/clippy/fmt) that gates all new code.

</domain>

<decisions>
## Implementation Decisions

### Temporary Honeypot Setup
- **D-01:** Use the full existing pipeline (init + generate) in a tempdir. Create a temp directory, write a default honeyprompt.toml with the test-agent's --listen address as callback_base, run the generate pipeline, then serve from there. Auto-cleanup the tempdir on exit. This reuses 100% of existing code and automatically picks up catalog changes.
- **D-02:** The tempdir approach adds ~200ms startup overhead — acceptable for a test harness with a 60s+ timeout.

### Scorecard Output
- **D-03:** Tier summary only — per-tier (1/2/3) pass/fail + overall score (e.g., "1/3 tiers triggered") + verdict string. No individual callback details in v2. Keep output clean and CI-scannable.
- **D-04:** JSON format (`--format json`) mirrors the same fields: `{ tiers: [{tier: 1, triggered: true}, ...], score: "1/3", verdict: "PARTIALLY_COMPLIANT" }`. No callbacks[] array.
- **D-05:** Exit codes: 0 = no canaries triggered (agent is safe), 1 = one or more triggered, 2 = error/timeout with no data.

### Server Lifecycle
- **D-06:** Reuse the existing `serve()` pipeline pattern — build_router + broker_task + db_writer_task — but replace `shutdown_signal()` (Ctrl+C) with a `tokio::time::timeout` + `CancellationToken` for auto-shutdown after the configured duration.
- **D-07:** Need a new `detections_by_tier()` store function that returns per-tier hit counts (the existing `count_detections()` only returns aggregate). Query the SQLite DB after the server shuts down.

### CI Workflow
- **D-08:** Three parallel GitHub Actions jobs: test, clippy, fmt. Each runs independently for faster feedback.
- **D-09:** Use dtolnay/rust-toolchain + Swatinem/rust-cache. Rust stable channel. Do NOT use actions-rs/* (archived/unmaintained since 2022).
- **D-10:** Pin all third-party actions to full commit SHA (not version tag). Add a comment with the human-readable version for readability. This is a security-focused project — supply chain integrity matters.

### Claude's Discretion
- Server-ready signaling approach (oneshot channel vs polling) — Claude can choose the most idiomatic pattern
- Exact JSON schema field names — Claude follows Rust serde conventions
- Whether test-agent uses an in-memory SQLite DB or a tempdir DB — Claude decides based on what simplifies cleanup

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Server Pipeline
- `src/server/mod.rs` — `build_router()` at line 83, `serve()` at line 95. The full wiring pattern to replicate for test-agent.
- `src/broker/mod.rs` — `broker_task`, `db_writer_task`, `stdout_logger_task`. Event pipeline tasks to spawn.
- `src/store/mod.rs` — `count_detections()` at line 108. Needs extension for per-tier query.

### CLI Structure
- `src/cli/mod.rs` — `Commands` enum, all existing `*Args` structs. Add `TestAgent(TestAgentArgs)` variant.
- `src/main.rs` — Command dispatch. Add `Commands::TestAgent` arm.

### Generation Pipeline
- `src/generator/mod.rs` — `generate()` function. Called in tempdir to create honeypot assets.
- `src/config/mod.rs` — `write_default_config()`, `load_config()`. Used for tempdir setup.

### Design Doc
- `~/.gstack/projects/johnzilla-honeyprompt/john-main-design-20260329-180748.md` — test-agent Interface Sketch section has CLI flags, example output, exit codes.

### Research
- `.planning/research/ARCHITECTURE.md` — Integration points, data flow, suggested build order
- `.planning/research/PITFALLS.md` — Server-ready race, port leak after timeout, actions-rs deprecation
- `.planning/research/STACK.md` — tokio-util CancellationToken, dtolnay/rust-toolchain

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `server::build_router()` — Creates the Axum router with callback handler + static file serving. Directly reusable.
- `server::serve()` — Full pipeline wiring (channels, broker, db writer, router, listener). test-agent replicates this with a timeout instead of Ctrl+C.
- `generator::generate()` — Creates output/ with index.html, callback-map.json, robots.txt, ai.txt. Run in tempdir.
- `config::write_default_config()` — Writes a honeyprompt.toml. Used for tempdir setup.
- `store::count_detections()` — Aggregate detection count. Needs a per-tier variant.
- `broker::broker_task`, `broker::db_writer_task` — Event pipeline. Reuse as-is.
- `monitor/mod.rs` lines 910-926 — Integrated mode already replicates serve() with `with_graceful_shutdown`. Closest precedent for test-agent's timeout-based shutdown.

### Established Patterns
- All async entry points use `tokio::runtime::Runtime::new()` in main.rs (not #[tokio::main])
- Axum handlers use `ConnectInfo<SocketAddr>` for peer address extraction
- `into_make_service_with_connect_info::<SocketAddr>()` required for ConnectInfo to work
- Event pipeline: mpsc(256) for raw callbacks → broadcast(1024) for processed events

### Integration Points
- `cli/mod.rs` — Add `TestAgent(TestAgentArgs)` to Commands enum
- `main.rs` — Add `Commands::TestAgent` dispatch arm
- `lib.rs` — Add `pub mod test_agent;` if using a new module
- `Cargo.toml` — Add `tokio-util` as explicit dependency (currently transitive)
- `.github/workflows/ci.yml` — New file for CI workflow

</code_context>

<specifics>
## Specific Ideas

- The design doc sketches example output format at `~/.gstack/projects/johnzilla-honeyprompt/john-main-design-20260329-180748.md` in the "test-agent Interface Sketch" section. Follow that format for human output.
- Pitfalls research warns about server-ready race (need oneshot ready-channel) and port leak after timeout (need CancellationToken + handle.abort). Address both proactively.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-test-agent-subcommand*
*Context gathered: 2026-03-29*
