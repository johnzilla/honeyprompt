# Phase 03: TUI Monitor - Research

**Researched:** 2026-03-29
**Domain:** Ratatui terminal UI, tokio async event loop, crossterm input handling
**Confidence:** HIGH

## Summary

Phase 3 builds the `honeyprompt monitor` subcommand — a live Ratatui TUI that consumes events from the existing tokio broadcast channel pipeline (Phase 2) and renders them in a `top`-style layout. The UI spec is fully locked in `03-UI-SPEC.md`: 4-section layout (stats header, filter bar, event table, key hint bar), vim-style keybindings, replay-hidden-by-default, integrated+attach modes. This phase is almost entirely additive — no existing modules need structural changes, only extension.

The core architecture adds a new `src/monitor/` module containing a `MonitorArgs` Clap struct, an async `monitor()` entry function, and an app state struct that owns: a `broadcast::Receiver<AppEvent>` subscriber, a `Vec<AppEvent>` event buffer, filter/sort/replay-toggle state, and a `TableState` for the scrollable table. The render loop uses `tokio::select!` to multiplex three arms: `event_rx.recv()` (new AppEvents from broker), `crossterm::event::EventStream` (keyboard input), and a render tick interval.

Ratatui 0.30.0 is the current version on crates.io (verified 2026-03-29). Its `Table` + `TableState` API is the right widget for the scrollable event table; `Block` + `Paragraph` covers the stats/filter/hint panels. Crossterm 0.29.0 is the current backend version; its `event-stream` feature (plus the `futures` crate) enables async key event reading. Both must be added to `Cargo.toml`.

**Primary recommendation:** Add `ratatui = "0.30"` and `crossterm = { version = "0.29", features = ["event-stream"] }` to Cargo.toml. Implement `src/monitor/mod.rs` with an immediate-mode render loop that subscribes to the broker broadcast channel as a new consumer, stores received events in a `Vec<AppEvent>`, and renders on a ~16ms tick.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** `top`-style layout: stats header bar at top with live counters (total detections, unique sessions, per-tier counts, replay count), scrollable event table below.
- **D-02:** Event table shows all 9 AppEvent fields as columns: timestamp, tier, classification, source IP, user-agent (truncated), session ID, nonce, fire count, replay flag.
- **D-03:** Vim-style keyboard controls: `j/k` to scroll, `tab` to cycle filters, `:` for commands.
- **D-04:** Filtering and sorting by tier, time, and source per success criteria TUI-02.
- **D-05:** Replay events hidden from the table by default. A key toggles replay visibility.
- **D-06:** Header stats area shows a replay counter (e.g., "N replays hidden") so users know replay events exist and can toggle to view them.
- **D-07:** Detection counts in the UI exclude replay events per success criteria SC-3.
- **D-08:** Integrated mode is the default: `honeyprompt monitor <project-dir>` starts the server and TUI in one process. Zero-friction for demos.
- **D-09:** Attach mode available via flag (e.g., `--attach`): connects to a separately running `honeyprompt serve` instance. For production/headless setups.

### Claude's Discretion
- Ratatui widget selection and layout grid specifics
- Exact key bindings beyond j/k/tab/: (e.g., which key toggles replays, sort cycling)
- Stats header content and formatting details
- Color scheme and visual styling
- Attach mode connection mechanism (shared DB polling, IPC, etc.)
- `:` command vocabulary

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CLI-04 | User can run `honeyprompt monitor` to view live TUI event display | Clap `Monitor(MonitorArgs)` variant added to `Commands` enum; `src/cli/mod.rs` extended; `main.rs` dispatch added |
| TUI-01 | Live event table displays callbacks in real time via Ratatui | `broadcast::Receiver<AppEvent>` consumed in `tokio::select!` loop; `Vec<AppEvent>` buffer rendered via Ratatui `Table` + `TableState` |
| TUI-02 | Events filterable and sortable by tier, time, and source | In-memory filter and sort state; applied to buffer before building Table rows each frame |
</phase_requirements>

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ratatui | 0.30.0 | Terminal UI widget framework | Project constraint (CLAUDE.md); current crates.io version verified 2026-03-29 |
| crossterm | 0.29.0 | Terminal backend + async key events | De facto crossterm backend for ratatui; `event-stream` feature enables async input |
| futures | 0.3 | `StreamExt` for `crossterm::event::EventStream` | Required by crossterm event-stream to poll as async stream in `tokio::select!` |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio (already present) | 1 | Async runtime, broadcast channel consumer | Already in Cargo.toml; no new dep needed |
| clap (already present) | 4.6 | `MonitorArgs` struct | Already in Cargo.toml; no new dep needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ratatui | tui-rs (archived) | tui-rs is unmaintained; ratatui is the active fork — never use tui-rs |
| crossterm backend | termion | crossterm supports Windows (important for future PLAT-01); termion is Unix-only |
| crossterm event-stream | polling crossterm::event::poll() in std thread | Async stream integrates cleanly with tokio::select! alongside broadcast recv |

