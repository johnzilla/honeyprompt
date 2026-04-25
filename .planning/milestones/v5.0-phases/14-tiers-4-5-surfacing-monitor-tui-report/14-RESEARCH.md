# Phase 14: Tiers 4 & 5 Surfacing (Monitor TUI + Report) - Research

**Researched:** 2026-04-24
**Domain:** Ratatui TUI extension + Rust SQLite aggregation + Markdown rendering
**Confidence:** HIGH

## Summary

Phase 14 is a mechanically-constrained, almost-entirely-additive extension. All architectural decisions are already locked in `14-CONTEXT.md`; the codebase landmarks (`render_event_table`, `tier_color`, `TierFilter`, `proof_level`, `ReportSummary`, `ReportSession`, `NonceMeta.t5_formula`) exist and are discoverable. Phase 13 already wrote `t4_capability`, `t5_proof`, `t5_proof_valid` to the `AppEvent` struct and to the `events` SQLite table — this phase is strictly about surfacing what's already in memory / on disk.

Research confirms three executionally important facts: (1) SQLite `MAX(col)` returns `NULL` when all rows in the group are `NULL`, so the "legacy v4.0 DB" case is NULL-safe without any runtime branching; (2) Ratatui 0.30 `Cell::from(x).style(...)` supports per-cell styling independent of row style, so T5 `NNN ✓`/`NNN ✗` coloring coexists with replay-row `Modifier::DIM`; (3) the T5 formula is already cached server-side in `NonceMeta.t5_formula`, but `AppState` in `src/monitor/mod.rs` does NOT currently hold the nonce_map / catalog — so the detail pane needs either a new `AppEvent` field or a small `HashMap<payload_id, T5Formula>` cached in `AppState` at startup.

**Primary recommendation:** Extend `AppEvent` with `t5_formula: Option<T5Formula>` populated by the broker/handler at event assembly time. This keeps `AppState` simple, makes attach-mode trivially work (read-only — a legacy DB has no formula, pane just omits the formula line), and avoids a second HashMap lookup at render time.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| Event-table column rendering | Monitor TUI (`src/monitor/mod.rs`) | — | All TUI state lives in `AppState`; cell construction is local to `render_event_table` |
| Detail-pane rendering | Monitor TUI (`src/monitor/mod.rs`) | — | New `render_detail_pane` helper; sibling to `render_event_table` |
| Tier-filter cycle | Monitor TUI (`src/monitor/mod.rs`) | — | `TierFilter` enum + `cycle_filter`; pure state mutation |
| T5 formula availability at render time | Broker (`src/broker/mod.rs`) | Monitor TUI | Formula already lives in `NonceMeta` server-side; propagate via `RawCallbackEvent` → `AppEvent` like T4 capability already does |
| Session-grouped T4/T5 aggregation | Store (`src/store/mod.rs`) | Report (`src/report/mod.rs`) | SQL `MAX(col)` with `GROUP BY session_id, tier` — first-write-wins already guaranteed by D-13-19 |
| Executive summary counts | Store (`src/store/mod.rs`) | Report (`src/report/mod.rs`) | Two new `SELECT COUNT(DISTINCT ...)` calls in `query_report_summary` mirroring tier1/2/3 pattern |
| Markdown cell rendering | Report (`src/report/mod.rs`) | — | Pure string formatting; `md_escape` already handles T1–T3 cells identically |
| Backward-compat NULL handling | Store (`src/store/mod.rs`) | Report + Monitor | `rusqlite::Row::get::<_, Option<T>>` already NULL-safe; no branching needed |

## User Constraints (from CONTEXT.md)

### Locked Decisions (verbatim from 14-CONTEXT.md)

**TUI Evidence Placement:**
- **D-14-01:** Monitor TUI gains (a) a compact `EVIDENCE` column in the event table and (b) a fixed always-visible detail pane below the table (above the hint bar).
- **D-14-02:** Detail pane is context-aware:
  - T4 row → full decoded, sorted capability list (no truncation).
  - T5 row → `proof=NNN ✓ VALID` or `proof=NNN ✗ INVALID` plus formula line `formula=(seed+A)*B % M`.
  - T1–T3 row → `payload_id`, `embedding_loc`, full `nonce`.
- **D-14-03:** T4 capability truncated with `…` in EVIDENCE column; detail pane always full.
- **D-14-04:** T5 renders `NNN ✓` green / `NNN ✗` red in EVIDENCE column; detail pane adds explicit VALID/INVALID text.

