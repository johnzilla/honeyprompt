---
phase: 02-server-and-detection
plan: 03
subsystem: api
tags: [axum, tokio, tower-http, rust, http-server, sqlite, fingerprinting]

# Dependency graph
requires:
  - phase: 02-server-and-detection-01
    provides: fingerprint::extract, AgentFingerprint, AgentClass, RawCallbackEvent, AppEvent types
  - phase: 02-server-and-detection-02
    provides: broker_task, db_writer_task, stdout_logger_task, insert_callback_event, lookup_nonce, count_detections
provides:
  - Axum HTTP server serving static honeypot files and /cb/v1/{nonce} callback endpoint
  - serve() async function wiring full event pipeline
  - build_router() for testable in-process router construction
  - Serve CLI subcommand with --json flag
  - Integration tests for callback (204 invariant), static serving, and event pipeline
affects: [03-tui-monitor, cli, server]

# Tech tracking
tech-stack:
  added: [tower (dev, util), http (dev, v1)]
  patterns: [MockConnectInfo layer for in-process axum test routing, build_router() extracted for testability]

key-files:
  created:
    - src/server/mod.rs
    - tests/test_serve.rs
  modified:
    - src/cli/mod.rs
    - src/main.rs
    - src/lib.rs
    - Cargo.toml

key-decisions:
  - "MockConnectInfo layer used in tests to satisfy ConnectInfo extractor without binding a port"
  - "build_router() extracted as pub fn to enable in-process oneshot testing via tower::ServiceExt"
  - "try_send() used in callback_handler for non-blocking mpsc delivery (best-effort)"
  - "count_detections return type needs explicit error mapping via tokio_rusqlite::Error::from"

patterns-established:
  - "Pattern: axum in-process test with MockConnectInfo layer + tower::ServiceExt::oneshot"
  - "Pattern: build_router() extracted from serve() for testability"

requirements-completed: [CLI-03, SRV-01]

# Metrics
duration: 3min
completed: 2026-03-29
---

# Phase 02 Plan 03: Server and Detection (Capstone) Summary

**Axum HTTP server on single port serving static honeypot pages and /cb/v1/{nonce} callback beacons with 204-always handler, full event pipeline, and graceful shutdown**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-29T03:41:08Z
- **Completed:** 2026-03-29T03:43:53Z
- **Tasks:** 2 of 3 complete (Task 3 is human-verify checkpoint)
- **Files modified:** 6

## Accomplishments
- Implemented `src/server/mod.rs` with Axum router, callback handler always returning 204 (D-03), and static file serving via ServeDir fallback
- Added `Serve(ServeArgs)` CLI subcommand with `--json` flag and dispatch in `main.rs`
- Wired full event pipeline: callback_handler -> mpsc -> broker_task -> broadcast -> db_writer_task + stdout_logger_task
- Startup output shows bind address, nonce count, DB path, "ready" (D-09)
- Graceful shutdown via Ctrl+C with detection count summary (D-11)
- 5 integration tests covering: valid/invalid/unknown nonce returning 204, static file GET /, and event channel delivery

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement Axum server module with callback handler and serve CLI subcommand** - `465d9e4` (feat)
2. **Task 2: Integration test for serve command** - `87e7f88` (test)

## Files Created/Modified
- `src/server/mod.rs` - Axum router, AppState, NonceMeta, callback_handler, serve(), build_router(), shutdown_signal()
- `src/cli/mod.rs` - Added Serve(ServeArgs) variant and ServeArgs struct with path/json fields
- `src/main.rs` - Added Commands::Serve dispatch using tokio::runtime::Runtime::new()
- `src/lib.rs` - Added pub mod server
- `tests/test_serve.rs` - 5 integration tests using tower::ServiceExt::oneshot + MockConnectInfo
- `Cargo.toml` - Added tower (dev) and http (dev) to dev-dependencies

## Decisions Made
- `build_router()` extracted as a `pub fn` so integration tests can build the router without binding a TCP port, using tower::ServiceExt::oneshot for in-process testing
- `MockConnectInfo` Axum layer applied in test helper to satisfy `ConnectInfo<SocketAddr>` extractor without a real socket
- `try_send()` in callback_handler for non-blocking delivery (drops if channel full — best-effort per design)
- Fixed type inference error on `conn.call(|c| Ok(count_detections(c)?))` — required explicit `map_err(tokio_rusqlite::Error::from)` to satisfy E: Send bound

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed type annotation error in count_detections conn.call() closure**
- **Found during:** Task 1 (cargo build verification)
- **Issue:** `conn.call(|c| Ok(crate::store::count_detections(c)?))` produced E0283 type inference error on the Result `E` parameter — tokio-rusqlite's `call()` requires `E: Send + 'static`
- **Fix:** Changed to `conn.call(|c| crate::store::count_detections(c).map_err(tokio_rusqlite::Error::from))`
- **Files modified:** src/server/mod.rs
- **Verification:** cargo build succeeds
- **Committed in:** 465d9e4 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Single compile-time type annotation fix. No scope creep.

## Issues Encountered
None beyond the type annotation fix above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `honeyprompt serve` command is fully implemented and tested
- Static pages served on same port as callback endpoint (SRV-01 complete)
- Callback always returns 204 (D-03 enforced)
- Event pipeline wired: fingerprint -> classify -> broker -> DB + stdout
- Waiting for human verification (Task 3 checkpoint) to confirm end-to-end UX
- Phase 3 (TUI monitor) can begin after checkpoint approval

## Known Stubs
None - all functionality is wired. The server loads real callback-map.json at startup, opens the real SQLite DB, and serves real generated output files.

---
*Phase: 02-server-and-detection*
*Completed: 2026-03-29*