**Installation:**
```bash
cargo add ratatui@0.30
cargo add crossterm@0.29 --features event-stream
cargo add futures
```

Or add directly to Cargo.toml:
```toml
ratatui = "0.30"
crossterm = { version = "0.29", features = ["event-stream"] }
futures = "0.3"
```

**Version verification:** ratatui 0.30.0 and crossterm 0.29.0 confirmed against crates.io on 2026-03-29.

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── monitor/
│   └── mod.rs         # MonitorArgs, monitor() entry fn, AppStateMonitor, render/event loop
├── cli/mod.rs         # Add Monitor(MonitorArgs) variant to Commands enum
└── main.rs            # Add Commands::Monitor dispatch to block_on(monitor(...))
```

The monitor module is self-contained. It uses types from `crate::types`, `crate::server`, `crate::store`, and `crate::broker` but adds no circular dependencies.

### Pattern 1: Ratatui Immediate-Mode Render Loop with Tokio

**What:** The TUI renders from scratch every frame (~60fps, 16ms tick). App state is stored in a struct; each frame rebuilds the widget tree from that state. Input and app events arrive via `tokio::select!`.

**When to use:** Standard pattern for all ratatui TUI applications. Ratatui is an immediate-mode library — widgets are not stateful objects; only `TableState` and similar `StatefulWidget` companions carry inter-frame state.

**Example (source: [Ratatui async event stream docs](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/)):**
```rust
// Cargo.toml: crossterm = { version = "0.29", features = ["event-stream"] }
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::prelude::*;

