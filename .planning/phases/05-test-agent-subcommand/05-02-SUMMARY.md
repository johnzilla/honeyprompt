---
phase: 05-test-agent-subcommand
plan: 02
subsystem: cli
tags: [rust, axum, tokio, tokio-util, cancellationtoken, sqlite, clap, tempfile, test-agent]

# Dependency graph
requires:
  - phase: 05-01
    provides: CI baseline and GitHub Actions workflow ensuring green build
  - phase: 01-foundation
    provides: store, server, broker, generator, config, crawler_catalog modules
provides:
  - test_agent module with ephemeral server lifecycle (src/test_agent/mod.rs)
  - Scorecard struct with tiers[bool;3], tier_counts[u32;3], exit_code()
  - TestAgent CLI subcommand with --listen, --timeout, --format flags
  - OutputFormat enum (Text/Json) for scorecard rendering
  - detections_by_tier() store query returning [u32;3] per-tier counts
  - Integration test stubs for TEST-01 and TEST-02 (tests/test_test_agent.rs)
affects: [05-03-scorecard-rendering, test-agent-scorecard]

# Tech tracking
tech-stack:
  added:
    - tokio-util = { version = "0.7", features = ["rt"] } (CancellationToken)
    - tempfile = "3" promoted from dev-deps to [dependencies]
  patterns:
    - Pre-bind std::net::TcpListener then convert via tokio::net::TcpListener::from_std() — eliminates port-stealing race condition
    - CancellationToken from tokio-util for bounded server lifetime with graceful shutdown
    - Drop broadcast Sender after broker drains before querying SQLite for scorecard
    - generator::generate() runs synchronously BEFORE rt.block_on() — no blocking of Tokio threads
    - TempDir wraps ephemeral project directory — auto-deleted on drop (panic-safe)

key-files:
  created:
    - src/test_agent/mod.rs (Scorecard struct, run() orchestrator, run_async() server lifecycle)
    - tests/test_test_agent.rs (integration test stubs for TEST-01/TEST-02)
  modified:
    - Cargo.toml (added tokio-util, tempfile to [dependencies])
    - src/cli/mod.rs (OutputFormat enum, TestAgent variant, TestAgentArgs struct)
    - src/store/mod.rs (detections_by_tier() function + unit test)
    - src/lib.rs (pub mod test_agent)
    - src/main.rs (Commands::TestAgent dispatch arm)

key-decisions:
  - "Pre-bind TCP socket with std::net::TcpListener before async runtime to avoid port-stealing race; convert via from_std() instead of rebinding"
  - "Await broker_handle before drop(event_tx) to ensure all callback events are broadcast before closing the channel to db_writer"
  - "generator::generate() called synchronously before rt.block_on() to avoid blocking Tokio thread pool on sync filesystem I/O"
  - "Exit codes: 0=no callbacks, 1=one or more tier triggered, 2=error (std::process::exit per D-05)"
  - "Scorecard rendering deferred to Plan 03 — Plan 02 wires exit codes only"

patterns-established:
  - "Ephemeral project pattern: TempDir + write_default_config + generate + serve + drain + query + exit"
  - "CancellationToken shutdown: token.cancel() → await server → await broker → drop event_tx → await db_writer"
  - "Per-tier SQLite query: detections_by_tier() iterates tiers 1-3, excludes KnownCrawler, returns [u32;3]"

requirements-completed: [TEST-01, TEST-02]

# Metrics
duration: 30min
completed: 2026-03-30
---

# Phase 05 Plan 02: test-agent Subcommand Core Summary

**Ephemeral generate-serve-wait-score pipeline via CancellationToken timeout, pre-bound TcpListener, and per-tier SQLite scorecard query**

## Performance

- **Duration:** ~30 min
- **Started:** 2026-03-30T12:10:00Z
- **Completed:** 2026-03-30T12:40:53Z
- **Tasks:** 3 of 3
- **Files modified:** 7 (Cargo.toml, cli/mod.rs, store/mod.rs, lib.rs, main.rs, test_agent/mod.rs, tests/test_test_agent.rs)

## Accomplishments

- Wired `honeyprompt test-agent` subcommand with --listen, --timeout, --format flags via Clap derive
- Created `src/test_agent/mod.rs` that orchestrates TempDir → generate → bind → serve → wait → drain → Scorecard
- Implemented `detections_by_tier()` store query returning `[u32; 3]` per-tier detection counts (excludes KnownCrawler)
- Added `OutputFormat` enum and `Scorecard` struct for Plan 03 scorecard rendering
- All 4 integration tests pass (TEST-01 lifecycle + TEST-02 flag parsing, ~5s with 1-2s timeouts)

