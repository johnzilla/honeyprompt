---
phase: 02-server-and-detection
plan: "02"
subsystem: event-pipeline
tags: [store, broker, sqlite, tokio, async, replay-detection, session-tracking]
dependency_graph:
  requires: ["02-01"]
  provides: ["02-03"]
  affects: ["src/store/mod.rs", "src/broker/mod.rs", "src/lib.rs"]
tech_stack:
  added: ["tokio-rusqlite (async SQLite calls)", "tokio broadcast channel (fan-out)", "tokio mpsc channel (broker input)"]
  patterns: ["INSERT ON CONFLICT upsert for replay detection", "broadcast fan-out to independent consumers", "session_id = SHA-256(ip+ua) 16-char hex"]
key_files:
  created: ["src/broker/mod.rs"]
  modified: ["src/store/mod.rs", "src/lib.rs", "Cargo.toml"]
decisions:
  - "classification stored in extra_headers JSON blob as {classification, headers} — avoids schema migration since no classification column exists"
  - "broker broadcasts AppEvent with is_replay=false/fire_count=1 as initial values — DB writer gets authoritative values from insert_callback_event return"
  - "tokio_rusqlite::Error::from() used for rusqlite-to-tokio-rusqlite error conversion (Error::Rusqlite variant does not exist in 0.7)"
metrics:
  duration_seconds: 163
  completed_date: "2026-03-29"
  tasks_completed: 2
  files_modified: 4
---

# Phase 02 Plan 02: Event Pipeline (Store + Broker) Summary

**One-liner:** Async event pipeline with upsert/replay store functions and broadcast broker fan-out routing RawCallbackEvent through session enrichment to DB writer and stdout logger.

## Tasks Completed

| Task | Description | Commit | Files |
|------|-------------|--------|-------|
| 1 | Add async event insertion and session-based detection counting to store | 2aae870 | src/store/mod.rs |
| 2 | Implement event broker with mpsc receive, session enrichment, and broadcast fan-out | 181b933 | src/broker/mod.rs, src/lib.rs, Cargo.toml |

## What Was Built

### Store Extensions (src/store/mod.rs)

Three new public functions added alongside existing `insert_nonce`:

- **`insert_callback_event`**: Upserts a callback event using `INSERT ... ON CONFLICT(nonce) DO UPDATE`. First fire creates row with fire_count=1, is_replay=0. Subsequent fires increment fire_count and set is_replay=1. Returns `(fire_count, is_replay)` so broker knows replay status. SRV-07 enforced: no body parameter.

- **`lookup_nonce`**: Queries nonce_map for `(tier, payload_id, embedding_loc)`. Returns `Option<(u8, String, String)>` — None for unknown nonces. Used by Plan 03 callback handler for nonce validation.

- **`count_detections`**: Counts unique `(session_id || '-' || tier)` pairs where extra_headers does NOT contain `"classification":"KnownCrawler"`. Implements D-08 per-session per-tier detection model.

### Event Broker (src/broker/mod.rs)

Three async tasks implementing the D-01 broadcast architecture:

- **`broker_task`**: Receives `RawCallbackEvent` from mpsc, computes `session_id = compute_session_id(ip, ua)`, constructs `AppEvent`, broadcasts to all subscribers.

- **`db_writer_task`**: Subscribes to broadcast, calls `insert_callback_event` via tokio-rusqlite `.call()` for each event. Handles `Lagged` (drops warning to stderr) and `Closed` (exits cleanly).

- **`stdout_logger_task`**: Subscribes to broadcast, formats log lines. Text mode: `{timestamp} tier={tier} class={classification} ip={ip} ua="{ua_60chars}"`. JSON mode: structured JSON line with all event fields.

### Supporting Helpers

- **`build_extra_headers`**: Serializes `AgentClass` + headers HashMap to JSON: `{"classification":"...", "headers":{...}}`. Used by db_writer_task to produce the `extra_headers` column value.
- **`classify_label`**: Converts `AgentClass` enum to stable string (`KnownCrawler:OpenAI`, `KnownAgent:Anthropic`, `Unknown`).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] tokio_rusqlite::Error::Rusqlite variant does not exist**
- **Found during:** Task 2 first compile
- **Issue:** Plan referenced `tokio_rusqlite::Error::Rusqlite(e)` but tokio-rusqlite 0.7 uses `Error::Error(rusqlite_error)` with an `impl From<rusqlite::Error>`
- **Fix:** Used `tokio_rusqlite::Error::from(e)` via `.map_err(tokio_rusqlite::Error::from)` which uses the existing From impl
- **Files modified:** src/broker/mod.rs
- **Commit:** 181b933 (included in task commit)

None of the other plan guidance required deviation — upsert logic, session enrichment, broadcast fan-out, and lag handling all implemented exactly as specified.

## Known Stubs

None. All functions are fully implemented and wired to real data sources.

## Test Coverage

| Module | Tests Added | Total Tests |
|--------|-------------|-------------|
| store | 9 new | 12 total |
| broker | 3 new | 3 total |
| Full suite | - | 60 total (0 failures) |

## Self-Check: PASSED

- [x] src/store/mod.rs — insert_callback_event, lookup_nonce, count_detections all present
- [x] src/broker/mod.rs — broker_task, db_writer_task, stdout_logger_task all present
- [x] src/lib.rs — `pub mod broker` added
- [x] Commit 2aae870 exists (store functions)
- [x] Commit 181b933 exists (broker module)
- [x] `cargo test` passes: 60 tests, 0 failures
- [x] No body parameter in insert_callback_event (SRV-07)
- [x] ON CONFLICT upsert present (replay detection)
- [x] compute_session_id called in broker_task (session enrichment)
- [x] Lagged handling in both db_writer_task and stdout_logger_task