pub async fn monitor(args: &MonitorArgs, project_path: &Path) -> anyhow::Result<()> {
    // Terminal setup
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Subscribe to broadcast BEFORE starting server
    let event_rx = event_tx.subscribe();

    let result = run_app(&mut terminal, event_rx, args).await;

    // Cleanup (MUST run even on error/panic)
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, mut event_rx: broadcast::Receiver<AppEvent>, args: &MonitorArgs) -> anyhow::Result<()> {
    let mut app = AppState::new();
    let mut key_events = EventStream::new();
    let mut render_tick = tokio::time::interval(Duration::from_millis(16));

    loop {
        tokio::select! {
            // Render tick
            _ = render_tick.tick() => {
                terminal.draw(|f| render(f, &mut app))?;
            }
            // New AppEvent from broker broadcast
            result = event_rx.recv() => {
                match result {
                    Ok(event) => app.push_event(event),
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("warning: tui lagged, dropped {} events", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // Keyboard input
            Some(Ok(crossterm_event)) = key_events.next() => {
                if app.handle_event(crossterm_event) { break; }
            }
        }
    }
    Ok(())
}
```

### Pattern 2: App State Struct

**What:** A single `AppState` struct owns all inter-frame state. The render function takes `&mut AppState` and produces the widget tree. No widget objects are stored between frames.

**Example:**
```rust
pub struct AppState {
    // Event buffer — all received events, unfiltered
    pub events: Vec<AppEvent>,
    // Filter/sort/display settings
    pub filter: TierFilter,       // All | T1 | T2 | T3
    pub sort: SortField,          // Time | Tier | Source
    pub show_replays: bool,       // D-05: false by default
    // Table scroll state (persisted across frames)
    pub table_state: TableState,
    pub at_bottom: bool,          // auto-scroll behavior
    pub new_events_count: usize,  // "N new events" indicator
    // Mode
    pub mode: UiMode,             // Normal | Command | Help
    pub command_input: String,    // `:` command buffer
    pub command_error: Option<(String, std::time::Instant)>, // 2-second error display
}

#[derive(Clone, Copy, PartialEq)]
pub enum TierFilter { All, T1, T2, T3 }

#[derive(Clone, Copy, PartialEq)]
pub enum SortField { Time, Tier, Source }

#[derive(Clone, Copy, PartialEq)]
pub enum UiMode { Normal, Command, Help }
```

### Pattern 3: Ratatui Table with TableState

**What:** `Table` is a `StatefulWidget`. `TableState` stores the selected row index across frames. `render_stateful_widget` is called instead of `render_widget`.

**Source:** [Ratatui Table docs](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Table.html)

**Example:**
```rust
use ratatui::widgets::{Table, Row, Cell, TableState};
use ratatui::layout::Constraint;
use ratatui::style::{Style, Color, Modifier};

fn render_event_table(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let visible_events = app.visible_events(); // filtered + sorted Vec<&AppEvent>

    let header = Row::new(vec!["TIME", "TIER", "CLASS", "SOURCE IP", "UA", "NONCE", "SESS", "FIRES", "REPLAY"])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));

    let rows: Vec<Row> = visible_events.iter().map(|ev| {
        let replay_style = if ev.is_replay {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            Style::default()
        };
        Row::new(vec![
            format_time(ev.received_at),
            format!("T{}", ev.tier),
            classify_label(&ev.classification),
            ev.fingerprint.source_ip.to_string(),
            truncate(&ev.fingerprint.user_agent, remaining_width),
            truncate_nonce(&ev.nonce),
            truncate_nonce(&ev.session_id),
            ev.fire_count.to_string(),
            if ev.is_replay { " [R] ".to_string() } else { "     ".to_string() },
        ]).style(replay_style)
    }).collect();

    let widths = [
        Constraint::Length(12), // TIME
        Constraint::Length(4),  // TIER
        Constraint::Length(12), // CLASS
        Constraint::Length(16), // SOURCE IP
        Constraint::Fill(1),    // UA (fills remaining)
        Constraint::Length(10), // NONCE
        Constraint::Length(10), // SESS
        Constraint::Length(5),  // FIRES
        Constraint::Length(6),  // REPLAY
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(table, area, &mut app.table_state);
}
```

### Pattern 4: Integrated Mode — Server + TUI in One Process

**What:** D-08 requires integrated mode to start the HTTP server and TUI together. The monitor module calls `server::build_router()` internals directly, subscribes to the same broadcast channel, and runs both inside the same tokio runtime.

**Example:**
```rust
pub async fn monitor(args: &MonitorArgs, project_path: &Path) -> anyhow::Result<()> {
    // Wire up the same pipeline as server::serve() ...
    let (callback_tx, callback_rx) = mpsc::channel::<RawCallbackEvent>(256);
    let (event_tx, _guard) = broadcast::channel::<AppEvent>(1024);

    // Subscribe BEFORE spawning broker (no events missed)
    let tui_rx = event_tx.subscribe();
    let db_rx  = event_tx.subscribe();

    tokio::spawn(broker::broker_task(callback_rx, event_tx.clone()));
    tokio::spawn(broker::db_writer_task(db_rx, conn.clone()));
    // Note: stdout_logger_task NOT spawned — TUI is the display

    let app_state = Arc::new(AppState { callback_tx, nonce_map, crawler_catalog });
    let app = build_router(app_state, output_dir);

    // Start server in background
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(shutdown_rx)
            .await
            .ok();
    });

    // Run TUI (blocks until quit)
    run_tui(tui_rx, conn, args).await?;

    // Signal server shutdown
    let _ = shutdown_tx.send(());
    Ok(())
}
```

### Pattern 5: Attach Mode — DB Polling

**What:** D-09 requires `--attach` to connect to a running `honeyprompt serve` instance. Since the broadcast channel is in-process only, attach mode must use the SQLite DB as the shared state source. Poll new rows periodically (e.g., every 250ms).

**How:** In attach mode, open the DB read-only and poll `SELECT * FROM events WHERE id > last_seen_id ORDER BY id ASC`. No broker subscription needed.

**Note (MEDIUM confidence):** This is the simplest approach consistent with the existing schema. The alternative (Unix socket IPC) was listed as Claude's discretion in CONTEXT.md but DB polling avoids a new IPC mechanism and is adequate for a monitor display.

### Pattern 6: Terminal Cleanup on Panic

**What:** If the app panics, the terminal must still be restored to a usable state. Raw mode and alternate screen must be disabled even on panic.

**How:** Install a custom panic hook before entering raw mode:
```rust
let original_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |panic_info| {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
    original_hook(panic_info);
}));
```

### Pattern 7: Layout with `Layout::vertical`

**What:** Ratatui `Layout` splits a `Rect` into chunks for each panel. The row allocations from UI-SPEC are: stats=3, filter=3, table=remaining, hint=1.

**Example:**
```rust
use ratatui::layout::{Layout, Direction, Constraint};

