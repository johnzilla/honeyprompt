---
phase: 09-server-side-identity-stats
plan: "02"
subsystem: server, store
tags: [stats, cors, json-api, integration-tests]
dependency_graph:
  requires: [09-01]
  provides: [/stats endpoint, ReportSummary serialization]
  affects: [src/server/mod.rs, src/store/mod.rs, tests/test_serve.rs]
tech_stack:
  added: []
  patterns: [axum-json-handler, tokio-rusqlite-in-appstate, cors-header-injection]
key_files:
  created: []
  modified:
    - src/store/mod.rs
    - src/server/mod.rs
    - src/monitor/mod.rs
    - src/test_agent/mod.rs
    - tests/test_serve.rs
decisions:
  - "Clone tokio-rusqlite Connection into AppState.conn so stats_handler can query DB without separate connection"
  - "Use axum::http::HeaderValue for CORS header — axum re-exports http types, no separate http crate import needed"
  - "stats_handler returns 500 on DB error, 200+JSON on success — no partial response"
metrics:
  duration: ~15 min
  completed_date: "2026-04-01"
  tasks_completed: 2
  files_changed: 5
---

# Phase 09 Plan 02: /stats Endpoint Summary

One-liner: Public `/stats` JSON endpoint returning aggregate ReportSummary with CORS header and all-zero empty-DB semantics, backed by tokio-rusqlite Connection in AppState.

## What Was Built

Added a `GET /stats` HTTP endpoint to the honeyprompt server that returns aggregate callback statistics as JSON, with an `Access-Control-Allow-Origin: *` header for cross-origin access from the forthcoming honeyprompt.dev landing page.

### Key changes

**`src/store/mod.rs`**
- Added `#[derive(serde::Serialize)]` to `ReportSummary` struct so axum::Json can serialize it
- Added `test_query_report_summary_empty_db` unit test confirming all-zero returns on empty DB

**`src/server/mod.rs`**
- Added `conn: tokio_rusqlite::Connection` field to `AppState`
- Added `use axum::response::IntoResponse` import
- Added `stats_handler` async function: calls `store::query_report_summary` via `conn.call()`, wraps result in `axum::Json`, injects CORS header, returns 500 on DB error
- Added `.route("/stats", get(stats_handler))` to `build_router`
- Updated `serve()` AppState construction to include `conn: conn.clone()`

**`src/monitor/mod.rs` and `src/test_agent/mod.rs`** (Rule 3 auto-fix)
- Updated AppState construction in both files to include `conn: conn.clone()` — required to resolve compile errors caused by the new mandatory `conn` field

**`tests/test_serve.rs`**
- Converted `build_test_state` from sync to async, opening a `tokio_rusqlite::Connection` to the temp DB
- Updated all 5 existing test calls to `.await` the new async helper
- Added 3 new integration tests: `test_stats_empty_db_returns_json`, `test_stats_populated_db_returns_counts`, `test_stats_has_cors_header`

## Verification

- `cargo check` passes: no errors
- `cargo clippy` passes: no warnings
- `cargo test` passes: all 8 serve integration tests + 4 test-agent tests pass
- `/stats` returns 200 JSON with all ReportSummary fields (total_sessions, detection_sessions, crawler_sessions, tier1_sessions, tier2_sessions, tier3_sessions, earliest_event, latest_event)
- `/stats` returns `Access-Control-Allow-Origin: *` header
- `/stats` on empty DB returns all-zero counts (not 500)

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 | 6baaecd | feat(09-02): add /stats endpoint with CORS header and Serialize derive |
| Task 2 | c422779 | test(09-02): add /stats integration tests and update AppState in test helpers |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed missing `conn` in AppState construction in monitor and test_agent**
- **Found during:** Task 1 — cargo check after updating AppState struct
- **Issue:** `src/monitor/mod.rs:932` and `src/test_agent/mod.rs:232` both construct AppState and were missing the new `conn` field
- **Fix:** Added `conn: conn.clone()` to both sites — `conn` was already in scope at both locations
- **Files modified:** src/monitor/mod.rs, src/test_agent/mod.rs
- **Commit:** 6baaecd

## Known Stubs

None — all fields in the /stats response are wired to live DB queries via `query_report_summary`.

## Self-Check: PASSED