**TUI Chrome:**
- **D-14-05:** Stats header appends `T4:n T5:n`. T4/T5 get distinct colors (Claude's Discretion).
- **D-14-06:** Tab filter cycle: `All → T1 → T2 → T3 → T4 → T5 → All`. `:filter t4` / `:filter t5` added.
- **D-14-07:** T5 valid/invalid split ONLY in detail pane. No `:filter t5valid` / `:filter t5invalid`.
- **D-14-07a:** Help overlay updated. No new key bindings.

**Report:**
- **D-14-08:** Evidence Table stays session-grouped; one new `Evidence` column.
- **D-14-09:** `proof_level()` extends: `4 → "Capability Introspection"`, `5 → "Multi-step Compliance"`.
- **D-14-10:** T4 capability rendered in full inside Markdown cell (no truncation); passed through `md_escape()`.
- **D-14-11:** T5 rendered as `NNN ✓ VALID` / `NNN ✗ INVALID`.

**Backward-Compat:**
- **D-14-12:** Always-show chrome — 5 tier rows in exec summary, zero-count visible.
- **D-14-13:** T1–T3 rows in Evidence column show em-dash `—`.
- **D-14-14:** Queries always SELECT T4/T5 columns; rely on Phase 13 migration guarantee.

### Claude's Discretion (research makes recommendations)

- Exact EVIDENCE column width and which existing column shrinks.
- Detail-pane height (1 vs 2–3 lines).
- Label wording in detail pane.
- Ratatui color variants for T4/T5 tier labels.
- Whether `ReportSummary` gains fields vs `ReportSession` gaining optionals.
- Markdown evidence column ordering.
- Filter bar overflow handling.
- Helper refactoring of `tier_color`/`proof_level`.

### Deferred Ideas (OUT OF SCOPE)

- `:filter t5valid` / `:filter t5invalid` split commands
- Header split `T5✓:n T5✗:n`
- Per-event (non-session-grouped) evidence rendering
- `PRAGMA table_info` schema probing
- `test-agent` T4/T5 scorecard + CI exit codes (Phase 15)
- README 5-tier docs + TODOS cleanup (Phase 15)
- JSON/HTML report formats
- Web dashboard for T4/T5

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UI-01 | Monitor TUI renders Tier 4 capability summaries in detail/row view | Add `EVIDENCE` column + detail pane; T4 branch shows full list — see §Pattern 1 (EVIDENCE Cell) + §Pattern 2 (Detail Pane) |
| UI-02 | Monitor TUI renders Tier 5 chain proofs with visible validity indicator | `Cell::from("{proof} ✓").style(Color::Green)` / `Cell::from("{proof} ✗").style(Color::Red)` — see §Pattern 1 |
| UI-03 | Markdown report shows per-event T4 evidence alongside T1–T3 | Session-grouped Evidence column, `md_escape(t4_capability)` — see §Pattern 3 |
| UI-04 | Markdown report shows per-event T5 evidence (proof + verification result) | `NNN ✓ VALID` / `NNN ✗ INVALID` formatting — see §Pattern 3 |
| UI-05 | Executive summary counts extend to Tier 4 and Tier 5 | Two new `COUNT(DISTINCT session_id)` subqueries in `query_report_summary` — see §Pattern 4 |

## Standard Stack

### Core (already installed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ratatui` | 0.30.0 | TUI widgets (`Table`, `Paragraph`, `Cell`, `Layout`, `Block`) | Already in use; no upgrade needed `[VERIFIED: Cargo.toml:25]` |
| `crossterm` | 0.29 | Terminal event stream + raw mode | Already paired with ratatui 0.30 `[VERIFIED: Cargo.toml:26]` |
| `rusqlite` | 0.37 bundled | Sync SQLite access for `generate_report` path | Already in use; `Option<T>` support for NULL is built-in `[VERIFIED: Cargo.toml:13]` |
| `tokio-rusqlite` | 0.7 | Async SQLite wrapper for TUI attach-mode DB polling | Already in use `[VERIFIED: Cargo.toml:22]` |
| `chrono` | 0.4 | Timestamp formatting in the report | Already in use in `src/report/mod.rs` `[VERIFIED: Cargo.toml:28]` |

### No new dependencies needed

Every dependency required by Phase 14 is already in `Cargo.toml`. No `cargo add` or `[dependencies]` edits needed. `[VERIFIED: Cargo.toml read in full]`

### Version verification

`ratatui = "0.30"` locked at `0.30.0` in Cargo.lock (1519–1520). `[VERIFIED: Cargo.lock]`

## Architecture Patterns

### System Data Flow Diagram

```
┌──────────────────────┐
│ Phase 13 server      │
│ /cb/v4/ & /cb/v5/    │  (already shipped)
│ handlers             │
└──────────┬───────────┘
           │ RawCallbackEvent with
           │ t4_capability / t5_proof /
           │ t5_proof_valid Option<_>
           ▼
┌──────────────────────┐
│ broker::broker_task  │  (already shipped — tier-agnostic)
└──────────┬───────────┘
           │ AppEvent with T4/T5 fields
           ▼
┌──────────────────────┐     ┌─────────────────────────┐
│ broadcast::Receiver  │────▶│ DB writer task          │
└──────────┬───────────┘     │ store::insert_callback  │  (already shipped)
           │                 │ writes t4_capability,   │
           │                 │ t5_proof, t5_proof_valid│
           │                 │ to events table         │
           │                 └────────────┬────────────┘
           │                              │
           │              ┌───────────────┘
           │              │
           ▼              ▼
┌──────────────────────┐  ┌──────────────────────────────┐
│ Monitor TUI          │  │ report::generate_report      │
│ (integrated mode)    │  │ queries events via           │
│                      │  │ query_report_summary/sessions│
│                      │  │                              │
│  ┌────────────────┐  │  │  ┌────────────────────────┐  │
│  │ AppState       │  │  │  │ Executive Summary      │  │
│  │  .events       │  │  │  │   + Tier 4 row (NEW)   │  │
│  │  .filter       │  │  │  │   + Tier 5 row (NEW)   │  │
│  │  .table_state  │  │  │  └────────────────────────┘  │
│  └────────┬───────┘  │  │                              │
│           │          │  │  ┌────────────────────────┐  │
│           ▼          │  │  │ Evidence Table         │  │
│  ┌────────────────┐  │  │  │   + Evidence col (NEW) │  │
│  │ render()       │  │  │  └────────────────────────┘  │
│  │  [A] stats     │  │  │                              │
│  │  [B] filter bar│  │  │  ┌────────────────────────┐  │
│  │  [C] event tbl │──┼──┼─▶│ Known Crawler Sessions │  │
│  │      + EVID col│  │  │  │   + Evidence col (NEW) │  │
│  │  [D] DETAIL    │  │  │  └────────────────────────┘  │
│  │      PANE(NEW) │  │  └──────────────────────────────┘
│  │  [E] hint bar  │  │
│  └────────────────┘  │
└──────────────────────┘

            NEW additions in Phase 14:
            - EVIDENCE column in [C]
            - Detail pane [D] between table and hint
            - Tier-4/5 filter cycle extension
            - Help overlay text updates
            - Exec summary tier-4/5 rows
            - Evidence column in both Markdown tables
```

### Pattern 1: Per-Cell Styling in Ratatui Table (for T5 ✓/✗ coloring)

**What:** A `Row` in a ratatui `Table` can have a row-level `Style` (used today for `Modifier::DIM` on replays), and *each* `Cell` can additionally carry its own `Style`. Per-cell styles layer on top of the row style — they do NOT override the dim modifier; they add `.fg(Color::Green)` on top. This is exactly the behavior Phase 14 needs: replay-row dim + T5-cell green/red.

**When to use:** Any event-table cell that needs its own color independent of the row.

**Example (the EVIDENCE column cell-builder helper):**

```rust
// Source: src/monitor/mod.rs — extend existing cell-building pattern at line 337
// Verified against ratatui 0.30.0 widgets::Cell API (Cargo.lock:1519)

fn tier_evidence_cell(ev: &AppEvent) -> Cell<'static> {
    match ev.tier {
        4 => {
            // D-14-03: truncate with `…` to fit EVIDENCE column width
            let full = ev.t4_capability.as_deref().unwrap_or("—");
            let shown = truncate_str(full, 20); // recommended width (see Pitfall 4)
            Cell::from(shown).style(Style::default().fg(tier_color(4)))
        }
        5 => {
            // D-14-04: NNN ✓ green / NNN ✗ red
            let proof = ev.t5_proof.as_deref().unwrap_or("---");
            match ev.t5_proof_valid {
                Some(true) => Cell::from(format!("{} ✓", proof))
                    .style(Style::default().fg(Color::Green)),
                Some(false) => Cell::from(format!("{} ✗", proof))
                    .style(Style::default().fg(Color::Red)),
                None => Cell::from(format!("{} ?", proof))
                    .style(Style::default().fg(Color::DarkGray)),
            }
        }
        _ => {
            // D-14-13: em-dash for T1–T3
            Cell::from("—").style(Style::default().fg(Color::DarkGray))
        }
    }
}
```

**Integration point in `render_event_table`:**

```rust
// Source: src/monitor/mod.rs:270-280 (widths array)
// CURRENT:
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
// Sum of fixed columns: 12+4+12+16+10+10+5+6 = 75 chars.
// 80-col minimum - 75 = 5 chars for UA + borders. Adding EVIDENCE means
// UA can go as low as Fill(1) which ratatui treats as "whatever is left
// after fixed + 1 unit", and the new fixed column displaces it proportionally.

// NEW (recommended — EVIDENCE between FIRES and REPLAY for visual grouping):
let widths = [
    Constraint::Length(12), // TIME
    Constraint::Length(4),  // TIER
    Constraint::Length(12), // CLASS
    Constraint::Length(16), // SOURCE IP
    Constraint::Fill(1),    // UA (fills remaining)
    Constraint::Length(10), // NONCE
    Constraint::Length(10), // SESS
    Constraint::Length(5),  // FIRES
    Constraint::Length(20), // EVIDENCE (NEW — 20 chars fits "web_search,browse_p…" or "123 ✗")
    Constraint::Length(6),  // REPLAY
];
// New fixed budget: 75 + 20 = 95 chars. At 80-col terminal, Fill(1) on UA
// collapses to 1 char (ratatui will wrap/clip) — document this as a known
// tradeoff per D-14-12 + Claude's Discretion on column trimming.
```

**Confidence:** HIGH `[VERIFIED: ratatui 0.30 widgets/table Cell API; existing codebase at src/monitor/mod.rs:337-347 already uses this pattern for tier_color and class_color]`

### Pattern 2: Always-Visible Detail Pane via Layout::vertical

**What:** Add one more `Constraint::Length(N)` to the existing vertical layout at `src/monitor/mod.rs:376-382`, between the event table and the hint bar. Render a `Paragraph` into that area with content driven by `app.table_state.selected()`.

**When to use:** Any auxiliary always-visible context panel that reacts to table selection.

**Layout constraint math:**

```rust
// Source: src/monitor/mod.rs:376-382
// CURRENT:
let chunks = Layout::vertical([
    Constraint::Length(3), // stats header
    Constraint::Length(3), // filter bar
    Constraint::Fill(1),   // event table
    Constraint::Length(1), // key hint bar
])
.split(frame.area());

// NEW (detail pane height = 4 lines: border top + 2 content lines + border bottom):
let chunks = Layout::vertical([
    Constraint::Length(3), // stats header
    Constraint::Length(3), // filter bar
    Constraint::Fill(1),   // event table (shrinks to accommodate pane)
    Constraint::Length(4), // detail pane (NEW — bordered, 2-line content area)
    Constraint::Length(1), // key hint bar
])
.split(frame.area());

// 20-line minimum terminal check at src/monitor/mod.rs:368 still passes:
// 3 + 3 + N + 4 + 1 = 11 + N, so N >= 9 for 20-line min. Event table
// still has 9+ lines of scroll area at minimum — acceptable.
```

**Recommended detail-pane renderer:**

```rust
// New function in src/monitor/mod.rs — sibling to render_event_table
fn render_detail_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let visible = app.visible_events();
    let selected_idx = app.table_state.selected().unwrap_or(0);
    let detail_lines: Vec<Line<'static>> = match visible.get(selected_idx) {
        None => vec![Line::from(Span::styled(
            "(no selection)",
            Style::default().fg(Color::DarkGray),
        ))],
        Some(ev) => match ev.tier {
            4 => {
                let caps = ev.t4_capability.as_deref().unwrap_or("(missing)");
                vec![
                    Line::from(vec![
                        Span::styled("T4 capabilities: ", Style::default()
                            .fg(tier_color(4))
                            .add_modifier(Modifier::BOLD)),
                        Span::raw(caps.to_string()),
                    ]),
                    Line::from(vec![
                        Span::styled("payload: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(ev.payload_id.clone()),
                    ]),
                ]
            }
            5 => {
                let proof = ev.t5_proof.as_deref().unwrap_or("---");
                let (glyph, label, color) = match ev.t5_proof_valid {
                    Some(true) => ("✓", "VALID", Color::Green),
                    Some(false) => ("✗", "INVALID", Color::Red),
                    None => ("?", "(unverified)", Color::DarkGray),
                };
                let formula_line = match ev.t5_formula {
                    Some(f) => format!("formula=(seed+{})*{} % {}", f.a, f.b, f.modulus),
                    None => "formula=(unavailable — legacy db)".to_string(),
                };
                vec![
                    Line::from(vec![
                        Span::styled("T5 proof: ", Style::default()
                            .fg(tier_color(5))
                            .add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{} ", proof)),
                        Span::styled(glyph.to_string(), Style::default().fg(color)),
                        Span::raw(" "),
                        Span::styled(label.to_string(), Style::default().fg(color)),
                    ]),
                    Line::from(Span::styled(
                        formula_line,
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            }
            _ => vec![
                Line::from(vec![
                    Span::styled("payload: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(ev.payload_id.clone()),
                    Span::raw("   "),
                    Span::styled("loc: ", Style::default().fg(Color::DarkGray)),
                    Span::raw(ev.embedding_loc.clone()),
                ]),
                Line::from(vec![
                    Span::styled("nonce: ", Style::default().fg(Color::DarkGray)),
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
```

**T4 wrapping:** T4 capability is up to 256 chars (server sanitization at D-13-09). At 78-col content width (80 - 2 border), that's up to 4 wrapped lines. The 2-line content area will clip the tail — acceptable tradeoff per D-14-01 ("detail pane for full evidence" — within a bounded pane). If clipping is observed in practice, bump `Constraint::Length(4)` → `Length(6)`. `[VERIFIED: Paragraph::wrap(Wrap { trim: bool }) at ratatui-widgets-0.3.0/src/paragraph.rs:123]`

**Confidence:** HIGH `[VERIFIED: Layout::vertical API already in use at src/monitor/mod.rs:376; Paragraph::wrap confirmed in ratatui 0.30 source]`

### Pattern 3: SQL Aggregation of Nullable T4/T5 Columns

**What:** SQLite `MAX(col)` across a `GROUP BY` returns `NULL` if and only if every row in the group has `NULL` for that column. This is the SQL standard and SQLite honors it. Combined with D-13-19 (first-write-wins replay semantics), `MAX(t4_capability)` within a `GROUP BY session_id, tier` returns the single non-null value for T4 sessions, and `NULL` for T1–T3 sessions — exactly what the report needs. `[CITED: SQLite docs on aggregate functions — MAX returns NULL for empty or all-NULL groups]`

**When to use:** Any column where replay semantics guarantee the first non-null write is canonical.

**Example (extended `query_report_sessions`):**

```rust
// Source: src/store/mod.rs:288-322
// CURRENT SQL (abbreviated):
"SELECT session_id, tier, payload_id, embedding_loc,
        MIN(first_seen_at) as first_seen_at,
        MAX(last_seen_at) as last_seen_at,
        SUM(fire_count) as total_fires,
        MAX(remote_addr) as remote_addr,
        MAX(user_agent) as user_agent,
        MAX(extra_headers) as extra_headers
 FROM events
 GROUP BY session_id, tier
 ORDER BY MIN(first_seen_at) DESC"

// NEW SQL (3 additional aggregations):
"SELECT session_id, tier, payload_id, embedding_loc,
        MIN(first_seen_at) as first_seen_at,
        MAX(last_seen_at) as last_seen_at,
        SUM(fire_count) as total_fires,
        MAX(remote_addr) as remote_addr,
        MAX(user_agent) as user_agent,
        MAX(extra_headers) as extra_headers,
        MAX(t4_capability) as t4_capability,
        MAX(t5_proof) as t5_proof,
        MAX(t5_proof_valid) as t5_proof_valid
 FROM events
 GROUP BY session_id, tier
 ORDER BY MIN(first_seen_at) DESC"

// Extended ReportSession:
pub struct ReportSession {
    pub session_id: String,
    pub tier: u8,
    pub payload_id: String,
    pub embedding_loc: String,
    pub first_seen_at: String,
    pub last_seen_at: String,
    pub fire_count: u32,
    pub remote_addr: String,
    pub user_agent: String,
    pub classification: String,
    // NEW (D-14-14: NULL-safe via Option<T>):
    pub t4_capability: Option<String>,
    pub t5_proof: Option<String>,
    pub t5_proof_valid: Option<bool>,
}

// Extended query_map closure — rusqlite Row::get<_, Option<T>> returns
// None for NULL, Some(value) otherwise. This is NULL-safe by construction.
Ok(ReportSession {
    session_id: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
    tier: row.get::<_, u8>(1)?,
    payload_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
    embedding_loc: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
    first_seen_at: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
    last_seen_at: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
    fire_count: row.get::<_, u32>(6)?,
    remote_addr: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
    user_agent: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
    classification: parse_classification(row.get::<_, Option<String>>(9)?.as_deref()),
    t4_capability: row.get::<_, Option<String>>(10)?,
    t5_proof: row.get::<_, Option<String>>(11)?,
    // SQLite stores bool as INTEGER 0/1; Option<bool> maps directly.
    t5_proof_valid: row.get::<_, Option<bool>>(12)?,
})
```

**Why `MAX(t5_proof_valid)` on an integer is safe:** `t5_proof_valid` is `0` or `1` (or NULL). `MAX` returns `1` if any row in the group has `1`, else `0`, else `NULL`. Per D-13-19 first-write-wins, there is only one non-null value in any (session_id, tier=5) group, so `MAX` returns exactly that value. For (session_id, tier≠5) groups, all values are NULL and `MAX` returns NULL.

**Markdown rendering (in `src/report/mod.rs`):**

```rust
// New helper — returns the Evidence column cell for a ReportSession
fn evidence_cell(s: &ReportSession) -> String {
    match s.tier {
        4 => {
            s.t4_capability
                .as_deref()
                .map(md_escape)
                .unwrap_or_else(|| "—".to_string())
        }
        5 => {
            let proof = s.t5_proof.as_deref().unwrap_or("---");
            match s.t5_proof_valid {
                Some(true) => format!("{} ✓ VALID", md_escape(proof)),
                Some(false) => format!("{} ✗ INVALID", md_escape(proof)),
                None => md_escape(proof).to_string(),
            }
        }
        _ => "—".to_string(), // D-14-13
    }
}

// Extended Evidence Table header (column ORDER — Evidence between Classification
// and Payload so proof-level flows visually into evidence):
md.push_str("| Session | Tier | Proof Level | First Seen | Source IP | User Agent | Fire Count | Classification | Evidence | Payload |\n");
md.push_str("|---------|------|-------------|------------|-----------|------------|------------|----------------|----------|--------|\n");

// Extended row format (inside existing for loop):
let evidence = evidence_cell(s);
md.push_str(&format!(
    "| {session_short} | {tier_str} | {proof} | {first_seen} | {ip} | {ua} | {fire} | {class} | {evidence} | {payload} |\n"
));

// Empty-state row — extend the `| — | ... |` pattern to match new column count:
md.push_str("| — | — | — | — | — | — | — | — | — | — |\n");
```

**Confidence:** HIGH `[VERIFIED: rusqlite 0.37 Row::get<Option<T>> documented behavior; existing codebase already uses this pattern at src/store/mod.rs:307-317 for nullable TEXT columns]`

### Pattern 4: Extended Executive Summary

**What:** Clone the existing `tier1_sessions` / `tier2_sessions` / `tier3_sessions` pattern for T4 and T5. No schema surprises.

**Example:**

```rust
// Source: src/store/mod.rs:196-205 — extend ReportSummary
#[derive(serde::Serialize)]
pub struct ReportSummary {
    pub total_sessions: u32,
    pub detection_sessions: u32,
    pub crawler_sessions: u32,
    pub tier1_sessions: u32,
    pub tier2_sessions: u32,
    pub tier3_sessions: u32,
    // NEW (Claude's Discretion per CONTEXT.md — recommended):
    pub tier4_sessions: u32,
    pub tier5_sessions: u32,
    pub earliest_event: Option<String>,
    pub latest_event: Option<String>,
}

// Source: src/store/mod.rs:245-264 — add two parallel subqueries
let tier4_sessions: u32 = conn.query_row(
    "SELECT COUNT(DISTINCT session_id) FROM events
     WHERE tier = 4 AND extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
    [],
    |row| row.get(0),
)?;

let tier5_sessions: u32 = conn.query_row(
    "SELECT COUNT(DISTINCT session_id) FROM events
     WHERE tier = 5 AND extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
    [],
    |row| row.get(0),
)?;
```

**Markdown (in `src/report/mod.rs:90-101`):**

```rust
// Extended Executive Summary — D-14-12 always-show chrome, 5 tier rows:
md.push_str(&format!("| Tier 1 (Arbitrary Callback) | {} |\n", summary.tier1_sessions));
md.push_str(&format!("| Tier 2 (Conditional Branch) | {} |\n", summary.tier2_sessions));
md.push_str(&format!("| Tier 3 (Computed Callback) | {} |\n", summary.tier3_sessions));
md.push_str(&format!("| Tier 4 (Capability Introspection) | {} |\n", summary.tier4_sessions));
md.push_str(&format!("| Tier 5 (Multi-step Compliance) | {} |\n", summary.tier5_sessions));
md.push_str(&format!("| Known Crawler Sessions | {} |\n", summary.crawler_sessions));
```

**Confidence:** HIGH `[VERIFIED: existing codebase pattern at src/store/mod.rs:245-264 and src/report/mod.rs:90-101]`

### Pattern 5: T5 Formula Propagation to AppEvent

**What:** Add `t5_formula: Option<T5Formula>` to `RawCallbackEvent` and `AppEvent`. Populate in the T5 handler (where formula is already in scope at `src/server/mod.rs:160`); leave `None` in T1/T2/T3/T4 handlers and in attach-mode DB loading. Broker propagates it identically to `t4_capability` (see `src/broker/mod.rs:32-34`).

**Why not cache catalog in `AppState`:** Attach mode (`monitor --attach`) reads events from a DB that has no callback-map.json context available. Forcing `AppState` to load the catalog couples monitor startup to the project's `honeyprompt.toml` existence, which isn't available for arbitrary DB files. Propagating via `AppEvent` makes attach mode gracefully handle "formula unknown" as `None` → detail pane shows `formula=(unavailable — legacy db)`.

**Integration (minimal edits):**

```rust
// src/types.rs:89-106 — extend RawCallbackEvent
pub struct RawCallbackEvent {
    // ... existing fields ...
    pub t4_capability: Option<String>,
    pub t5_proof: Option<String>,
    pub t5_proof_valid: Option<bool>,
    // NEW (Phase 14):
    pub t5_formula: Option<T5Formula>,
}

// src/types.rs:110-127 — extend AppEvent
pub struct AppEvent {
    // ... existing fields ...
    pub t4_capability: Option<String>,
    pub t5_proof: Option<String>,
    pub t5_proof_valid: Option<bool>,
    // NEW (Phase 14):
    pub t5_formula: Option<T5Formula>,
}

// src/server/mod.rs:182 — populate in t5_callback_handler
let event = RawCallbackEvent {
    // ... existing fields ...
    t4_capability: None,
    t5_proof: Some(proof_str),
    t5_proof_valid: Some(proof_valid),
    t5_formula: Some(*formula), // NEW — formula already in scope at line 160
};

// src/broker/mod.rs:32-34 — extend the propagation
t4_capability: raw.t4_capability,
t5_proof: raw.t5_proof,
t5_proof_valid: raw.t5_proof_valid,
t5_formula: raw.t5_formula, // NEW

// Attach-mode AppEvent construction at src/monitor/mod.rs:811-827:
// Leave t5_formula: None — attach mode doesn't have catalog loaded;
// detail pane falls back to "formula=(unavailable — legacy db)".
```

**Confidence:** HIGH `[VERIFIED: src/server/mod.rs:144-196 handler already has formula in scope; src/broker/mod.rs:32-34 already propagates 3 other Option<> fields the same way]`

### Anti-Patterns to Avoid

- **Adding an `Option<&Catalog>` field to `AppState`:** Couples attach mode to project-directory assumptions; more state than needed; rejected in favor of Pattern 5.
- **Computing the T5 formula string at render time from `payload_id`:** Requires synchronous catalog access in the render hot path; couples monitor rendering to catalog crate; rejected in favor of Pattern 5 pre-population.
- **`if summary.tier4_sessions > 0 { md.push_str(...) }` conditional row rendering:** Violates D-14-12 always-show chrome. Always emit the row with `{}` count, even when zero.
- **Extending `TierFilter` without updating `cycle_filter` tests:** `test_handle_filter_cycle` at `src/monitor/mod.rs:1184-1194` exhaustively checks the 4-state cycle. Extending to 6 states without updating the test will silently pass on incomplete implementations.
- **Assuming `Constraint::Fill(1)` gracefully handles 80-col minimum with new fixed column:** 75 + 20 = 95 > 80. Plan must either document this as degraded-but-functional (UA clipped) or reduce column count. Recommend documenting as acceptable — the "terminal too small" guard at `src/monitor/mod.rs:368` is the escape hatch.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| NULL column handling in Markdown rendering | Custom `is_null` tracking in query logic | `rusqlite::Row::get::<_, Option<T>>` | Already NULL-safe; aligns with existing code at lines 304-315 |
| Conditional SQL column selection ("is this a v4 or v5 DB?") | `PRAGMA table_info` probe before SELECT | Unconditional SELECT + Option<T> | D-14-14 locked; Phase 13 migration guarantee (D-13-17) makes the column always present |
| Detail pane text wrapping | Manual `char_indices` wrap loop | `Paragraph::wrap(Wrap { trim: false })` | Built into ratatui 0.30; handles Unicode widths correctly |
| Per-tier color coordination | Scattered `match ev.tier { 4 => Color::X, ... }` at each call site | Central `tier_color(u8) -> Color` at src/monitor/mod.rs:248 | Already exists — extend match arms in one place |
| T5 proof validity → glyph mapping | Scattered `if proof_valid { "✓" } else { "✗" }` | Small helper `fn validity_glyph(v: Option<bool>) -> (&'static str, Color)` | Centralizes D-14-04 convention for reuse in both TUI and Markdown |
| Session-grouped aggregation | Manual `BTreeMap<(session, tier), Vec<Row>>` in application code | SQL `GROUP BY session_id, tier` with `MAX(col)` | Leverages SQLite; already the existing pattern; `MAX` on first-write-wins columns is canonical |

**Key insight:** Every Phase 14 change is an *extension* of an existing pattern — no new problem domains. The discipline is: find the existing pattern for T1–T3, extend it symmetrically, update its test.

## Common Pitfalls

### Pitfall 1: `MAX(col)` over all-NULL group returns NULL — but the test must prove it

**What goes wrong:** Developer assumes `MAX(t4_capability)` on a T1–T3-only row group returns empty string; report cell ends up rendering "null" or panicking on `unwrap()`.

**Why it happens:** SQLite aggregate semantics differ from SUM (which returns 0 for empty/NULL groups); developers reach for wrong mental model.

**How to avoid:** Declare `t4_capability: Option<String>` on `ReportSession`; use `row.get::<_, Option<String>>(N)`; let rusqlite map NULL → None idiomatically.

**Warning signs:** Any `row.get::<_, String>(N)` on the new columns — will panic on NULL at runtime for legacy DBs. `[VERIFIED: rusqlite Row::get docs — mapping NULL to String returns Err(InvalidColumnType)]`

### Pitfall 2: Ratatui Row::style vs Cell::style precedence

**What goes wrong:** Developer styles the row with `Modifier::DIM` for replays (existing pattern), then styles cells with `Color::Green` — expects both to combine, but discovers DIM is only applied in some rendering paths.

**Why it happens:** ratatui layers row style first, then cell style on top; the composition is well-defined but non-obvious. Modifier::DIM affects intensity; Color::Green is still rendered green but dimmed.

**How to avoid:** Verify empirically with a quick render snapshot. Per §Pattern 1, the layering is well-behaved — replay rows render T5 cells as "dimmed green" / "dimmed red" which is actually correct defender UX (replay = historical, validity signal still visible).

**Warning signs:** Visual regression on "replay T5 is invisible" — if observed, set the row style to `Style::default()` for T5-bearing rows or use `.patch_style()` at cell level.

### Pitfall 3: Attach mode has no catalog — detail pane must not panic

**What goes wrong:** Detail pane `render_detail_pane` unconditionally dereferences `ev.t5_formula`; monitor `--attach` loads from an arbitrary DB with `t5_formula: None`; unwrap panics.

**Why it happens:** Developer codes the integrated-mode happy path and forgets attach mode is a second entry point.

**How to avoid:** All T5 formula access in the detail pane MUST pattern-match on `Option<T5Formula>`. Fallback text: `formula=(unavailable — legacy db)` (see §Pattern 2).

**Warning signs:** Test with a fresh v4.0-style DB (no T5 rows) — detail pane must render without panic. Add a test `test_detail_pane_attach_mode_no_formula`.

### Pitfall 4: EVIDENCE column width at 80-col terminal

**What goes wrong:** `Constraint::Length(20)` for EVIDENCE + `Constraint::Fill(1)` for UA means UA is clipped to ~1 char at 80-col terminals; users with narrow terminals see unreadable UA.

**Why it happens:** Sum of fixed widths + 1-char minimum Fill doesn't fit in 80 cols.

**How to avoid:** Accept the tradeoff — D-14-12 says chrome is always-shown; UA is already `truncate_str(_, 30)`-ed. Users with tighter terminals see terminal-too-small guard at width < 80 (existing behavior at `src/monitor/mod.rs:368`). Recommendation: document the 80-col minimum as "UA may be aggressively truncated" in help overlay or as a TODO for a future `--narrow` mode.

**Alternative (if planner prefers):** Shrink EVIDENCE to `Length(16)` — fits `"web_search,brows…"` (13 chars + `…`) or `"123 ✓"`. Recommended as primary.

**Warning signs:** Manual test at exactly 80 cols — if UA shows just a single char or is truncated mid-letter, consider reducing EVIDENCE to 16.

### Pitfall 5: Command parser silently accepting `:filter t4` after extension, tests stale

**What goes wrong:** Developer adds match arms for `"filter t4"` / `"filter t5"` but leaves the test `test_handle_filter_cycle` at 4-state cycle; test passes because the cycle still works for the first 4 states, but the 2 new states are untested and a typo goes undetected.

**Why it happens:** TierFilter is an enum — adding variants doesn't break compile, but stale tests don't catch logic errors.

**How to avoid:** Require `test_handle_filter_cycle` to assert exactly 6 states in the cycle (All→T1→T2→T3→T4→T5→All) with assertion on each transition. Test file: `src/monitor/mod.rs:1184-1194`.

**Warning signs:** Adding new enum variants to `TierFilter` without editing the existing cycle test.

### Pitfall 6: Report column count mismatch in Markdown

**What goes wrong:** Developer adds Evidence column to header but forgets to add it to the empty-state row `| — | — | ... |` — Markdown table renders with misaligned columns and rendering libraries choke.

**Why it happens:** Two string-literal locations (header + empty-state) drift apart.

**How to avoid:** Consolidate header + empty-state row count via a small helper or a const:

```rust
const EVIDENCE_COL_COUNT: usize = 10; // was 9 in Phase 13
const EVIDENCE_EMPTY_ROW: &str = "| — | — | — | — | — | — | — | — | — | — |\n";
```

Or verify visually that both locations have the same pipe count.

**Warning signs:** Any Markdown rendering test that parses the table — column-count mismatch breaks it. Add a test: `assert_eq!(header_line.matches('|').count(), empty_row.matches('|').count())`.

## Runtime State Inventory

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | Phase 13 SQLite columns `t4_capability`, `t5_proof`, `t5_proof_valid` already present on `events` table (`src/store/mod.rs:60-62`). No new columns in Phase 14. | None — Phase 14 only reads existing columns |
| Live service config | None — Phase 14 is purely code-local. No external service configuration (n8n, Datadog, etc.) depends on Phase 14 output. | None |
| OS-registered state | None — Phase 14 does not register any OS-level services, tasks, or daemons. | None |
| Secrets/env vars | None — no new env vars introduced by Phase 14. Reuses existing `honeyprompt.toml` config. | None |
| Build artifacts | None — no build tooling changes. Existing `cargo build` produces updated binary with Phase 14 code. | Binary rebuild on release (expected) |

**Nothing found in category:** All categories confirmed explicitly above. Phase 14 is purely code-local with no runtime-state migration burden.

## Code Examples

### Complete EVIDENCE cell builder (ready to paste)

```rust
// Source: new helper in src/monitor/mod.rs near tier_color() at line 248
fn tier_evidence_cell(ev: &AppEvent) -> Cell<'static> {
    match ev.tier {
        4 => {
            let full = ev.t4_capability.as_deref().unwrap_or("—");
            Cell::from(truncate_str(full, 20))
                .style(Style::default().fg(tier_color(4)))
        }
        5 => {
            let proof = ev.t5_proof.as_deref().unwrap_or("---");
            match ev.t5_proof_valid {
                Some(true) => Cell::from(format!("{} ✓", proof))
                    .style(Style::default().fg(Color::Green)),
                Some(false) => Cell::from(format!("{} ✗", proof))
                    .style(Style::default().fg(Color::Red)),
                None => Cell::from(format!("{} ?", proof))
                    .style(Style::default().fg(Color::DarkGray)),
            }
        }
        _ => Cell::from("—").style(Style::default().fg(Color::DarkGray)),
    }
}
```

### Complete `proof_level` extension

```rust
// Source: replace src/report/mod.rs:38-45
fn proof_level(tier: u8) -> &'static str {
    match tier {
        1 => "Arbitrary Callback",
        2 => "Conditional Branch",
        3 => "Computed Callback",
        4 => "Capability Introspection",  // D-14-09
        5 => "Multi-step Compliance",      // D-14-09
        _ => "Unknown",
    }
}
```

### Complete `tier_color` extension

```rust
// Source: replace src/monitor/mod.rs:248-255
fn tier_color(tier: u8) -> Color {
    match tier {
        1 => Color::Cyan,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Magenta,    // D-14-05 (Claude's Discretion)
        5 => Color::LightBlue,  // D-14-05 (Claude's Discretion; distinct from T1 Cyan)
        _ => Color::White,
    }
}
```

### Complete `TierFilter` extension

```rust
// Source: replace src/monitor/mod.rs:25-42
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TierFilter {
    All,
    T1,
    T2,
    T3,
    T4,  // NEW (D-14-06)
    T5,  // NEW (D-14-06)
}

impl TierFilter {
    pub fn next(self) -> Self {
        match self {
            TierFilter::All => TierFilter::T1,
            TierFilter::T1 => TierFilter::T2,
            TierFilter::T2 => TierFilter::T3,
            TierFilter::T3 => TierFilter::T4,  // NEW
            TierFilter::T4 => TierFilter::T5,  // NEW
            TierFilter::T5 => TierFilter::All, // NEW
        }
    }
}
```

### Complete `visible_events` filter extension

```rust
// Source: replace src/monitor/mod.rs:123-129
match self.filter {
    TierFilter::All => true,
    TierFilter::T1 => e.tier == 1,
    TierFilter::T2 => e.tier == 2,
    TierFilter::T3 => e.tier == 3,
    TierFilter::T4 => e.tier == 4,  // NEW
    TierFilter::T5 => e.tier == 5,  // NEW
}
```

### Complete `tier_counts` refactor

```rust
// Source: replace src/monitor/mod.rs:172-178
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

// Callers at src/monitor/mod.rs:385 — update:
let counts = app.tier_counts();
let (t1, t2, t3, t4, t5) = (counts[0], counts[1], counts[2], counts[3], counts[4]);
// (or just index counts[0..5] directly in the stats_spans literal)
```

### Complete command parser extension

```rust
// Source: append to match in src/monitor/mod.rs:672-705
"filter t4" => {
    app.filter = TierFilter::T4;
    app.table_state.select(Some(0));
}
"filter t5" => {
    app.filter = TierFilter::T5;
    app.table_state.select(Some(0));
}
```

### Complete help overlay extension

```rust
// Source: replace src/monitor/mod.rs:559 (Tab description)
Line::from("  Tab         Cycle filter: All -> T1 -> T2 -> T3 -> T4 -> T5"),

// Source: insert after src/monitor/mod.rs:572 (existing `filter all|t1|t2|t3`)
Line::from("  filter all|t1|t2|t3|t4|t5"),

// Source: insert new help line after D-14-07a detail pane documentation
Line::from(""),
Line::from(vec![Span::styled(
    "Detail Pane",
    Style::default().add_modifier(Modifier::BOLD),
)]),
Line::from("  The pane below the event table shows full context"),
Line::from("  for the selected row (T4 capabilities, T5 proof+formula,"),
Line::from("  T1-T3 payload_id + embedding_loc + full nonce)."),
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `tier_counts` returning `(usize, usize, usize)` tuple | `[usize; 5]` array | Phase 14 | Symmetric indexing, future tier extensions don't change arity |
| Executive Summary conditionally emitting tier rows | Always-show 5 tier rows with zero-counts visible | Phase 14 (D-14-12) | Stable report structure; downstream parsers see consistent schema |
| `PRAGMA table_info` schema probing | Unconditional SELECT + Option<T> | Phase 14 (D-14-14) | Leverages Phase 13 migration guarantee; simpler code |
| Monitor TUI detail-on-hover-only | Always-visible detail pane | Phase 14 (D-14-01) | Discoverability over density for a defender-facing tool |

**Deprecated/outdated:** None — Phase 14 is purely additive.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `Constraint::Length(20)` for EVIDENCE gracefully clips UA at 80-col terminals rather than breaking the layout | Pattern 1 / Pitfall 4 | Visual glitch at exactly 80 cols; fixed by reducing EVIDENCE to 16 or trimming UA more aggressively. Confirmed empirically by planner or discovered in Wave 0 smoke test. |
| A2 | `Paragraph::wrap(Wrap { trim: false })` in a 2-line bounded area clips tail gracefully rather than panicking | Pattern 2 | Tail clipping on 256-char T4 cap at narrow terminal; fixed by bumping pane height to 6 if needed. |
| A3 | Magenta (T4) and LightBlue (T5) are readable on common terminal color schemes | tier_color extension | Color invisible on some monochrome terminals; overridable by extending the discretion choice. |
| A4 | Row-level `Modifier::DIM` on replay rows composes correctly with per-cell `Color::Green/Red` such that the validity glyph remains discernible | Pitfall 2 | Replay T5 cells unreadable; mitigated by only applying row DIM to non-evidence cells or dropping DIM on T5 rows. |

**If A1 or A2 are wrong, plan includes a Wave 0 task:** "visual smoke test at 80×20 terminal, adjust constants based on screenshot."

## Open Questions

1. **Column ordering in Markdown: Evidence before or after Payload?**
   - What we know: CONTEXT.md leaves this to Claude's Discretion.
   - What's unclear: whether readers scan "proof level → evidence" (Evidence right after Classification) or "payload → evidence" (Evidence right after Payload).
   - Recommendation: Place Evidence immediately after Classification (column 9 of 10), so the narrative reads "classification → evidence for that tier → payload id". This lets T4/T5 cells sit adjacent to the tier label and reads naturally in a monospace report.

2. **Does rendering ✓ / ✗ Unicode glyphs produce usable output in typical defender workflows (GitHub Markdown, email, print)?**
   - What we know: GitHub and most terminals render ✓ (U+2713) and ✗ (U+2717) correctly.
   - What's unclear: ASCII-only fallback for paste-into-text reports.
   - Recommendation: Stick with ✓ / ✗ per D-14-04. If fallback needed later, add a `--ascii-evidence` flag — out of scope for Phase 14.

3. **Should `AppEvent.t5_formula` be `T5Formula` struct or pre-formatted string?**
   - What we know: Pattern 5 recommends the struct.
   - What's unclear: Whether storing struct vs string affects serde (the struct derives Serialize/Deserialize already per `src/types.rs:21-26`).
   - Recommendation: Struct. The formula string is a view concern; structure stays close to the data.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (stable) | Build | ✓ (assumed — Phase 13 shipped) | 2021 edition | — |
| `cargo` | Build / test | ✓ | — | — |
| `ratatui 0.30` | TUI extension | ✓ | 0.30.0 | — |
| `crossterm 0.29` | Terminal events | ✓ | — | — |
| `rusqlite 0.37` (bundled) | Report generation | ✓ | — | — |
| `tokio-rusqlite 0.7` | Monitor attach mode | ✓ | — | — |
| SQLite runtime | store / report | ✓ (rusqlite bundled) | — | — |
| Unicode-capable terminal | TUI rendering of ✓/✗ | ✓ (assumed for target platforms Linux/macOS) | — | Document 80-col/UTF-8 requirement |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None.

## Validation Architecture

> `workflow.nyquist_validation = true` confirmed in `.planning/config.json`.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` (Rust built-in) |
| Config file | None — uses `Cargo.toml` `[dev-dependencies]` |
| Quick run command | `cargo test --lib <module_filter>` (fast, no rebuild) |
| Full suite command | `cargo test --all` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UI-01 | `tier_evidence_cell` returns truncated T4 capability with Magenta style | unit | `cargo test --lib monitor::tests::test_tier_evidence_cell_t4` | Wave 0 |
| UI-02 | `tier_evidence_cell` returns `NNN ✓` green for valid T5, `NNN ✗` red for invalid | unit | `cargo test --lib monitor::tests::test_tier_evidence_cell_t5_valid` / `test_tier_evidence_cell_t5_invalid` | Wave 0 |
| UI-02 | Detail pane renders formula line for T5 events with `t5_formula: Some(_)` | unit | `cargo test --lib monitor::tests::test_render_detail_pane_t5_with_formula` | Wave 0 |
| UI-03 | `evidence_cell` returns `md_escape`-safe T4 capability | unit | `cargo test --lib report::tests::test_evidence_cell_t4` | Wave 0 |
| UI-04 | `evidence_cell` returns `NNN ✓ VALID` / `NNN ✗ INVALID` | unit | `cargo test --lib report::tests::test_evidence_cell_t5_valid` / `test_evidence_cell_t5_invalid` | Wave 0 |
| UI-05 | `query_report_summary` returns T4/T5 counts matching inserted events | integration | `cargo test --test test_report test_report_summary_tier4_tier5_counts` | Wave 0 |
| UI-01–05 | `proof_level(4)` == "Capability Introspection", `proof_level(5)` == "Multi-step Compliance" | unit | `cargo test --lib report::tests::test_proof_level_mapping` (extend existing) | ✅ extend at src/report/mod.rs:212 |
| Backward-compat #5 | v4.0-style DB (no T4/T5 rows inserted) produces report with `Tier 4 \| 0` and `Tier 5 \| 0` rows; Evidence cells all `—` | integration | `cargo test --test test_report test_report_backward_compat_v40_db` | Wave 0 |
| TierFilter cycle | 6-state cycle All→T1→T2→T3→T4→T5→All | unit | `cargo test --lib monitor::tests::test_handle_filter_cycle` (extend existing at :1184) | ✅ extend |
| Command parser | `:filter t4` / `:filter t5` set state correctly | unit | `cargo test --lib monitor::tests::test_command_filter_t4` / `test_command_filter_t5` | Wave 0 |
| tier_color T4/T5 | `tier_color(4) == Magenta`, `tier_color(5) == LightBlue` | unit | `cargo test --lib monitor::tests::test_tier_color` (extend existing at :1239) | ✅ extend |
| tier_counts arity | returns `[usize; 5]` with T4/T5 counts | unit | `cargo test --lib monitor::tests::test_tier_counts_excludes_replays` (extend existing at :1160) | ✅ extend |
| MAX(NULL) aggregation | `query_report_sessions` returns `t4_capability: None` for T1–T3 session groups | integration | `cargo test --test test_report test_report_sessions_null_t4_for_t1_row` | Wave 0 |
| Detail pane attach-mode | render_detail_pane does not panic when `t5_formula: None` | unit | `cargo test --lib monitor::tests::test_detail_pane_attach_mode_no_formula` | Wave 0 |
| End-to-end Markdown | Full report with 1 T1, 1 T2, 1 T3, 1 T4, 1 T5 event renders valid Markdown with Evidence column populated correctly per tier | integration (golden-string) | `cargo test --test test_report test_report_full_5tier_markdown` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --lib <module>` for the module being edited (monitor, report, store, types). Typical run: < 5 seconds.
- **Per wave merge:** `cargo test --all` (unit + integration). Typical run: 15–30 seconds.
- **Phase gate:** `cargo fmt --all -- --check && cargo clippy --all -- -D warnings && cargo test --all` before `/gsd-verify-work`.

### Wave 0 Gaps

- [ ] `tests/test_report.rs` — extend with T4/T5 insertion helper and 5 new integration tests (backward_compat_v40_db, summary_tier4_tier5_counts, sessions_null_t4_for_t1_row, evidence_column_present, full_5tier_markdown)
- [ ] `src/monitor/mod.rs` tests module — add `tier_evidence_cell` unit tests, `command_filter_t4`/`t5`, detail-pane tests
- [ ] `src/report/mod.rs` tests module — add `evidence_cell` unit tests, extend `test_proof_level_mapping` to cover tier 4 and 5
- [ ] `src/types.rs` tests module — extend `test_app_event_fields` and `test_raw_callback_event_fields` to cover new `t5_formula: Option<T5Formula>` field
- [ ] No new framework install needed — `cargo test` already configured; `tempfile` dev-dep already present at Cargo.toml:34

## Security Domain

> `security_enforcement` not explicitly set in config; treating as enabled by default.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | Phase 14 is purely rendering; no auth surface |
| V3 Session Management | no | No sessions; monitor is single-user local tool |
| V4 Access Control | no | Local binary; no multi-user access surface |
| V5 Input Validation | yes | `md_escape` on T4/T5 strings before Markdown interpolation; sanitization happened server-side in Phase 13 (D-13-09) |
| V6 Cryptography | no | Phase 14 does not introduce crypto; T5 proof verification happens in Phase 13 server |

### Known Threat Patterns for Rust TUI + Markdown rendering

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Markdown injection via agent-supplied payload (pipe, backtick, newline) | Tampering | `md_escape()` applied to every agent-supplied cell (existing pattern at src/report/mod.rs:11-23) — extends unchanged to `t4_capability` and `t5_proof` |
| Terminal control sequence injection via T4 capability decoded from base64 | Tampering / DoS | Server-side D-13-09 regex `^[a-z0-9_,.\-]{1,256}$` already rejects control chars; TUI renders as plain text via ratatui `Cell::from(String)` which does not interpret ANSI |
| T5 proof value containing terminal escapes | Tampering / DoS | Server-side 3-ASCII-digit constraint (src/server/mod.rs:165) already rejects anything else; render is `{proof} ✓` where proof is 3 digits |
| SQL injection via new columns | Tampering | All SQL uses `rusqlite::params!` macro (verified throughout src/store/mod.rs); no string interpolation |
| Detail pane memory leak via large T4 capability repeated selection | DoS | `AppEvent.t4_capability: Option<String>` is owned; dropped when event is dropped; no caching beyond event's natural lifetime |

## Project Constraints (from CLAUDE.md)

Extracted from `/home/john/vault/projects/github.com/honeyprompt/CLAUDE.md`:

- **Language:** Rust stable; 2021 edition
- **TUI:** Ratatui (0.30 locked in Cargo.toml)
- **HTTP:** Axum (not touched in this phase)
- **Storage:** SQLite via rusqlite + tokio-rusqlite
- **Async entry points:** Must use `tokio::runtime::Runtime::new()` in main.rs, NOT `#[tokio::main]`. Phase 14 does not touch `main.rs`.
- **Tests:** unit tests in `#[cfg(test)] mod tests` within each module; integration tests in `tests/`. Phase 14 adds tests in BOTH locations per §Validation Architecture.
- **GSD workflow:** atomic commits per task — plans should decompose edits so each task ends with `cargo fmt --check && cargo clippy -D warnings && cargo test` green.
- **CI check before push:** `fmt / clippy / test` locally before every `git push` (per `feedback_ci_checks.md` memory).

## Sources

### Primary (HIGH confidence)

- `/home/john/vault/projects/github.com/honeyprompt/src/monitor/mod.rs` — 1301-line TUI module; all integration points verified by direct read
- `/home/john/vault/projects/github.com/honeyprompt/src/report/mod.rs` — 218-line report module; full verbatim
- `/home/john/vault/projects/github.com/honeyprompt/src/store/mod.rs` — `ReportSummary`, `ReportSession`, `query_report_summary`, `query_report_sessions` read and diff'd
- `/home/john/vault/projects/github.com/honeyprompt/src/types.rs` — AppEvent, RawCallbackEvent, Tier, T5Formula confirmed
- `/home/john/vault/projects/github.com/honeyprompt/src/server/mod.rs` — NonceMeta.t5_formula, t5_callback_handler confirmed
- `/home/john/vault/projects/github.com/honeyprompt/src/broker/mod.rs` — T4/T5 propagation already shipped (Phase 13)
- `/home/john/vault/projects/github.com/honeyprompt/Cargo.toml` + `Cargo.lock` — dependency versions verified
- `/home/john/vault/projects/github.com/honeyprompt/.planning/phases/14-tiers-4-5-surfacing-monitor-tui-report/14-CONTEXT.md` — all D-14-* decisions
- `/home/john/vault/projects/github.com/honeyprompt/.planning/phases/13-tiers-4-5-backend-payloads-routes-store/13-CONTEXT.md` — all D-13-* carry-forward decisions

### Secondary (MEDIUM confidence)

- ratatui 0.30 widget API — verified via local cargo cache at `/home/john/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ratatui-widgets-0.3.0/src/paragraph.rs:72-123` for `Wrap { trim }` confirmation
- SQLite `MAX` aggregate NULL semantics — well-established SQL standard; confirmed via rusqlite `Row::get<Option<T>>` usage pattern already in `src/store/mod.rs:307-317`

### Tertiary (LOW confidence)

- None — all critical claims backed by direct code inspection in this session.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all libraries locked in Cargo.toml, no new dependencies.
- Architecture: HIGH — all integration points read directly; no speculation.
- Pitfalls: HIGH (1, 3, 5, 6) / MEDIUM (2, 4) — MEDIUM items depend on empirical terminal behavior verifiable in Wave 0 smoke test.
- Validation strategy: HIGH — test patterns mirror existing Phase 13 tests (`test_schema_t4_columns`, `test_insert_callback_event_replay_t4_first_write_wins`) which already shipped and pass.

**Research date:** 2026-04-24
**Valid until:** 2026-05-24 (30 days — stable codebase, no fast-moving dependencies touched)