let chunks = Layout::vertical([
    Constraint::Length(3),   // stats header
    Constraint::Length(3),   // filter bar
    Constraint::Fill(1),     // event table (fills remaining)
    Constraint::Length(1),   // key hint bar
]).split(frame.area());
```

### Anti-Patterns to Avoid

- **Creating `TableState` inside the render function:** TableState must live in `AppState` across frames. Re-creating it each render loses scroll position.
- **Forgetting `render_stateful_widget`:** Calling `render_widget` on a `Table` ignores selection. Must use `frame.render_stateful_widget(table, area, &mut app.table_state)`.
- **Blocking inside `tokio::select!`:** All arms must be async. Never call `std::thread::sleep` or synchronous I/O inside the select loop.
- **Missing panic hook:** Raw mode will corrupt the terminal if the app panics without cleanup.
- **Forgetting `features = ["event-stream"]`:** `crossterm::event::EventStream` only exists when this Cargo feature is enabled.
- **`use tui::` imports:** The crate is `ratatui`, not `tui`. Old blog posts use `tui` (the archived predecessor). Any `tui::` import indicates stale guidance.
- **Using `broadcast::Receiver` from the locked `_guard`:** In `server::serve()`, the broadcast sender is created with `let (event_tx, _) = broadcast::channel(1024)` — the `_` immediately drops the initial receiver. Monitor must subscribe BEFORE the `_` receiver is dropped, or call `event_tx.subscribe()` while `event_tx` is still in scope.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Terminal raw mode + alternate screen enter/exit | Custom ANSI escape sequences | `crossterm::terminal::enable_raw_mode()`, `EnterAlternateScreen`, `LeaveAlternateScreen` | Edge cases in different terminals, signal handling, panic cleanup |
| Scrollable table with row selection | Custom offset tracking | `ratatui::widgets::TableState` | Built-in scroll-to-selected-row logic; handles viewport math |
| Async terminal key events | Spawning a std thread with `crossterm::event::read()` blocking | `crossterm::event::EventStream` with `features = ["event-stream"]` | Integrates directly with `tokio::select!`; no thread + channel needed |
| Column width distribution | Manual string truncation and width arithmetic | `ratatui::layout::Constraint::Fill(1)` for UA column | Ratatui handles terminal width allocation; `Fill` takes remaining space |
| Minimum terminal size check | Count terminal columns manually | `frame.area().width < 80` check at render start | `frame.area()` is always the current terminal size; check once per render |

**Key insight:** Ratatui's widget system handles all viewport math. Application code should only compute what data to display, not how to lay it out.

---

## Common Pitfalls

### Pitfall 1: Broadcast Channel — No Subscriber at Spawn Time

**What goes wrong:** `server::serve()` creates `let (event_tx, _) = broadcast::channel(1024)`. The `_` placeholder drops the initial receiver immediately. If `monitor()` calls `event_tx.subscribe()` after `broker_task` is spawned, early events during startup can be missed, but more critically: if `event_tx` is moved into the spawned server task and the monitor subscribes after that move, compilation fails.

**Why it happens:** Integrated mode must subscribe to the broadcast sender before it is moved into the spawned server goroutine.

**How to avoid:** Subscribe (`event_tx.subscribe()`) for all consumers (TUI, DB writer) before any `tokio::spawn` that consumes `event_tx`. Or clone `event_tx` and keep a reference for late subscribers.

**Warning signs:** `error[E0382]: use of moved value: event_tx` at compile time.

### Pitfall 2: Broadcast Lag Under High Event Rate

**What goes wrong:** If the TUI render loop falls behind (slow terminal I/O, blocking render), the broadcast receiver accumulates a lag. The channel capacity is 1024 events. Under sustained high callback rates, the TUI receiver may see `RecvError::Lagged(n)`.

**Why it happens:** Same pattern documented in Phase 2 research for db_writer_task. Broadcast channels drop the oldest events when a slow receiver falls behind.

**How to avoid:** Handle `RecvError::Lagged(n)` gracefully in the TUI event loop — log a warning in the stats header, do not panic. For a demo tool receiving callbacks from 1-2 AI agents, 1024-event capacity is sufficient.

**Warning signs:** TUI shows fewer events than DB has recorded.

### Pitfall 3: Terminal Not Restored on Panic or Ctrl+C

**What goes wrong:** The terminal stays in raw mode / alternate screen after the process exits. The user's terminal is unusable until they run `reset`.

**Why it happens:** Raw mode and alternate screen are process-global state. Normal Rust `Drop` runs on clean exit but not on panic (without a panic hook) or `std::process::exit`.

**How to avoid:** Install a custom panic hook that calls `disable_raw_mode()` + `LeaveAlternateScreen`. Also handle `Ctrl+C` in the key event loop rather than relying on signal behavior.

**Warning signs:** Terminal appears blank/corrupted after `honeyprompt monitor` exits with an error.

### Pitfall 4: `TableState` Row Index Out of Bounds After Filter Change

**What goes wrong:** User is scrolled to row 15. User changes filter from `All` to `T1`, which yields only 3 visible rows. `TableState::selected()` is still `Some(15)`, but the table has 3 rows. Ratatui clamps internally but the visual highlight jumps to the last row unexpectedly.

**Why it happens:** `TableState` selected index is not automatically clamped when the visible row count changes.

**How to avoid:** When filter or sort changes, reset `table_state.select(Some(0))` or clamp to `min(selected, visible_count.saturating_sub(1))`.

### Pitfall 5: Rendering in `main()` Without `#[tokio::main]`

