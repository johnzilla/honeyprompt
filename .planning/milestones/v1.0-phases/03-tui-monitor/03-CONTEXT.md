# Phase 3: TUI Monitor - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Live terminal UI (`honeyprompt monitor`) for watching callback events arrive in real time via Ratatui. `top`-style layout with stats header and scrollable event table. Vim-style keyboard controls, filtering/sorting, and replay event management. Supports integrated mode (starts server + TUI) and attach mode (connects to running server). This is the flagship demo experience — screenshot-worthy.

</domain>

<decisions>
## Implementation Decisions

### Layout and information density
- **D-01:** `top`-style layout: stats header bar at top with live counters (total detections, unique sessions, per-tier counts, replay count), scrollable event table below.
- **D-02:** Event table shows all 9 AppEvent fields as columns: timestamp, tier, classification, source IP, user-agent (truncated), session ID, nonce, fire count, replay flag.

### Filtering and sorting UX
- **D-03:** Vim-style keyboard controls: `j/k` to scroll, `tab` to cycle filters, `:` for commands.
- **D-04:** Filtering and sorting by tier, time, and source per success criteria TUI-02.

### Visual treatment of replays
- **D-05:** Replay events hidden from the table by default. A key toggles replay visibility.
- **D-06:** Header stats area shows a replay counter (e.g., "N replays hidden") so users know replay events exist and can toggle to view them.
- **D-07:** Detection counts in the UI exclude replay events per success criteria SC-3.

### Monitor invocation model
- **D-08:** Integrated mode is the default: `honeyprompt monitor <project-dir>` starts the server and TUI in one process. Zero-friction for demos.
- **D-09:** Attach mode available via flag (e.g., `--attach`): connects to a separately running `honeyprompt serve` instance. For production/headless setups.

### Claude's Discretion
- Ratatui widget selection and layout grid specifics
- Exact key bindings beyond j/k/tab/: (e.g., which key toggles replays, sort cycling)
- Stats header content and formatting details
- Color scheme and visual styling
- Attach mode connection mechanism (shared DB polling, IPC, etc.)
- `:` command vocabulary

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project context
- `.planning/PROJECT.md` — Core value, safety model, "TUI as flagship experience" decision, Ratatui constraint
- `.planning/REQUIREMENTS.md` — v1 requirements CLI-04, TUI-01, TUI-02

### Phase 2 code (event pipeline)
- `src/broker/mod.rs` — Broadcast architecture: TUI subscribes to `broadcast::Receiver<AppEvent>` same as db_writer and stdout_logger
- `src/types.rs` — `AppEvent` struct with all 9 fields (tier, classification, session_id, source_ip, user_agent, nonce, fire_count, is_replay, received_at)
- `src/server/mod.rs` — `serve()` and `build_router()` functions, `AppState`, startup/shutdown flow
- `src/store/mod.rs` — `count_detections()` for detection stats, `lookup_nonce()` for nonce metadata

### Research findings
- `.planning/research/ARCHITECTURE.md` — Component build layers, async patterns
- `.planning/research/SUMMARY.md` — Stack recommendations

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/broker/mod.rs`: Broadcast channel architecture — TUI subscribes as a new consumer alongside db_writer and stdout_logger
- `src/types.rs`: `AppEvent` already has all fields needed for table rows
- `src/store/mod.rs`: `count_detections()` provides the detection count for header stats
- `src/server/mod.rs`: `serve()` function can be called from monitor's integrated mode

### Established Patterns
- tokio broadcast channel for fan-out to independent consumers
- rust-embed for binary-embedded assets
- Clap subcommands with args structs (`ServeArgs` pattern)

### Integration Points
- `src/main.rs` needs `Monitor(MonitorArgs)` subcommand dispatch
- `src/broker/mod.rs` — TUI task subscribes via `event_tx.subscribe()` same as existing consumers
- `src/server/mod.rs` — integrated mode calls `serve()` internals alongside TUI event loop
- `Cargo.toml` needs `ratatui` and `crossterm` dependencies

</code_context>

<specifics>
## Specific Ideas

- Modeled after `top` — dense, functional, stats-driven header with process-table-style event rows
- Screenshot-worthy: this is the demo image, the conference talk slide, the README hero
- Vim-style controls match the security researcher audience expectation

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-tui-monitor*
*Context gathered: 2026-03-29*
