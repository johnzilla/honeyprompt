---
phase: 03-tui-monitor
plan: 02
subsystem: ui
tags: [ratatui, crossterm, tui, tokio, axum, sqlite, broadcast, monitor]

# Dependency graph
requires:
  - phase: 03-tui-monitor plan 01
    provides: AppState struct with filter/sort/replay logic, MonitorArgs CLI definition, unit tests
  - phase: 02-server-and-detection
    provides: build_router(), broker_task(), db_writer_task(), AppEvent broadcast pipeline

provides:
  - Full TUI monitor command: real-time event table, 4-panel layout (stats/filter/table/keys)
  - Integrated mode: server + TUI in one process via shared broadcast channel
  - Attach mode: read-only DB polling for running sessions
  - Keyboard controls: j/k scroll, Tab filter, s sort, r replay toggle, : command, ? help, q quit
  - Terminal panic safety: panic hook restores raw mode and alternate screen
  - CLI-04 integration tests: --help, missing project dir, attach with missing DB

affects:
  - 04-reporting (consumes AppEvent and DB schema)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "tokio::select! with EventStream::new() + broadcast::Receiver + interval tick for async TUI loop"
    - "Panic hook pattern: take_hook + set_hook wrapping disable_raw_mode + LeaveAlternateScreen"
    - "Terminal setup before run_loop, restore after via result-preserving let result = ...; restore; result"
    - "render_stateful_widget for scrollable Table with TableState"
    - "Layout::vertical with Constraint::Fill(1) for flexible center panel"

key-files:
  created:
    - src/monitor/mod.rs (TUI rendering, key handling, integrated mode, attach mode, monitor entry point)
    - tests/test_monitor.rs (CLI-04 integration tests)
  modified:
    - src/main.rs (Commands::Monitor wired to monitor::monitor())

key-decisions:
  - "Integrated mode replicates serve() pipeline inline rather than calling serve() — allows TUI to own the broadcast subscriber before any spawn"
  - "Attach mode polls DB every 250ms via tokio::select! interval arm — avoids SQLite LISTEN/NOTIFY complexity"
  - "Terminal setup happens inside the async monitor() function body — avoids raw mode before async runtime (Research Pitfall 5)"
  - "restore_terminal called unconditionally after run_loop regardless of Ok/Err — ensures terminal cleanup on error paths"

patterns-established:
  - "TUI entry: setup_terminal() → run_loop() → restore_terminal() with result preservation"
  - "broadcast::Receiver subscribed before tokio::spawn of producers to avoid missed events"

requirements-completed: [CLI-04, TUI-01, TUI-02]

# Metrics
duration: ~45min
completed: 2026-03-29
---

# Phase 03 Plan 02: TUI Monitor Summary

**Ratatui-based real-time event monitor with 4-panel layout, integrated Axum server mode, DB attach mode, and terminal panic safety**

## Performance

- **Duration:** ~45 min
- **Started:** 2026-03-29
- **Completed:** 2026-03-29
- **Tasks:** 3 (including human-verify checkpoint)
- **Files modified:** 3

## Accomplishments

- Implemented the full TUI render loop in `src/monitor/mod.rs`: 4-panel layout (stats header, filter bar, event table, key hint bar), minimum terminal size guard (80x20), empty-state messaging, help overlay
- Built integrated mode that starts the Axum server and TUI in the same process, sharing a broadcast channel — callbacks arrive at the HTTP handler and appear on screen without any external process coordination
- Added attach mode polling existing SQLite DB every 250ms for live viewing of sessions started by a separate `honeyprompt serve` instance
- Wired `Commands::Monitor` in `main.rs` with real dispatch (replacing placeholder bail); created `tests/test_monitor.rs` with three CLI-04 integration tests covering --help, missing project dir, and attach with missing DB

## Task Commits

Each task was committed atomically:

1. **Task 1: TUI rendering, key handling, and event loop** - `3d46484` (feat)
2. **Task 2: Wire monitor dispatch in main.rs and add integration test** - `56927a1` (feat)
3. **Task 3: Visual verification checkpoint** - N/A (human-verify, approved)

## Files Created/Modified

- `src/monitor/mod.rs` - Full TUI implementation: setup_terminal, restore_terminal, render, render_event_table, handle_key_event, run_loop, run_loop_attach, monitor entry point, helper functions (format_time, truncate_str, tier_color, class_label)
- `src/main.rs` - Commands::Monitor arm wired to `monitor::monitor(&cfg, path, &args)` with config load
- `tests/test_monitor.rs` - Three integration tests for CLI-04 error paths

## Decisions Made

- **Integrated mode replicates serve() pipeline inline** rather than calling serve() — the TUI must subscribe to the broadcast channel before producers are spawned, which requires owning the setup sequence
- **Attach mode uses 250ms poll interval** rather than inotify or SQLite triggers — simpler, cross-platform, acceptable latency for a security demo tool
- **Terminal setup inside async function** — per Research Pitfall 5, raw mode must not be enabled before the async runtime is running; setup_terminal() is called inside monitor() after the runtime is active
- **restore_terminal called unconditionally** — `let result = run_loop(...).await; restore_terminal(&mut terminal)?; let _ = shutdown_tx.send(()); result` — this ensures cleanup even if the event loop returns an error

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None — implementation followed the plan's detailed specification. All acceptance criteria met on first pass. Visual checkpoint approved by user.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 03 is complete. The flagship demo experience is working: `honeyprompt monitor <project-dir>` starts server and TUI together, shows live callback events, supports filtering/sorting/replay toggle, and restores the terminal cleanly.
- Phase 04 (reporting) can consume AppEvent and the SQLite DB schema — both are stable and documented.
- No blockers.

---
*Phase: 03-tui-monitor*
*Completed: 2026-03-29*