**What goes wrong:** The existing `main.rs` uses `tokio::runtime::Runtime::new()` and `rt.block_on(...)` for the `Serve` command. This pattern works for monitor too, but the terminal setup (enable_raw_mode) must happen inside the runtime's thread, not before it.

**Why it happens:** `crossterm::event::EventStream` requires the tokio reactor to be active when it is created.

**How to avoid:** Call `enable_raw_mode()` inside the `block_on(monitor(...))` closure, not in the `Commands::Monitor` match arm before `rt.block_on`.

### Pitfall 6: UA Column Width Calculation

**What goes wrong:** The UA column width is "fills remaining − 52 columns" per UI-SPEC. If the terminal is narrower than 80 columns, this calculation goes negative.

**Why it happens:** `Constraint::Fill(1)` handles this automatically if used correctly, but manual string truncation based on computed width can underflow.

**How to avoid:** Use `Constraint::Fill(1)` for the UA column in the widths array. Ratatui will never allocate negative width. Add a minimum terminal size guard (render an error Paragraph if `frame.area().width < 80 || frame.area().height < 20`).

---

## Code Examples

### Terminal Setup and Cleanup (verified pattern)
```rust
// Source: https://ratatui.rs/recipes/apps/terminal-and-event-handler/
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
```

### Async Select Loop (verified pattern)
```rust
// Source: https://ratatui.rs/tutorials/counter-async-app/async-event-stream/
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::StreamExt;
use tokio::sync::broadcast;

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut event_rx: broadcast::Receiver<AppEvent>,
    mut app: AppState,
) -> anyhow::Result<()> {
    let mut key_stream = EventStream::new();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(16));

    loop {
        tokio::select! {
            _ = tick.tick() => {
                terminal.draw(|f| render(f, &mut app))?;
            }
            result = event_rx.recv() => {
                match result {
                    Ok(ev) => app.push_event(ev),
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("tui: dropped {} events (lag)", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            Some(Ok(crossterm_event)) = key_stream.next() => {
                if handle_key(&crossterm_event, &mut app) {
                    break; // quit
                }
            }
        }
    }
    Ok(())
}
```

### Table with TableState (verified pattern)
```rust
// Source: https://docs.rs/ratatui/latest/ratatui/widgets/struct.Table.html
use ratatui::widgets::{Block, Borders, Row, Cell, Table};
use ratatui::layout::Constraint;
use ratatui::style::{Style, Modifier};

fn render_table(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let visible: Vec<&AppEvent> = app.visible_events();

    let header = Row::new(["TIME", "T", "CLASS", "IP", "UA", "NONCE", "SESS", "#", "R"])
        .style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        .height(1);

    let rows: Vec<Row> = visible.iter().map(|ev| {
        let style = if ev.is_replay {
            Style::default().add_modifier(Modifier::DIM)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(format_unix_time(ev.received_at)),
            Cell::from(format!("T{}", ev.tier)),
            // ... remaining cells
        ]).style(style)
    }).collect();

    let widths = [
        Constraint::Length(12),
        Constraint::Length(4),
        Constraint::Length(12),
        Constraint::Length(16),
        Constraint::Fill(1),    // UA takes remaining width
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(5),
        Constraint::Length(6),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Events"))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(table, area, &mut app.table_state);
}
```

