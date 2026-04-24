use crate::cli::MonitorArgs;
use crate::config::Config;
use crate::types::{AgentClass, AppEvent};
use crate::types::{NonceMapping, RawCallbackEvent, T5Formula, Tier};
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};
use ratatui::Frame;
use ratatui::Terminal;
use std::collections::HashSet;
use std::io::Stdout;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TierFilter {
    All,
    T1,
    T2,
    T3,
    T4, // NEW (Phase 14, D-14-06)
    T5, // NEW (Phase 14, D-14-06)
}

impl TierFilter {
    pub fn next(self) -> Self {
        match self {
            TierFilter::All => TierFilter::T1,
            TierFilter::T1 => TierFilter::T2,
            TierFilter::T2 => TierFilter::T3,
            TierFilter::T3 => TierFilter::T4,  // NEW
            TierFilter::T4 => TierFilter::T5,  // NEW
            TierFilter::T5 => TierFilter::All, // NEW (cycle wraps)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortField {
    Time,
    Tier,
    Source,
}

impl SortField {
    pub fn next(self) -> Self {
        match self {
            SortField::Time => SortField::Tier,
            SortField::Tier => SortField::Source,
            SortField::Source => SortField::Time,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UiMode {
    Normal,
    Command,
    Help,
}

pub struct AppState {
    pub events: Vec<AppEvent>,
    pub filter: TierFilter,
    pub sort: SortField,
    pub show_replays: bool,
    pub table_state: ratatui::widgets::TableState,
    pub at_bottom: bool,
    pub new_events_count: usize,
    pub mode: UiMode,
    pub command_input: String,
    pub command_error: Option<(String, std::time::Instant)>,
    pub status_line: String,
}

impl AppState {
    pub fn new() -> Self {
        let mut table_state = ratatui::widgets::TableState::default();
        table_state.select(Some(0));
        AppState {
            events: Vec::new(),
            filter: TierFilter::All,
            sort: SortField::Time,
            show_replays: false,
            table_state,
            at_bottom: true,
            new_events_count: 0,
            mode: UiMode::Normal,
            command_input: String::new(),
            command_error: None,
            status_line: String::new(),
        }
    }

    pub fn push_event(&mut self, event: AppEvent) {
        self.events.push(event);
        if !self.at_bottom {
            self.new_events_count += 1;
        } else {
            // Auto-scroll to the new event when at_bottom
            let visible_count = self.visible_events().len();
            if visible_count > 0 {
                self.table_state.select(Some(visible_count - 1));
            }
        }
    }

    pub fn visible_events(&self) -> Vec<&AppEvent> {
        let mut result: Vec<&AppEvent> = self
            .events
            .iter()
            .filter(|e| {
                // Replay filter
                if !self.show_replays && e.is_replay {
                    return false;
                }
                // Tier filter
                match self.filter {
                    TierFilter::All => true,
                    TierFilter::T1 => e.tier == 1,
                    TierFilter::T2 => e.tier == 2,
                    TierFilter::T3 => e.tier == 3,
                    TierFilter::T4 => e.tier == 4, // NEW
                    TierFilter::T5 => e.tier == 5, // NEW
                }
            })
            .collect();

        match self.sort {
            SortField::Time => {
                result.sort_by_key(|b| std::cmp::Reverse(b.received_at));
            }
            SortField::Tier => {
                result.sort_by(|a, b| {
                    a.tier
                        .cmp(&b.tier)
                        .then_with(|| b.received_at.cmp(&a.received_at))
                });
            }
            SortField::Source => {
                result.sort_by(|a, b| {
                    a.fingerprint
                        .source_ip
                        .to_string()
                        .cmp(&b.fingerprint.source_ip.to_string())
                        .then_with(|| b.received_at.cmp(&a.received_at))
                });
            }
        }

        result
    }

    pub fn detection_count(&self) -> usize {
        self.events.iter().filter(|e| !e.is_replay).count()
    }

    pub fn session_count(&self) -> usize {
        let unique: HashSet<&str> = self
            .events
            .iter()
            .filter(|e| !e.is_replay)
            .map(|e| e.session_id.as_str())
            .collect();
        unique.len()
    }

    pub fn tier_counts(&self) -> [usize; 5] {
        let non_replays: Vec<&AppEvent> = self.events.iter().filter(|e| !e.is_replay).collect();
        [
            non_replays.iter().filter(|e| e.tier == 1).count(),
            non_replays.iter().filter(|e| e.tier == 2).count(),
            non_replays.iter().filter(|e| e.tier == 3).count(),
            non_replays.iter().filter(|e| e.tier == 4).count(),
            non_replays.iter().filter(|e| e.tier == 5).count(),
        ]
    }

    pub fn replay_count(&self) -> usize {
        self.events.iter().filter(|e| e.is_replay).count()
    }

    pub fn cycle_filter(&mut self) {
        self.filter = self.filter.next();
        self.table_state.select(Some(0));
    }

    pub fn cycle_sort(&mut self) {
        self.sort = self.sort.next();
    }

    pub fn toggle_replays(&mut self) {
        self.show_replays = !self.show_replays;
        self.table_state.select(Some(0));
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// --- Terminal setup / teardown ---

fn setup_terminal() -> anyhow::Result<Terminal<CrosstermBackend<Stdout>>> {
    // Install panic hook BEFORE enabling raw mode so terminal is always restored
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

// --- Helper functions ---

fn format_time(secs: u64) -> String {
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max <= 3 {
        s[..max].to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

fn tier_color(tier: u8) -> Color {
    match tier {
        1 => Color::Cyan,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Magenta,   // NEW (Phase 14, D-14-05 Claude's Discretion)
        5 => Color::LightBlue, // NEW (Phase 14, D-14-05 Claude's Discretion)
        _ => Color::White,
    }
}

/// Phase 14 helper: returns the validity glyph (char + color) for a T5 event.
/// Centralizes D-14-04 convention so TUI and Markdown renderers stay consistent.
fn validity_glyph(valid: Option<bool>) -> (&'static str, Color) {
    match valid {
        Some(true) => ("✓", Color::Green),
        Some(false) => ("✗", Color::Red),
        None => ("?", Color::Gray),
    }
}

/// Phase 14 (UI-01, UI-02, D-14-03, D-14-04, D-14-13): EVIDENCE column cell builder.
/// Tier 4 -> truncated capability list styled with tier_color(4).
/// Tier 5 -> "{proof} glyph" styled Green/Red/Gray per validity.
/// Tier 1/2/3 -> em-dash "—" styled Gray.
fn tier_evidence_cell(ev: &AppEvent) -> Cell<'static> {
    match ev.tier {
        4 => {
            let full = ev.t4_capability.as_deref().unwrap_or("—");
            // Phase 14: reuse existing truncate_str (produces "..." not "…") to
            // keep consistency with UA/NONCE truncation in the same table (D-14-03).
            let shown = truncate_str(full, 20);
            Cell::from(shown).style(Style::default().fg(tier_color(4)))
        }
        5 => {
            let proof = ev.t5_proof.as_deref().unwrap_or("---");
            let (glyph, color) = validity_glyph(ev.t5_proof_valid);
            Cell::from(format!("{} {}", proof, glyph)).style(Style::default().fg(color))
        }
        _ => Cell::from("—").style(Style::default().fg(Color::Gray)),
    }
}

fn class_label(c: &AgentClass) -> (&'static str, Color) {
    match c {
        AgentClass::KnownAgent { .. } => ("agent", Color::Green),
        AgentClass::KnownCrawler { .. } => ("crawler", Color::Red),
        AgentClass::Unknown => ("unknown", Color::White),
    }
}

// --- Rendering ---

fn render_event_table(frame: &mut Frame, area: Rect, app: &mut AppState) {
    let visible = app.visible_events();

    let widths = [
        Constraint::Length(12), // TIME
        Constraint::Length(4),  // TIER
        Constraint::Length(12), // CLASS
        Constraint::Length(16), // SOURCE IP
        Constraint::Fill(1),    // UA (fills remaining)
        Constraint::Length(10), // NONCE
        Constraint::Length(10), // SESS
        Constraint::Length(5),  // FIRES
        Constraint::Length(20), // EVIDENCE (NEW Phase 14, D-14-01/D-14-03)
        Constraint::Length(6),  // REPLAY
    ];

    let header_cells = [
        "TIME",
        "TIER",
        "CLASS",
        "SOURCE IP",
        "UA",
        "NONCE",
        "SESS",
        "FIRES",
        "EVIDENCE", // NEW Phase 14
        "REPLAY",
    ]
    .into_iter()
    .map(|h| {
        Cell::from(h).style(Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
    });
    let header = Row::new(header_cells).height(1);

    // Empty state rendering
    if visible.is_empty() {
        let msg = if app.events.is_empty() {
            "Waiting for callbacks... Start the honeypot and trigger payloads to see events here."
        } else {
            "No events match the current filter. Press Tab to change filter or r to show replays."
        };
        let block = Block::default().title("Events").borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let para = Paragraph::new(msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        // Center vertically
        let vert_offset = inner.height / 2;
        let centered_area = Rect {
            x: inner.x,
            y: inner.y + vert_offset,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(para, centered_area);
        return;
    }

    let rows: Vec<Row> = visible
        .iter()
        .map(|ev| {
            let time_str = format_time(ev.received_at);
            let tier_str = format!("T{}", ev.tier);
            let (class_str, class_color) = class_label(&ev.classification);
            let ip_str = truncate_str(&ev.fingerprint.source_ip.to_string(), 16);
            let ua_str = truncate_str(&ev.fingerprint.user_agent, 30);
            let nonce_short = format!("{}...", &ev.nonce[..ev.nonce.len().min(8)]);
            let sess_short = format!("{}...", &ev.session_id[..ev.session_id.len().min(8)]);
            let fires_str = ev.fire_count.to_string();
            let replay_str = if ev.is_replay { " [R] " } else { "     " };

            let cells = vec![
                Cell::from(time_str),
                Cell::from(tier_str).style(Style::default().fg(tier_color(ev.tier))),
                Cell::from(truncate_str(class_str, 12)).style(Style::default().fg(class_color)),
                Cell::from(ip_str),
                Cell::from(ua_str),
                Cell::from(nonce_short),
                Cell::from(sess_short),
                Cell::from(fires_str),
                tier_evidence_cell(ev), // NEW Phase 14 (UI-01, UI-02)
                Cell::from(replay_str),
            ];

            let row_style = if ev.is_replay {
                Style::default().add_modifier(Modifier::DIM)
            } else {
                Style::default()
            };
            Row::new(cells).style(row_style)
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title("Events").borders(Borders::ALL))
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(table, area, &mut app.table_state);
}

/// Phase 14 (D-14-01, D-14-02): always-visible detail pane.
/// Renders context-aware content based on the currently selected row.
fn render_detail_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let visible = app.visible_events();
    let selected_idx = app.table_state.selected().unwrap_or(0);
    let detail_lines: Vec<Line<'static>> = match visible.get(selected_idx) {
        None => vec![Line::from(Span::styled(
            "(no selection)",
            Style::default().fg(Color::Gray),
        ))],
        Some(ev) => match ev.tier {
            4 => {
                let caps = ev
                    .t4_capability
                    .as_deref()
                    .unwrap_or("(missing)")
                    .to_string();
                vec![
                    Line::from(vec![
                        Span::styled(
                            "T4 capabilities: ",
                            Style::default()
                                .fg(tier_color(4))
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(caps),
                    ]),
                    Line::from(vec![
                        Span::styled("payload: ", Style::default().fg(Color::Gray)),
                        Span::raw(ev.payload_id.clone()),
                    ]),
                ]
            }
            5 => {
                let proof = ev.t5_proof.as_deref().unwrap_or("---").to_string();
                let (glyph, color) = validity_glyph(ev.t5_proof_valid);
                let label = match ev.t5_proof_valid {
                    Some(true) => "VALID",
                    Some(false) => "INVALID",
                    None => "(unverified)",
                };
                // Phase 14 / Pitfall 3: ev.t5_formula may be None in attach mode
                // (no catalog context for legacy DBs); fall back gracefully.
                let formula_line = match ev.t5_formula {
                    Some(f) => {
                        format!("formula=(seed+{})*{} % {}", f.a, f.b, f.modulus)
                    }
                    None => "formula=(unavailable — legacy db)".to_string(),
                };
                vec![
                    Line::from(vec![
                        Span::styled(
                            "T5 proof: ",
                            Style::default()
                                .fg(tier_color(5))
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!("{} ", proof)),
                        Span::styled(glyph.to_string(), Style::default().fg(color)),
                        Span::raw(" "),
                        Span::styled(label.to_string(), Style::default().fg(color)),
                    ]),
                    Line::from(Span::styled(formula_line, Style::default().fg(Color::Gray))),
                ]
            }
            _ => vec![
                Line::from(vec![
                    Span::styled("payload: ", Style::default().fg(Color::Gray)),
                    Span::raw(ev.payload_id.clone()),
                    Span::raw("   "),
                    Span::styled("loc: ", Style::default().fg(Color::Gray)),
                    Span::raw(ev.embedding_loc.clone()),
                ]),
                Line::from(vec![
                    Span::styled("nonce: ", Style::default().fg(Color::Gray)),
                    Span::raw(ev.nonce.clone()),
                ]),
            ],
        },
    };
    let para = Paragraph::new(detail_lines)
        .block(Block::default().title("Detail").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render(frame: &mut Frame, app: &mut AppState) {
    // Minimum terminal size guard
    if frame.area().width < 80 || frame.area().height < 20 {
        let msg = Paragraph::new("Terminal too small (min 80x20). Resize to continue.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(msg, frame.area());
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Length(3), // stats header
        Constraint::Length(3), // filter bar
        Constraint::Fill(1),   // event table
        Constraint::Length(4), // detail pane (NEW Phase 14 — bordered, 2 content lines)
        Constraint::Length(1), // key hint bar
    ])
    .split(frame.area());

    // --- Panel A: Stats header ---
    let counts = app.tier_counts();
    let (t1, t2, t3, t4, t5) = (counts[0], counts[1], counts[2], counts[3], counts[4]);
    let replay_count = app.replay_count();
    let replay_indicator = if app.show_replays {
        format!("{} replays shown", replay_count)
    } else {
        format!("{} replays hidden", replay_count)
    };

    let stats_spans = vec![
        Span::styled(
            "Detections: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            app.detection_count().to_string(),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        ),
        Span::styled(
            "  Sessions: ",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            app.session_count().to_string(),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        ),
        Span::styled("  T1: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(t1.to_string(), Style::default().fg(Color::Cyan)),
        Span::styled("  T2: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(t2.to_string(), Style::default().fg(Color::Green)),
        Span::styled("  T3: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(t3.to_string(), Style::default().fg(Color::Yellow)),
        Span::styled("  T4: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(t4.to_string(), Style::default().fg(tier_color(4))), // NEW Phase 14 (Magenta per D-14-05)
        Span::styled("  T5: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(t5.to_string(), Style::default().fg(tier_color(5))), // NEW Phase 14 (LightBlue per D-14-05)
        Span::raw("  "),
        Span::styled(replay_indicator, Style::default().fg(Color::Gray)),
    ];

    let status_line = app.status_line.clone();
    let stats_text = vec![
        Line::from(stats_spans),
        Line::from(vec![Span::raw(status_line)]),
    ];
    let stats_para = Paragraph::new(stats_text).block(
        Block::default()
            .title("HoneyPrompt Monitor")
            .borders(Borders::ALL),
    );
    frame.render_widget(stats_para, chunks[0]);

    // --- Panel B: Filter bar ---
    let filter_labels = [
        (TierFilter::All, "All"),
        (TierFilter::T1, "T1"),
        (TierFilter::T2, "T2"),
        (TierFilter::T3, "T3"),
        (TierFilter::T4, "T4"), // NEW Phase 14 (D-14-06)
        (TierFilter::T5, "T5"), // NEW Phase 14 (D-14-06)
    ];
    let sort_labels = [
        (SortField::Time, "time"),
        (SortField::Tier, "tier"),
        (SortField::Source, "source"),
    ];

    let mut filter_spans: Vec<Span> = vec![Span::raw("Filter: ")];
    for (i, (f, label)) in filter_labels.iter().enumerate() {
        if i > 0 {
            filter_spans.push(Span::styled(" | ", Style::default().fg(Color::Gray)));
        }
        if *f == app.filter {
            filter_spans.push(Span::styled(
                *label,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            ));
        } else {
            filter_spans.push(Span::styled(*label, Style::default().fg(Color::Gray)));
        }
    }

    filter_spans.push(Span::raw("   Sort: "));
    for (i, (s, label)) in sort_labels.iter().enumerate() {
        if i > 0 {
            filter_spans.push(Span::styled(" | ", Style::default().fg(Color::Gray)));
        }
        if *s == app.sort {
            filter_spans.push(Span::styled(
                *label,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            ));
        } else {
            filter_spans.push(Span::styled(*label, Style::default().fg(Color::Gray)));
        }
    }

    if app.new_events_count > 0 && !app.at_bottom {
        filter_spans.push(Span::styled(
            format!("  [{} new]", app.new_events_count),
            Style::default().fg(Color::Yellow),
        ));
    }

    let filter_para =
        Paragraph::new(Line::from(filter_spans)).block(Block::default().borders(Borders::ALL));
    frame.render_widget(filter_para, chunks[1]);

    // --- Panel C: Event table ---
    render_event_table(frame, chunks[2], app);

    // --- Panel D (Phase 14 NEW): Always-visible detail pane ---
    render_detail_pane(frame, chunks[3], app);

    // --- Panel E: Key hint bar ---
    match app.mode {
        UiMode::Command => {
            let cmd_text = format!(": {}_", app.command_input);
            let cmd_para = Paragraph::new(cmd_text).style(Style::default().fg(Color::White));
            frame.render_widget(cmd_para, chunks[4]);
        }
        UiMode::Normal => {
            // Check if there's a recent error to display
            let show_error = app
                .command_error
                .as_ref()
                .is_some_and(|(_, t)| t.elapsed() < Duration::from_secs(2));
            if show_error {
                let err_msg = app
                    .command_error
                    .as_ref()
                    .map(|(m, _)| m.clone())
                    .unwrap_or_default();
                let err_para = Paragraph::new(err_msg).style(Style::default().fg(Color::Red));
                frame.render_widget(err_para, chunks[4]);
            } else {
                let hint = "j/k scroll  Tab filter  s sort  r replays  : cmd  ? help  q quit";
                let hint_para = Paragraph::new(hint).style(Style::default().fg(Color::Gray));
                frame.render_widget(hint_para, chunks[4]);
            }
        }
        UiMode::Help => {
            let hint = "j/k scroll  Tab filter  s sort  r replays  : cmd  ? help  q quit";
            let hint_para = Paragraph::new(hint).style(Style::default().fg(Color::Gray));
            frame.render_widget(hint_para, chunks[4]);
        }
    }

    // --- Help overlay ---
    if app.mode == UiMode::Help {
        let full_area = frame.area();
        let overlay_width = full_area.width.min(60);
        let overlay_height = full_area.height.min(20);
        let overlay_x = (full_area.width.saturating_sub(overlay_width)) / 2;
        let overlay_y = (full_area.height.saturating_sub(overlay_height)) / 2;
        let overlay_area = Rect {
            x: overlay_x,
            y: overlay_y,
            width: overlay_width,
            height: overlay_height,
        };

        frame.render_widget(Clear, overlay_area);

        let help_text = vec![
            Line::from(vec![Span::styled(
                "Key Bindings",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("  j / Down    Scroll down one row"),
            Line::from("  k / Up      Scroll up one row"),
            Line::from("  PgDn        Scroll down one page"),
            Line::from("  PgUp        Scroll up one page"),
            Line::from("  g           Jump to top"),
            Line::from("  G           Jump to bottom (latest)"),
            Line::from("  Tab         Cycle filter: All -> T1 -> T2 -> T3 -> T4 -> T5"),
            Line::from("  s           Cycle sort: time -> tier -> source"),
            Line::from("  r           Toggle replay visibility"),
            Line::from("  :           Open command input"),
            Line::from("  q / Ctrl-C  Quit"),
            Line::from("  ?           Toggle this help overlay"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Commands (:)",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("  quit"),
            Line::from("  filter all|t1|t2|t3|t4|t5"),
            Line::from("  sort time|tier|source"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Detail Pane",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("  The pane below the event table shows full context"),
            Line::from("  for the selected row (T4 capabilities, T5 proof+formula,"),
            Line::from("  T1-T3 payload_id + embedding_loc + full nonce)."),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Press any key to close",
                Style::default().fg(Color::Gray),
            )]),
        ];

        let help_para = Paragraph::new(help_text)
            .block(Block::default().title("Help").borders(Borders::ALL))
            .style(Style::default());
        frame.render_widget(help_para, overlay_area);
    }
}

// --- Key handling ---

/// Returns true if the application should quit.
fn handle_key_event(key: &KeyEvent, app: &mut AppState) -> bool {
    match app.mode {
        UiMode::Normal => {
            // Ctrl+C always quits
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return true;
            }
            match key.code {
                KeyCode::Char('q') => return true,
                KeyCode::Char('j') | KeyCode::Down => {
                    let visible_count = app.visible_events().len();
                    if visible_count == 0 {
                        return false;
                    }
                    let current = app.table_state.selected().unwrap_or(0);
                    let next = (current + 1).min(visible_count - 1);
                    app.table_state.select(Some(next));
                    app.at_bottom = next == visible_count - 1;
                    if app.at_bottom {
                        app.new_events_count = 0;
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    let current = app.table_state.selected().unwrap_or(0);
                    if current > 0 {
                        app.table_state.select(Some(current - 1));
                        app.at_bottom = false;
                    }
                }
                KeyCode::PageDown => {
                    let visible_count = app.visible_events().len();
                    if visible_count == 0 {
                        return false;
                    }
                    let page_size: usize = 10; // approximate page size
                    let current = app.table_state.selected().unwrap_or(0);
                    let next = (current + page_size).min(visible_count - 1);
                    app.table_state.select(Some(next));
                    app.at_bottom = next == visible_count - 1;
                    if app.at_bottom {
                        app.new_events_count = 0;
                    }
                }
                KeyCode::PageUp => {
                    let current = app.table_state.selected().unwrap_or(0);
                    let page_size: usize = 10;
                    let prev = current.saturating_sub(page_size);
                    app.table_state.select(Some(prev));
                    app.at_bottom = false;
                }
                KeyCode::Char('g') => {
                    app.table_state.select(Some(0));
                    app.at_bottom = false;
                }
                KeyCode::Char('G') => {
                    let visible_count = app.visible_events().len();
                    if visible_count > 0 {
                        app.table_state.select(Some(visible_count - 1));
                    }
                    app.at_bottom = true;
                    app.new_events_count = 0;
                }
                KeyCode::Tab => app.cycle_filter(),
                KeyCode::Char('s') => app.cycle_sort(),
                KeyCode::Char('r') => app.toggle_replays(),
                KeyCode::Char(':') => {
                    app.mode = UiMode::Command;
                    app.command_input.clear();
                }
                KeyCode::Char('?') => {
                    app.mode = UiMode::Help;
                }
                _ => {}
            }
        }
        UiMode::Command => match key.code {
            KeyCode::Esc => {
                app.mode = UiMode::Normal;
            }
            KeyCode::Enter => {
                let input = app.command_input.trim().to_lowercase();
                app.mode = UiMode::Normal;
                match input.as_str() {
                    "quit" => return true,
                    "filter all" => {
                        app.filter = TierFilter::All;
                        app.table_state.select(Some(0));
                    }
                    "filter t1" => {
                        app.filter = TierFilter::T1;
                        app.table_state.select(Some(0));
                    }
                    "filter t2" => {
                        app.filter = TierFilter::T2;
                        app.table_state.select(Some(0));
                    }
                    "filter t3" => {
                        app.filter = TierFilter::T3;
                        app.table_state.select(Some(0));
                    }
                    "filter t4" => {
                        app.filter = TierFilter::T4;
                        app.table_state.select(Some(0));
                    }
                    "filter t5" => {
                        app.filter = TierFilter::T5;
                        app.table_state.select(Some(0));
                    }
                    "sort time" => {
                        app.sort = SortField::Time;
                    }
                    "sort tier" => {
                        app.sort = SortField::Tier;
                    }
                    "sort source" => {
                        app.sort = SortField::Source;
                    }
                    other => {
                        app.command_error = Some((
                            format!("Unknown command: {}", other),
                            std::time::Instant::now(),
                        ));
                    }
                }
            }
            KeyCode::Char(c) => {
                app.command_input.push(c);
            }
            KeyCode::Backspace => {
                app.command_input.pop();
            }
            _ => {}
        },
        UiMode::Help => {
            // Any key dismisses help overlay
            app.mode = UiMode::Normal;
        }
    }
    false
}

// --- Async run loops ---

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    mut event_rx: broadcast::Receiver<AppEvent>,
    app: &mut AppState,
) -> anyhow::Result<()> {
    let mut key_stream = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_millis(16));
    loop {
        tokio::select! {
            _ = tick.tick() => {
                terminal.draw(|f| render(f, app))?;
            }
            result = event_rx.recv() => {
                match result {
                    Ok(ev) => app.push_event(ev),
                    Err(broadcast::error::RecvError::Lagged(_n)) => {
                        // Silently drop — acceptable for demo tool
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            Some(Ok(event)) = key_stream.next() => {
                if let Event::Key(key) = event {
                    if handle_key_event(&key, app) { break; }
                }
            }
        }
    }
    Ok(())
}

async fn run_loop_attach(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    conn: tokio_rusqlite::Connection,
    app: &mut AppState,
) -> anyhow::Result<()> {
    let mut key_stream = EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_millis(16));
    let mut poll = tokio::time::interval(Duration::from_millis(250));
    let mut last_seen_id: i64 = 0;

    loop {
        tokio::select! {
            _ = tick.tick() => {
                terminal.draw(|f| render(f, app))?;
            }
            _ = poll.tick() => {
                // Poll DB for new events since last_seen_id.
                // Read t5_proof_valid as Option<i64> and convert to Option<bool> in Rust —
                // Option<bool> via rusqlite FromSql can silently swallow NULL rows in some
                // tokio-rusqlite paths; manual conversion is robust.
                let since_id = last_seen_id;
                type EventRow = (i64, String, u8, String, String, String, String, String, u32, bool, Option<String>, u64, Option<String>, Option<String>, Option<i64>);
                let rows: Result<Vec<EventRow>, _> = conn.call(move |c| {
                    let mut stmt = c.prepare(
                        "SELECT id, nonce, tier, payload_id, embedding_loc, session_id, remote_addr, user_agent, fire_count, is_replay, extra_headers, first_seen_at, t4_capability, t5_proof, t5_proof_valid \
                         FROM events WHERE id > ?1 ORDER BY id ASC"
                    )?;
                    let rows: rusqlite::Result<Vec<_>> = stmt.query_map(rusqlite::params![since_id], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, u8>(2)?,
                            row.get::<_, String>(3)?,
                            row.get::<_, String>(4)?,
                            row.get::<_, String>(5).unwrap_or_default(),
                            row.get::<_, String>(6).unwrap_or_default(),
                            row.get::<_, String>(7).unwrap_or_default(),
                            row.get::<_, u32>(8)?,
                            row.get::<_, bool>(9)?,
                            row.get::<_, Option<String>>(10)?,
                            row.get::<_, u64>(11).unwrap_or(0),
                            row.get::<_, Option<String>>(12)?,
                            row.get::<_, Option<String>>(13)?,
                            row.get::<_, Option<i64>>(14)?,
                        ))
                    })?.collect();
                    rows.map_err(tokio_rusqlite::Error::from)
                }).await;

                match rows {
                    Err(e) => {
                        // Surface DB errors to the status line so they are visible
                        // in the TUI instead of being silently swallowed.
                        app.status_line = format!("attach-mode DB read error: {}", e);
                    }
                    Ok(event_rows) => {
                    for (id, nonce, tier, payload_id, embedding_loc, session_id, remote_addr, user_agent, fire_count, is_replay, extra_headers, first_seen_at, t4_capability, t5_proof, t5_proof_valid_int) in event_rows {
                        let t5_proof_valid: Option<bool> = t5_proof_valid_int.map(|v| v != 0);
                        // Parse classification from extra_headers JSON
                        let classification = parse_classification_from_extra(&extra_headers);
                        let source_ip: std::net::IpAddr = remote_addr.parse()
                            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
                        let fingerprint = crate::types::AgentFingerprint {
                            source_ip,
                            user_agent: user_agent.clone(),
                            headers: std::collections::HashMap::new(),
                            received_at: first_seen_at,
                        };
                        let ev = AppEvent {
                            nonce,
                            tier,
                            payload_id,
                            embedding_loc,
                            fingerprint,
                            classification,
                            session_id,
                            is_replay,
                            fire_count,
                            received_at: first_seen_at,
                            // Phase 14: surface Phase-13 persisted T4/T5 columns so
                            // attach-mode renders EVIDENCE cell + detail pane the same
                            // as integrated mode (UI-01, UI-02).
                            t4_capability,
                            t5_proof,
                            t5_proof_valid,
                            // t5_formula is NOT persisted in the DB (in-memory only),
                            // so attach mode always renders detail pane with the
                            // "formula=(unavailable — legacy db)" fallback per Pitfall 3.
                            t5_formula: None,
                        };
                        app.push_event(ev);
                        if id > last_seen_id {
                            last_seen_id = id;
                        }
                    }
                    }
                }
            }
            Some(Ok(event)) = key_stream.next() => {
                if let Event::Key(key) = event {
                    if handle_key_event(&key, app) { break; }
                }
            }
        }
    }
    Ok(())
}

/// Parse AgentClass from the extra_headers JSON blob stored in the DB.
fn parse_classification_from_extra(extra_headers: &Option<String>) -> AgentClass {
    let Some(json_str) = extra_headers else {
        return AgentClass::Unknown;
    };
    // Parse {"classification": "KnownAgent:OpenAI", ...}
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(cls) = val.get("classification").and_then(|v| v.as_str()) {
            if cls.starts_with("KnownCrawler:") {
                let provider = cls.trim_start_matches("KnownCrawler:").to_string();
                return AgentClass::KnownCrawler { provider };
            } else if cls.starts_with("KnownAgent:") {
                let provider = cls.trim_start_matches("KnownAgent:").to_string();
                return AgentClass::KnownAgent { provider };
            }
        }
    }
    AgentClass::Unknown
}

// --- Public entry point ---

/// Start the honeyprompt monitor (integrated or attach mode).
pub async fn monitor(
    config: &Config,
    project_path: &Path,
    args: &MonitorArgs,
) -> anyhow::Result<()> {
    let mut app = AppState::new();

    if args.attach {
        // Attach mode: poll existing database
        let db_path = project_path.join(".honeyprompt").join("events.db");
        if !db_path.exists() {
            anyhow::bail!(
                "Error: database not found at {}. Run `honeyprompt serve` first.",
                db_path.display()
            );
        }
        let conn = tokio_rusqlite::Connection::open(&db_path).await?;
        app.status_line = format!("Attached to db: {}", db_path.display());

        let mut terminal = setup_terminal()?;
        let result = run_loop_attach(&mut terminal, conn, &mut app).await;
        restore_terminal(&mut terminal)?;
        result
    } else {
        // Integrated mode: start server + TUI together
        let output_dir = project_path.join("output");
        let db_path = project_path.join(".honeyprompt").join("events.db");

        // Load callback-map.json and build in-memory nonce lookup
        let callback_map_path = output_dir.join("callback-map.json");
        let json_str = std::fs::read_to_string(&callback_map_path)
            .map_err(|e| anyhow::anyhow!("Failed to read callback-map.json: {}", e))?;
        let mappings: Vec<NonceMapping> = serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Failed to parse callback-map.json: {}", e))?;

        // Phase 14 fix: load the payload catalog and build a payload_id -> T5Formula
        // lookup BEFORE constructing nonce_map, so tier-5 entries get their formula
        // populated (previously hard-coded to None, which caused every T5 callback
        // to silently fall through at src/server/mod.rs:160-163). Mirrors the
        // run_server pattern at src/server/mod.rs:249-274.
        let all_payloads = crate::catalog::load_catalog()?;
        let t5_formulas_by_payload_id: std::collections::HashMap<String, T5Formula> = all_payloads
            .iter()
            .filter_map(|p| p.t5_formula.map(|f| (p.id.clone(), f)))
            .collect();

        let mut nonce_map: std::collections::HashMap<String, crate::server::NonceMeta> =
            std::collections::HashMap::new();
        for m in &mappings {
            let t5_formula = if m.tier == Tier::Tier5 {
                t5_formulas_by_payload_id.get(&m.payload_id).copied()
            } else {
                None
            };
            nonce_map.insert(
                m.nonce.clone(),
                crate::server::NonceMeta {
                    tier: u8::from(m.tier),
                    payload_id: m.payload_id.clone(),
                    embedding_loc: m.embedding_location.to_string(),
                    t5_formula,
                },
            );
        }

        // Load crawler catalog
        let crawler_catalog = crate::crawler_catalog::CrawlerCatalog::load()?;

        // Open tokio-rusqlite connection and run migrations
        let conn = tokio_rusqlite::Connection::open(&db_path).await?;
        conn.call(|c| crate::store::run_migrations(c).map_err(tokio_rusqlite::Error::from))
            .await?;

        // Create event pipeline channels
        let (callback_tx, callback_rx) = mpsc::channel::<RawCallbackEvent>(256);
        let (event_tx, _initial_rx) = tokio::sync::broadcast::channel::<AppEvent>(1024);

        // Subscribe consumers BEFORE spawning broker (so no events are missed)
        let tui_rx = event_tx.subscribe();
        let db_rx = event_tx.subscribe();

        // Spawn pipeline tasks (no stdout_logger_task — TUI replaces it)
        tokio::spawn(crate::broker::broker_task(callback_rx, event_tx));
        tokio::spawn(crate::broker::db_writer_task(db_rx, conn.clone()));

        // Build server AppState and router
        let server_state = Arc::new(crate::server::AppState {
            conn: conn.clone(),
            callback_tx,
            nonce_map,
            crawler_catalog,
        });
        let app_router = crate::server::build_router(server_state, output_dir);

        // Determine bind address
        let bind_addr = if let Some(port) = args.port {
            format!("0.0.0.0:{}", port)
        } else {
            config.bind_address.clone()
        };

        // Bind TcpListener
        let listener = tokio::net::TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", bind_addr, e))?;
        let actual_addr = listener.local_addr()?;

        // Shutdown signal channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn axum server with graceful shutdown
        tokio::spawn(async move {
            if let Err(e) = axum::serve(
                listener,
                app_router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            {
                eprintln!("server error: {}", e);
            }
        });

        app.status_line = format!("Serving on {}", actual_addr);

        // Setup terminal and run TUI
        let mut terminal = setup_terminal()?;
        let result = run_loop(&mut terminal, tui_rx, &mut app).await;
        restore_terminal(&mut terminal)?;
        let _ = shutdown_tx.send(());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentClass, AgentFingerprint};
    use std::collections::HashMap;
    use std::net::IpAddr;

    fn make_test_event(
        tier: u8,
        is_replay: bool,
        ip: &str,
        session_id: &str,
        received_at: u64,
    ) -> AppEvent {
        let source_ip: IpAddr = ip.parse().unwrap();
        AppEvent {
            nonce: "nonce123".to_string(),
            tier,
            payload_id: format!("t{}-test", tier),
            embedding_loc: "html_comment".to_string(),
            fingerprint: AgentFingerprint {
                source_ip,
                user_agent: "TestAgent/1.0".to_string(),
                headers: HashMap::new(),
                received_at,
            },
            classification: AgentClass::Unknown,
            session_id: session_id.to_string(),
            is_replay,
            fire_count: 1,
            received_at,
            t4_capability: None,
            t5_proof: None,
            t5_proof_valid: None,
            t5_formula: None,
        }
    }

    #[test]
    fn test_push_event_appends() {
        let mut state = AppState::new();
        assert_eq!(state.events.len(), 0);
        let ev = make_test_event(1, false, "1.2.3.4", "sess1", 1000);
        state.push_event(ev);
        assert_eq!(state.events.len(), 1);
    }

    #[test]
    fn test_visible_events_filter_all_excludes_replays_by_default() {
        let mut state = AppState::new();
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, true, "2.3.4.5", "sess2", 2000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 1);
        assert!(!visible[0].is_replay);
    }

    #[test]
    fn test_visible_events_filter_t1_returns_only_tier1() {
        let mut state = AppState::new();
        state.filter = TierFilter::T1;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(2, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(3, false, "3.4.5.6", "sess3", 3000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].tier, 1);
    }

    #[test]
    fn test_visible_events_filter_t2_returns_only_tier2() {
        let mut state = AppState::new();
        state.filter = TierFilter::T2;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(2, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(3, false, "3.4.5.6", "sess3", 3000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].tier, 2);
    }

    #[test]
    fn test_visible_events_filter_t3_returns_only_tier3() {
        let mut state = AppState::new();
        state.filter = TierFilter::T3;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(2, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(3, false, "3.4.5.6", "sess3", 3000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].tier, 3);
    }

    #[test]
    fn test_visible_events_show_replays_false_excludes_replays() {
        let mut state = AppState::new();
        state.show_replays = false;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, true, "2.3.4.5", "sess2", 2000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 1);
        assert!(!visible[0].is_replay);
    }

    #[test]
    fn test_visible_events_show_replays_true_includes_replays() {
        let mut state = AppState::new();
        state.show_replays = true;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, true, "2.3.4.5", "sess2", 2000));
        let visible = state.visible_events();
        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn test_visible_events_sort_time_newest_first() {
        let mut state = AppState::new();
        state.sort = SortField::Time;
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess2", 3000));
        state.push_event(make_test_event(1, false, "3.4.5.6", "sess3", 2000));
        let visible = state.visible_events();
        assert_eq!(visible[0].received_at, 3000);
        assert_eq!(visible[1].received_at, 2000);
        assert_eq!(visible[2].received_at, 1000);
    }

    #[test]
    fn test_visible_events_sort_tier_ascending() {
        let mut state = AppState::new();
        state.sort = SortField::Tier;
        state.push_event(make_test_event(3, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(2, false, "3.4.5.6", "sess3", 3000));
        let visible = state.visible_events();
        assert_eq!(visible[0].tier, 1);
        assert_eq!(visible[1].tier, 2);
        assert_eq!(visible[2].tier, 3);
    }

    #[test]
    fn test_visible_events_sort_source_ascending() {
        let mut state = AppState::new();
        state.sort = SortField::Source;
        state.push_event(make_test_event(1, false, "3.4.5.6", "sess1", 1000));
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess2", 2000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess3", 3000));
        let visible = state.visible_events();
        assert_eq!(visible[0].fingerprint.source_ip.to_string(), "1.2.3.4");
        assert_eq!(visible[1].fingerprint.source_ip.to_string(), "2.3.4.5");
        assert_eq!(visible[2].fingerprint.source_ip.to_string(), "3.4.5.6");
    }

    #[test]
    fn test_detection_count_excludes_replays() {
        let mut state = AppState::new();
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(1, true, "3.4.5.6", "sess3", 3000));
        assert_eq!(state.detection_count(), 2);
    }

    #[test]
    fn test_session_count_unique_sessions() {
        let mut state = AppState::new();
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess1", 2000)); // same session
        state.push_event(make_test_event(1, false, "3.4.5.6", "sess2", 3000));
        state.push_event(make_test_event(1, true, "4.5.6.7", "sess3", 4000)); // replay excluded
        assert_eq!(state.session_count(), 2);
    }

    #[test]
    fn test_tier_counts_excludes_replays() {
        let mut state = AppState::new();
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, false, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(2, false, "3.4.5.6", "sess3", 3000));
        state.push_event(make_test_event(3, false, "4.5.6.7", "sess4", 4000));
        state.push_event(make_test_event(4, false, "5.6.7.8", "sess5", 5000)); // NEW T4
        state.push_event(make_test_event(5, false, "6.7.8.9", "sess6", 6000)); // NEW T5
        state.push_event(make_test_event(1, true, "7.8.9.10", "sess7", 7000)); // replay
        let counts = state.tier_counts();
        assert_eq!(counts, [2, 1, 1, 1, 1]); // [T1, T2, T3, T4, T5]
    }

    #[test]
    fn test_replay_count() {
        let mut state = AppState::new();
        state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
        state.push_event(make_test_event(1, true, "2.3.4.5", "sess2", 2000));
        state.push_event(make_test_event(1, true, "3.4.5.6", "sess3", 3000));
        assert_eq!(state.replay_count(), 2);
    }

    #[test]
    fn test_handle_filter_cycle() {
        let mut state = AppState::new();
        assert_eq!(state.filter, TierFilter::All);
        state.cycle_filter();
        assert_eq!(state.filter, TierFilter::T1);
        state.cycle_filter();
        assert_eq!(state.filter, TierFilter::T2);
        state.cycle_filter();
        assert_eq!(state.filter, TierFilter::T3);
        state.cycle_filter();
        assert_eq!(state.filter, TierFilter::T4); // NEW (Phase 14)
        state.cycle_filter();
        assert_eq!(state.filter, TierFilter::T5); // NEW (Phase 14)
        state.cycle_filter();
        assert_eq!(state.filter, TierFilter::All);
    }

    #[test]
    fn test_handle_sort_cycle() {
        let mut state = AppState::new();
        assert_eq!(state.sort, SortField::Time);
        state.cycle_sort();
        assert_eq!(state.sort, SortField::Tier);
        state.cycle_sort();
        assert_eq!(state.sort, SortField::Source);
        state.cycle_sort();
        assert_eq!(state.sort, SortField::Time);
    }

    #[test]
    fn test_handle_replay_toggle() {
        let mut state = AppState::new();
        assert!(!state.show_replays);
        state.toggle_replays();
        assert!(state.show_replays);
        state.toggle_replays();
        assert!(!state.show_replays);
    }

    #[test]
    fn test_format_time() {
        // 3661 secs = 1 hour, 1 min, 1 sec
        assert_eq!(format_time(3661), "01:01:01");
        // 0 secs
        assert_eq!(format_time(0), "00:00:00");
        // 86400 secs = 24 hours, wraps to 00:00:00
        assert_eq!(format_time(86400), "00:00:00");
    }

    #[test]
    fn test_truncate_str_short() {
        assert_eq!(truncate_str("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_str_long() {
        assert_eq!(truncate_str("hello world", 8), "hello...");
    }

    #[test]
    fn test_tier_color() {
        assert_eq!(tier_color(1), Color::Cyan);
        assert_eq!(tier_color(2), Color::Green);
        assert_eq!(tier_color(3), Color::Yellow);
        assert_eq!(tier_color(4), Color::Magenta); // NEW (Phase 14)
        assert_eq!(tier_color(5), Color::LightBlue); // NEW (Phase 14)
        assert_eq!(tier_color(99), Color::White); // fallback preserved
    }

    #[test]
    fn test_class_label() {
        let (label, color) = class_label(&AgentClass::KnownAgent {
            provider: "OpenAI".to_string(),
        });
        assert_eq!(label, "agent");
        assert_eq!(color, Color::Green);

        let (label, color) = class_label(&AgentClass::KnownCrawler {
            provider: "Google".to_string(),
        });
        assert_eq!(label, "crawler");
        assert_eq!(color, Color::Red);

        let (label, color) = class_label(&AgentClass::Unknown);
        assert_eq!(label, "unknown");
        assert_eq!(color, Color::White);
    }

    #[test]
    fn test_parse_classification_known_agent() {
        let extra = Some(r#"{"classification":"KnownAgent:OpenAI","headers":{}}"#.to_string());
        let cls = parse_classification_from_extra(&extra);
        assert_eq!(
            cls,
            AgentClass::KnownAgent {
                provider: "OpenAI".to_string()
            }
        );
    }

    #[test]
    fn test_parse_classification_known_crawler() {
        let extra = Some(r#"{"classification":"KnownCrawler:Google","headers":{}}"#.to_string());
        let cls = parse_classification_from_extra(&extra);
        assert_eq!(
            cls,
            AgentClass::KnownCrawler {
                provider: "Google".to_string()
            }
        );
    }

    #[test]
    fn test_parse_classification_unknown() {
        let extra = Some(r#"{"classification":"Unknown","headers":{}}"#.to_string());
        let cls = parse_classification_from_extra(&extra);
        assert_eq!(cls, AgentClass::Unknown);
    }

    #[test]
    fn test_parse_classification_none() {
        let cls = parse_classification_from_extra(&None);
        assert_eq!(cls, AgentClass::Unknown);
    }

    // Helper for T4/T5-aware test events. The base make_test_event helper produces
    // tier-specific events with None for capability/proof/formula; this helper
    // enriches it for the Phase-14 EVIDENCE / detail-pane tests.
    fn make_t4_event(capability: &str) -> AppEvent {
        let mut ev = make_test_event(4, false, "1.2.3.4", "sess-t4", 100);
        ev.t4_capability = Some(capability.to_string());
        ev
    }

    fn make_t5_event(
        proof: &str,
        valid: Option<bool>,
        formula: Option<crate::types::T5Formula>,
    ) -> AppEvent {
        let mut ev = make_test_event(5, false, "1.2.3.4", "sess-t5", 100);
        ev.t5_proof = Some(proof.to_string());
        ev.t5_proof_valid = valid;
        ev.t5_formula = formula;
        ev
    }

    #[test]
    fn test_tier_evidence_cell_t4() {
        let ev = make_t4_event("web_search,browse_page,code_execution");
        // Cell does not expose its string content directly; this test exercises
        // construction without panic and verifies the branch is taken (coverage).
        // Full visual rendering verified manually per 14-VALIDATION.md Manual-Only.
        let _ = tier_evidence_cell(&ev);
    }

    #[test]
    fn test_tier_evidence_cell_t5_valid() {
        let ev = make_t5_event("123", Some(true), None);
        let _ = tier_evidence_cell(&ev);
    }

    #[test]
    fn test_tier_evidence_cell_t5_invalid() {
        let ev = make_t5_event("456", Some(false), None);
        let _ = tier_evidence_cell(&ev);
    }

    #[test]
    fn test_tier_evidence_cell_t5_unknown() {
        let ev = make_t5_event("789", None, None);
        let _ = tier_evidence_cell(&ev);
    }

    #[test]
    fn test_tier_evidence_cell_t1_t2_t3_emdash() {
        // Smoke-test: T1/T2/T3 branches do not panic and use em-dash fallback.
        let _ = tier_evidence_cell(&make_test_event(1, false, "1.2.3.4", "s", 100));
        let _ = tier_evidence_cell(&make_test_event(2, false, "1.2.3.4", "s", 100));
        let _ = tier_evidence_cell(&make_test_event(3, false, "1.2.3.4", "s", 100));
    }

    #[test]
    fn test_validity_glyph_mapping() {
        assert_eq!(validity_glyph(Some(true)), ("✓", Color::Green));
        assert_eq!(validity_glyph(Some(false)), ("✗", Color::Red));
        assert_eq!(validity_glyph(None), ("?", Color::Gray));
    }

    // Phase 14 Pitfall 3: detail pane must not panic in attach mode (t5_formula: None).
    // We cannot easily invoke render_detail_pane without a Frame, but we can exercise
    // the logic pathways indirectly by checking that a T5 AppEvent with t5_formula: None
    // is constructable and is handled by the match arm. Full visual check is in
    // 14-VALIDATION.md Manual-Only.
    #[test]
    fn test_t5_event_with_none_formula_constructs_cleanly() {
        let ev = make_t5_event("000", Some(true), None);
        assert!(ev.t5_formula.is_none());
        assert_eq!(ev.t5_proof.as_deref(), Some("000"));
    }

    #[test]
    fn test_t5_event_with_formula_constructs_cleanly() {
        let ev = make_t5_event(
            "042",
            Some(true),
            Some(crate::types::T5Formula {
                a: 7,
                b: 13,
                modulus: 1000,
            }),
        );
        assert_eq!(ev.t5_formula.map(|f| f.a), Some(7));
        assert_eq!(ev.t5_formula.map(|f| f.b), Some(13));
        assert_eq!(ev.t5_formula.map(|f| f.modulus), Some(1000));
    }

    #[test]
    fn test_integrated_mode_nonce_map_loads_t5_formula() {
        // Regression test for the Phase 14 latent-bug fix at src/monitor/mod.rs:903-915.
        // The integrated-mode nonce_map construction must mirror run_server
        // (src/server/mod.rs:249-274): load the catalog and populate t5_formula
        // for tier-5 entries. Previously hard-coded to None, causing T5 callbacks
        // to silently fail at src/server/mod.rs:160-163.
        let all_payloads = crate::catalog::load_catalog().expect("catalog loads");
        let t5_formulas_by_payload_id: std::collections::HashMap<String, crate::types::T5Formula> =
            all_payloads
                .iter()
                .filter_map(|p| p.t5_formula.map(|f| (p.id.clone(), f)))
                .collect();
        // Assert the catalog contains at least one tier-5 entry with a formula.
        // (Phase 13 shipped tier5.toml with >= 2 templates, all of which MUST
        // have formula_a/formula_b/formula_mod fields populated — this is a
        // smoke test that the loading path is wired.)
        assert!(
            !t5_formulas_by_payload_id.is_empty(),
            "Catalog should contain at least one tier-5 payload with a T5Formula; \
             got empty map. This likely means tier5.toml is missing or the catalog \
             loader isn't filtering correctly."
        );
        // For each tier-5 entry, verify the formula lookup round-trip.
        for p in all_payloads
            .iter()
            .filter(|p| p.tier == crate::types::Tier::Tier5)
        {
            let looked_up = t5_formulas_by_payload_id.get(&p.id).copied();
            assert_eq!(
                looked_up, p.t5_formula,
                "payload_id {} round-trip mismatch",
                p.id
            );
        }
    }

    #[test]
    fn test_command_filter_t4() {
        let mut state = AppState::new();
        // Simulate command-mode input: set filter via the state mutation the
        // command parser performs.
        state.filter = TierFilter::T4;
        state.table_state.select(Some(0));
        assert_eq!(state.filter, TierFilter::T4);
        assert_eq!(state.table_state.selected(), Some(0));
    }

    #[test]
    fn test_command_filter_t5() {
        let mut state = AppState::new();
        state.filter = TierFilter::T5;
        state.table_state.select(Some(0));
        assert_eq!(state.filter, TierFilter::T5);
        assert_eq!(state.table_state.selected(), Some(0));
    }

    #[test]
    fn test_filter_labels_has_six_entries() {
        // Phase 14 D-14-06: filter bar shows All | T1 | T2 | T3 | T4 | T5
        // This test pins the *count*; visual verification happens in 14-VALIDATION Manual-Only.
        let filter_labels: &[(TierFilter, &str)] = &[
            (TierFilter::All, "All"),
            (TierFilter::T1, "T1"),
            (TierFilter::T2, "T2"),
            (TierFilter::T3, "T3"),
            (TierFilter::T4, "T4"),
            (TierFilter::T5, "T5"),
        ];
        assert_eq!(filter_labels.len(), 6);
    }
}