## Task Commits

1. **Task 1: Add dependencies, CLI args, and store query** - `4de74aa` (feat)
2. **Task 2: Create test_agent module with ephemeral server lifecycle** - pending (see note)
3. **Task 3: Create integration test stubs for TEST-01 and TEST-02** - pending (see note)

**Note:** Git access was blocked by sandbox during parallel agent execution after Task 1. Tasks 2 and 3 source files are written to disk and verified passing via `cargo test --workspace` and `cargo test --test test_test_agent`. The orchestrator's final hook pass should stage and commit these files:
- `src/test_agent/mod.rs` (new)
- `src/lib.rs` (added pub mod test_agent)
- `src/main.rs` (Commands::TestAgent dispatch arm)
- `tests/test_test_agent.rs` (new integration tests)
- `.planning/phases/05-test-agent-subcommand/05-02-SUMMARY.md` (this file)
- `.planning/STATE.md`
- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`

## Files Created/Modified

- `Cargo.toml` - Added tokio-util and tempfile to [dependencies]
- `src/cli/mod.rs` - OutputFormat enum, TestAgent variant in Commands, TestAgentArgs struct
- `src/store/mod.rs` - detections_by_tier() function + test_detections_by_tier unit test
- `src/lib.rs` - pub mod test_agent declaration
- `src/main.rs` - Commands::TestAgent dispatch arm calling test_agent::run()
- `src/test_agent/mod.rs` - Scorecard struct, run() sync orchestrator, run_async() async lifecycle
- `tests/test_test_agent.rs` - Integration tests: lifecycle_clean_shutdown, timeout_flag, listen_flag, format_json

## Decisions Made

- Pre-bind with `std::net::TcpListener::bind()` before the async runtime, then pass to `tokio::net::TcpListener::from_std()` — eliminates the port-stealing race condition where binding and listening happen in separate steps
- Await `broker_handle` before `drop(event_tx)` so the broker can forward all queued callback events to db_writer before the broadcast channel closes
- No `stdout_logger_task` spawned in test-agent — the server is quiet during collection per D-06; only startup/shutdown messages go to stderr
- `Scorecard` struct with `tiers: [bool; 3]` and `tier_counts: [u32; 3]` carries both boolean pass/fail and raw counts for Plan 03 rendering

## Deviations from Plan

None — plan executed exactly as written. All patterns from the research doc were applied. The port-binding and drain ordering in the plan were followed precisely.

## Issues Encountered

- Git commit access was blocked by the sandbox during parallel agent execution for Tasks 2 and 3. Task 1 committed successfully (`4de74aa`). Tasks 2 and 3 files are written and verified (all tests green) but not committed. The orchestrator must complete these commits.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `honeyprompt test-agent --timeout N` is fully functional: starts server, waits N seconds, exits 0 (no callbacks) or 1 (triggered)
- `Scorecard` struct is ready for Plan 03 rendering (text + JSON output via OutputFormat enum)
- `detections_by_tier()` returns `[u32; 3]` for Plan 03 scorecard display
- Integration tests in `tests/test_test_agent.rs` have JSON assertions commented out — Plan 03 should uncomment after wiring stdout render

## Self-Check: PASSED

All files verified on disk:
- FOUND: `src/test_agent/mod.rs`
- FOUND: `tests/test_test_agent.rs`
- FOUND: `src/cli/mod.rs` contains OutputFormat (3 occurrences), TestAgent (2 occurrences)
- FOUND: `src/store/mod.rs` contains detections_by_tier (3 occurrences)
- FOUND: `src/test_agent/mod.rs` uses build_router (not server::serve)
- FOUND: `src/test_agent/mod.rs` uses tokio::net::TcpListener::from_std()
- FOUND: `src/test_agent/mod.rs` uses CancellationToken (5 occurrences)
- FOUND: `src/main.rs` uses std::process::exit(scorecard.exit_code()) and exit(2)
- All 4 integration tests pass (cargo test --test test_test_agent in 5.05s)
- Task 1 commit hash `4de74aa` exists in git log

Git commits for Tasks 2+3 are pending due to sandbox restriction during parallel execution.

---
*Phase: 05-test-agent-subcommand*
*Completed: 2026-03-30*