### Tier Color Mapping (from UI-SPEC)
```rust
// Source: 03-UI-SPEC.md color section
use ratatui::style::Color;

fn tier_color(tier: u8) -> Color {
    match tier {
        1 => Color::Cyan,    // accent
        2 => Color::Green,   // success
        3 => Color::Yellow,  // warning
        _ => Color::White,
    }
}

fn class_color(classification: &AgentClass) -> Color {
    match classification {
        AgentClass::KnownAgent { .. }   => Color::Green,
        AgentClass::KnownCrawler { .. } => Color::Red,
        AgentClass::Unknown             => Color::White,
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `tui` crate | `ratatui` crate | 2022 (fork) | `tui` is archived/unmaintained; all imports should use `ratatui::` |
| `crossterm` 0.27 event API | `crossterm` 0.29 with `event-stream` feature | 2024-2025 | `EventStream` available; enables clean `StreamExt::next()` in tokio::select! |
| `ratatui` 0.28 (split crates) | `ratatui` 0.30 (re-exports everything) | 2024-2025 | `ratatui` 0.29+ split into ratatui-core/ratatui-widgets internally but the main crate re-exports all; no user-visible change at the Cargo.toml level |

**Deprecated/outdated:**
- `tui` crate: Do not use. Archived. Use `ratatui`.
- `tui-rs`: Same as above (alternate name for the archived project).

---

## Open Questions

1. **Attach mode polling frequency**
   - What we know: DB polling is chosen as the implementation mechanism; schema has `id` column for `WHERE id > last_seen_id` queries.
   - What's unclear: Optimal poll interval (250ms is a reasonable default; too fast adds lock contention, too slow makes the attach display feel stale).
   - Recommendation: Default 250ms poll interval; acceptable for demo use. Not configurable in v1.

2. **Integrated mode shutdown coordination**
   - What we know: `axum::serve(...).with_graceful_shutdown(shutdown_rx)` requires a future. In integrated mode, the TUI quit must also stop the server.
   - What's unclear: Whether to use a `tokio::sync::oneshot` channel (TUI sends to shutdown receiver) or a `CancellationToken`.
   - Recommendation: Use a `tokio::sync::oneshot::channel()`. TUI sends `()` on quit; server awaits the receiver as its graceful shutdown future.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| ratatui | TUI rendering | Not yet in Cargo.toml | 0.30.0 (crates.io) | None — required |
| crossterm event-stream feature | Async key input | crossterm present, feature not enabled | 0.29.0 | None — required |
| futures crate | StreamExt for EventStream | Not in Cargo.toml | 0.3 | None — required |
| tokio (full features) | Async runtime | Present in Cargo.toml | 1 | — |
| SQLite (bundled) | Attach mode DB polling | Present via rusqlite bundled | 0.37 | — |

**Missing dependencies with no fallback:**
- `ratatui = "0.30"` — must be added to Cargo.toml
- `crossterm = { version = "0.29", features = ["event-stream"] }` — must replace/extend current crossterm entry (crossterm is a transitive dep currently; needs explicit entry with feature flag)
- `futures = "0.3"` — must be added to Cargo.toml for `StreamExt`

**Note:** `crossterm` is already a transitive dependency (pulled in by ratatui). An explicit `[dependencies]` entry is needed only to enable the `event-stream` feature. After adding ratatui, verify whether ratatui pulls crossterm with that feature; if not, add explicitly.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + integration tests in `tests/` |
| Config file | None — standard Rust test runner |
| Quick run command | `cargo test -p honeyprompt 2>&1 \| tail -5` |
| Full suite command | `cargo test -p honeyprompt 2>&1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLI-04 | `honeyprompt monitor --help` exits 0 with monitor subcommand visible | unit | `cargo test -p honeyprompt monitor -- --nocapture` | ❌ Wave 0 |
| TUI-01 | `AppState::push_event` appends to events vec; `visible_events()` returns correct subset | unit | `cargo test -p honeyprompt monitor::tests -- --nocapture` | ❌ Wave 0 |
| TUI-02 | Filter by tier returns only matching events; sort by time orders newest-first; sort by source orders by IP | unit | `cargo test -p honeyprompt monitor::tests::test_filter -- --nocapture` | ❌ Wave 0 |
| TUI-01 (replay) | `visible_events()` excludes replays when `show_replays=false`; includes them when `true` | unit | `cargo test -p honeyprompt monitor::tests::test_replay_filter` | ❌ Wave 0 |
| TUI-01 (stats) | `AppState::detection_count()` excludes replay events; `replay_count()` counts them | unit | `cargo test -p honeyprompt monitor::tests::test_stats` | ❌ Wave 0 |
| CLI-04 (integration) | `honeyprompt monitor` with missing project dir exits with error | integration | `cargo test -p honeyprompt --test test_monitor` | ❌ Wave 0 |

**Note:** TUI rendering (the visual frame output) is not directly testable without a real terminal. Tests focus on the `AppState` logic layer — filtering, sorting, replay exclusion, stats computation — which is pure Rust and fully testable. Rendering correctness is verified by manual screenshot during acceptance.

### Sampling Rate
- **Per task commit:** `cargo test -p honeyprompt monitor::tests 2>&1 | tail -10`
- **Per wave merge:** `cargo test -p honeyprompt 2>&1`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/monitor/mod.rs` — module file (created in Wave 0 with stub + unit tests)
- [ ] `tests/test_monitor.rs` — integration test for CLI-04 error paths
- [ ] AppState unit tests inline in `src/monitor/mod.rs` — covers TUI-01, TUI-02

*(Framework install: `cargo add ratatui@0.30 crossterm@0.29 futures@0.3` — Wave 0 step 1)*

---

## Project Constraints (from CLAUDE.md)

| Directive | Binding |
|-----------|---------|
| Language: Rust | Locked — single-binary, no Python/Node helper |
| CLI: Clap | Locked — `MonitorArgs` must use `#[derive(Parser)]` |
| TUI: Ratatui | Locked — no alternative TUI library |
| HTTP: Axum | Locked — server in integrated mode uses existing `build_router()` |
| Storage: SQLite via rusqlite | Locked — attach mode polls existing `events` table |
| Platform: Linux and macOS first | Locked — crossterm is the correct cross-platform backend |
| Performance: Fast startup, low memory | Guideline — avoid large allocations in render loop; Vec<AppEvent> grows unbounded but is acceptable for demo sessions |
| Ethics: All generated content must include visible warnings | Not directly applicable to TUI; no generated content in this phase |

---

## Sources

### Primary (HIGH confidence)
- [Ratatui Table docs (docs.rs)](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Table.html) — Table/TableState API, widths requirement, StatefulWidget usage
- [Ratatui terminal + event handler recipe](https://ratatui.rs/recipes/apps/terminal-and-event-handler/) — terminal setup/teardown pattern
- [Ratatui async event stream tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) — tokio::select! + EventStream pattern
- crates.io: ratatui 0.30.0, crossterm 0.29.0 — version numbers verified 2026-03-29 via `cargo search`
- `src/broker/mod.rs`, `src/types.rs`, `src/server/mod.rs`, `src/cli/mod.rs`, `src/main.rs` — existing codebase (HIGH confidence — direct inspection)
- `.planning/phases/03-tui-monitor/03-UI-SPEC.md` — full UI design contract (locked)
- `.planning/phases/03-tui-monitor/03-CONTEXT.md` — locked decisions D-01 through D-09

### Secondary (MEDIUM confidence)
- [Ratatui v0.29 highlights](https://ratatui.rs/highlights/v029/) — confirms TableState selected column additions in recent versions
- DB polling approach for attach mode — inferred from existing schema (`id` column, WAL mode) and CONTEXT.md "DB polling" suggestion; no official ratatui doc required

### Tertiary (LOW confidence)
- Attach mode polling interval (250ms) — heuristic; not from an authoritative source

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — versions verified against crates.io 2026-03-29
- Architecture: HIGH — based on direct codebase inspection + official Ratatui docs
- Pitfalls: HIGH — Pitfalls 1-4 verified against Ratatui docs and existing broker patterns; Pitfall 5 observed from existing main.rs structure
- Attach mode mechanism: MEDIUM — DB polling inferred from schema; IPC alternatives not researched

**Research date:** 2026-03-29
**Valid until:** 2026-04-28 (ratatui 0.30 is current; API unlikely to change in 30 days; crossterm 0.29 is stable)
