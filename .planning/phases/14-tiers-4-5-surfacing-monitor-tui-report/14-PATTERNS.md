# Phase 14: Tiers 4 & 5 Surfacing (Monitor TUI + Report) - Pattern Map

**Mapped:** 2026-04-24
**Files analyzed:** 7 (modified) + 1 (new integration test file)
**Analogs found:** 8 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/types.rs` | model | transform | Existing `AppEvent` T4/T5 Option fields at `src/types.rs:97-106, 121-127` | exact (extension) |
| `src/monitor/mod.rs` | TUI/component | event-driven render | `render_event_table`, `tier_color`, `TierFilter`, `tier_counts`, filter bar, help overlay — all in same file | exact (extension — same file) |
| `src/report/mod.rs` | service | batch transform | `generate_report`, `proof_level`, Evidence Table row loop at `src/report/mod.rs:118-141` | exact (extension — same file) |
| `src/store/mod.rs` | service | CRUD | `query_report_summary` T1/T2/T3 subqueries (`:245-264`), `query_report_sessions` GROUP BY MAX pattern (`:288-322`) | exact (extension — same file) |
| `src/server/mod.rs` | controller | request-response | `t5_callback_handler` at `:144-196` — only ~1 line addition populating `t5_formula` on `RawCallbackEvent` | exact |
| `src/broker/mod.rs` | service | event-driven | `broker_task` propagation of `t4_capability/t5_proof/t5_proof_valid` at `:18-35` | exact (one additional field) |
| `tests/test_report.rs` | test | integration | Existing `insert_event` helper + `test_report_with_events` / `test_report_session_based_counting` | exact (extension) |
| `tests/test_monitor.rs` (or monitor module tests) | test | unit | `test_handle_filter_cycle` at `src/monitor/mod.rs:1183-1194`; `test_tier_color` at `:1239-1244`; `test_tier_counts_excludes_replays` at `:1160-1171` | exact (extension) |

## Pattern Assignments

### `src/types.rs` (model, transform)

**Analog:** same file — existing Phase-13 `t4_capability: Option<String>` / `t5_proof: Option<String>` / `t5_proof_valid: Option<bool>` pattern on `RawCallbackEvent` and `AppEvent`.

**Field additions (lines 97-106 / 121-127):**

```rust
// src/types.rs:89-106 — CURRENT RawCallbackEvent
#[derive(Debug, Clone)]
pub struct RawCallbackEvent {
    pub nonce: String,
    pub tier: u8,
    pub payload_id: String,
    pub embedding_loc: String,
    pub fingerprint: AgentFingerprint,
    pub classification: AgentClass,
    pub received_at: u64,
    /// Phase 13 (T4): sanitized capability string decoded from the /cb/v4/ path.
    pub t4_capability: Option<String>,
    /// Phase 13 (T5): raw 3-digit proof string submitted on the /cb/v5/ path.
    pub t5_proof: Option<String>,
    /// Phase 13 (T5): whether `t5_proof` matched the expected value derived from
    /// `nonce::derive_seed` and the payload's `T5Formula`.
    pub t5_proof_valid: Option<bool>,
    // MIRROR THIS: add `pub t5_formula: Option<T5Formula>,` with a Phase-14 doc
    // comment explaining why the monitor detail pane needs it.
}
```

**Mirror for `AppEvent`** (same file, lines 108-127) — add identical `t5_formula: Option<T5Formula>` field with the same rustdoc style.

**T5Formula struct already derives `Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize`** (`:21-26`), so no trait work.

**Test pattern to mirror** — extend `test_raw_callback_event_fields` at `src/types.rs:180-203` and `test_app_event_fields` at `:205-234` to set and assert the new field:

```rust
// Extend existing init blocks at lines 196-200 and 226-229:
t4_capability: None,
t5_proof: None,
t5_proof_valid: None,
t5_formula: None, // NEW — mirror existing Option<_> defaults
```

---

### `src/monitor/mod.rs` (TUI/component, event-driven render)

**Analog:** same file — every needed pattern exists for T1–T3; extend match arms symmetrically.

#### TierFilter enum extension (lines 25-42)

```rust
// CURRENT
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TierFilter { All, T1, T2, T3 }

impl TierFilter {
    pub fn next(self) -> Self {
        match self {
            TierFilter::All => TierFilter::T1,
            TierFilter::T1 => TierFilter::T2,
            TierFilter::T2 => TierFilter::T3,
            TierFilter::T3 => TierFilter::All,
        }
    }
}
// MIRROR: add T4, T5 variants; extend next() to T3 -> T4 -> T5 -> All.
// MIRROR: extend visible_events match at lines 124-129 with T4/T5 arms.
```

#### `tier_color` match extension (lines 248-255)

```rust
// CURRENT
fn tier_color(tier: u8) -> Color {
    match tier {
        1 => Color::Cyan,
        2 => Color::Green,
        3 => Color::Yellow,
        _ => Color::White,
    }
}
// MIRROR: add `4 => Color::Magenta, 5 => Color::LightBlue` (Claude's Discretion per D-14-05).
```

#### `tier_counts` refactor (lines 172-178)

```rust
// CURRENT — returns 3-tuple
pub fn tier_counts(&self) -> (usize, usize, usize) {
    let non_replays: Vec<&AppEvent> = self.events.iter().filter(|e| !e.is_replay).collect();
    let t1 = non_replays.iter().filter(|e| e.tier == 1).count();
    let t2 = non_replays.iter().filter(|e| e.tier == 2).count();
    let t3 = non_replays.iter().filter(|e| e.tier == 3).count();
    (t1, t2, t3)
}
// MIRROR: refactor return type to `[usize; 5]`. Extend caller at :385.
```

#### `render_event_table` — widths/header/cell extension (lines 270-363)

Key pattern to mirror — per-cell styling with `Cell::from(x).style(...)`:

```rust
// CURRENT — tier label + class label use per-cell styling (lines 337-347)
let cells = vec![
    Cell::from(time_str),
    Cell::from(tier_str).style(Style::default().fg(tier_color(ev.tier))),
    Cell::from(truncate_str(class_str, 12)).style(Style::default().fg(class_color)),
    Cell::from(ip_str),
    Cell::from(ua_str),
    Cell::from(nonce_short),
    Cell::from(sess_short),
    Cell::from(fires_str),
    Cell::from(replay_str),
];

let row_style = if ev.is_replay {
    Style::default().add_modifier(Modifier::DIM)
} else {
    Style::default()
};
Row::new(cells).style(row_style)
// MIRROR: new `tier_evidence_cell(ev) -> Cell` helper (pattern Research §Pattern 1);
// insert Cell in cells vec between `fires_str` and `replay_str`; extend widths
// array at :270-280 with `Constraint::Length(20)` between FIRES and REPLAY;
// extend header_cells at :282-292 with "EVIDENCE".
```

#### Layout constraint extension (lines 376-382)

```rust
// CURRENT
let chunks = Layout::vertical([
    Constraint::Length(3), // stats header
    Constraint::Length(3), // filter bar
    Constraint::Fill(1),   // event table
    Constraint::Length(1), // key hint bar
])
.split(frame.area());
// MIRROR: insert `Constraint::Length(4)` (bordered pane, 2 content lines) between
// table and hint bar. Keep 80x20 minimum guard at :368 intact — 3+3+9+4+1=20 passes.
// New function `render_detail_pane(frame, chunks[3], app)` rendered into the new slot.
// Shift hint bar index from chunks[3] to chunks[4] at :498-528.
```

#### Stats header extension (lines 385-422)

```rust
// CURRENT — lines 385 + 414-419
let (t1, t2, t3) = app.tier_counts();
// ...
Span::styled("  T1: ", Style::default().add_modifier(Modifier::BOLD)),
Span::styled(t1.to_string(), Style::default().fg(Color::Cyan)),
Span::styled("  T2: ", Style::default().add_modifier(Modifier::BOLD)),
Span::styled(t2.to_string(), Style::default().fg(Color::Green)),
Span::styled("  T3: ", Style::default().add_modifier(Modifier::BOLD)),
Span::styled(t3.to_string(), Style::default().fg(Color::Yellow)),
// MIRROR: destructure `[t1,t2,t3,t4,t5] = app.tier_counts()` (or index the array);
// append two Span pairs: `"  T4: "` + count styled Magenta, `"  T5: "` + count styled
// LightBlue. Reuse `tier_color(4)` / `tier_color(5)` for consistency.
```

#### Filter bar extension (lines 437-442)

```rust
// CURRENT
let filter_labels = [
    (TierFilter::All, "All"),
    (TierFilter::T1, "T1"),
    (TierFilter::T2, "T2"),
    (TierFilter::T3, "T3"),
];
// MIRROR: add (TierFilter::T4, "T4"), (TierFilter::T5, "T5"). The loop at :450-464
// already handles any length array and applies bold+cyan for `*f == app.filter`.
```

#### Command parser extension (lines 672-704)

```rust
// CURRENT — the existing `:filter tN` arms
"filter all" => {
    app.filter = TierFilter::All;
    app.table_state.select(Some(0));
}
"filter t1" => { app.filter = TierFilter::T1; app.table_state.select(Some(0)); }
"filter t2" => { app.filter = TierFilter::T2; app.table_state.select(Some(0)); }
"filter t3" => { app.filter = TierFilter::T3; app.table_state.select(Some(0)); }
// MIRROR: add `"filter t4"` and `"filter t5"` arms using the exact 2-statement
// body pattern. Append BEFORE the `"sort ..."` arms to preserve grouping.
```

#### Help overlay extension (lines 559, 571)

```rust
// CURRENT
Line::from("  Tab         Cycle filter: All -> T1 -> T2 -> T3"),
// ... later ...
Line::from("  filter all|t1|t2|t3"),
// MIRROR: extend Tab line to `"All -> T1 -> T2 -> T3 -> T4 -> T5"`.
// MIRROR: extend filter command line to `"filter all|t1|t2|t3|t4|t5"`.
// ADD: new "Detail Pane" block per Research §Code Examples (§Complete help overlay extension).
```

#### Attach-mode AppEvent construction (lines 811-827)

```rust
// CURRENT — NOTE: this location needs the NEW t5_formula field set to None.
let ev = AppEvent {
    nonce,
    tier,
    // ... existing fields ...
    t4_capability: None,
    t5_proof: None,
    t5_proof_valid: None,
};
// MIRROR: add `t5_formula: None,` — attach mode reads from DB which has no
// formula; detail pane falls back to "formula=(unavailable — legacy db)".
```

#### Also: integrated-mode `NonceMeta` init (line 912)

Already sets `t5_formula: None` — this is a BUG site that must also be fixed (Phase 14 has T5 formulas populated from the payload catalog in `src/server/mod.rs:250-266` at `run_server` but the integrated-mode monitor path at `src/monitor/mod.rs:903-915` hand-builds a separate nonce_map and drops the formula). Plan must either (a) populate `t5_formula` in the monitor's nonce_map construction by loading the catalog, OR (b) delegate nonce_map construction to a shared helper. See Research §Pattern 5 for resolution.

#### Test patterns to mirror

```rust
// Analog: src/monitor/mod.rs:1183-1194
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
    assert_eq!(state.filter, TierFilter::All);
}
// MIRROR: extend to 6-state cycle All->T1->T2->T3->T4->T5->All.
// Research §Pitfall 5 warns: stale test silently passes on incomplete impl.
```

```rust
// Analog: src/monitor/mod.rs:1239-1244
#[test]
fn test_tier_color() {
    assert_eq!(tier_color(1), Color::Cyan);
    assert_eq!(tier_color(2), Color::Green);
    assert_eq!(tier_color(3), Color::Yellow);
    assert_eq!(tier_color(4), Color::White);
}
// MIRROR: change T4 to Magenta; add T5 => LightBlue; keep fallback for tier=99.
```

```rust
// Analog: src/monitor/mod.rs:1160-1171
#[test]
fn test_tier_counts_excludes_replays() {
    let mut state = AppState::new();
    state.push_event(make_test_event(1, false, "1.2.3.4", "sess1", 1000));
    // ... existing body ...
    let (t1, t2, t3) = state.tier_counts();
    assert_eq!(t1, 2);
    assert_eq!(t2, 1);
    assert_eq!(t3, 1);
}
// MIRROR: extend with T4 and T5 events; assert `[usize; 5]` values.
```

```rust
// Analog: make_test_event helper at src/monitor/mod.rs:995-1023
// MIRROR: extend with `t5_formula: None,` default. For T5-specific tests, add
// a variant `make_test_event_t5(proof, valid, formula)` or extend signature.
```

---

### `src/report/mod.rs` (service, batch transform)

**Analog:** same file — `proof_level`, Evidence Table loop, Executive Summary block.

#### `proof_level` match extension (lines 38-45)

```rust
// CURRENT
fn proof_level(tier: u8) -> &'static str {
    match tier {
        1 => "Arbitrary Callback",
        2 => "Conditional Branch",
        3 => "Computed Callback",
        _ => "Unknown",
    }
}
// MIRROR per D-14-09: add
//   4 => "Capability Introspection",
//   5 => "Multi-step Compliance",
```

#### Executive Summary extension (lines 90-101)

```rust
// CURRENT
md.push_str(&format!(
    "| Tier 1 (Arbitrary Callback) | {} |\n", summary.tier1_sessions));
md.push_str(&format!(
    "| Tier 2 (Conditional Branch) | {} |\n", summary.tier2_sessions));
md.push_str(&format!(
    "| Tier 3 (Computed Callback) | {} |\n", summary.tier3_sessions));
// MIRROR: append two more push_str calls with Tier 4 and Tier 5 labels,
// matching the `proof_level()` strings exactly. D-14-12: always emit rows,
// even when counts are 0.
```

#### Evidence Table header + row extension (lines 118-141)

```rust
// CURRENT header + separator
md.push_str("| Session | Tier | Proof Level | First Seen | Source IP | User Agent | Fire Count | Classification | Payload |\n");
md.push_str("|---------|------|-------------|------------|-----------|------------|------------|----------------|--------|\n");

// CURRENT row body
for s in &detection_sessions {
    let session_short = md_escape(&s.session_id[..s.session_id.len().min(8)]);
    let tier_str = s.tier.to_string();
    let proof = proof_level(s.tier);
    let first_seen = format_timestamp(&s.first_seen_at);
    let ip = md_escape(&s.remote_addr);
    let ua = md_escape(&s.user_agent);
    let fire = s.fire_count.to_string();
    let class = md_escape(&s.classification);
    let payload = md_escape(&s.payload_id);
    md.push_str(&format!(
        "| {session_short} | {tier_str} | {proof} | {first_seen} | {ip} | {ua} | {fire} | {class} | {payload} |\n"
    ));
}

// CURRENT empty state
md.push_str("| — | — | — | — | — | — | — | — | — |\n");

// MIRROR: (1) add "Evidence" column to header (recommended: between Classification
// and Payload per Research Open Question 1); (2) add `let evidence = evidence_cell(s);`
// local and interpolate `{evidence}` into format string; (3) extend empty-state row
// pipe count to match (Research Pitfall 6).
//
// NEW helper per Research §Pattern 3:
//   fn evidence_cell(s: &ReportSession) -> String
// T4: md_escape(t4_capability) or "—"
// T5: "{NNN} ✓ VALID" / "{NNN} ✗ INVALID" via md_escape
// T1/T2/T3: "—" (D-14-13)
```

**Apply identical treatment** to the Known Crawler Sessions table at lines 144-165 — same header, same loop, same empty state row.

#### Test pattern to mirror (lines 211-217)

```rust
// CURRENT
#[test]
fn test_proof_level_mapping() {
    assert_eq!(proof_level(1), "Arbitrary Callback");
    assert_eq!(proof_level(2), "Conditional Branch");
    assert_eq!(proof_level(3), "Computed Callback");
    assert_eq!(proof_level(99), "Unknown");
}
// MIRROR: add assertions for proof_level(4) and proof_level(5) matching D-14-09.
```

---

### `src/store/mod.rs` (service, CRUD)

**Analog:** same file — `ReportSummary`, `ReportSession`, `query_report_summary`, `query_report_sessions`.

#### `ReportSummary` extension (lines 195-205)

```rust
// CURRENT
#[derive(serde::Serialize)]
pub struct ReportSummary {
    pub total_sessions: u32,
    pub detection_sessions: u32,
    pub crawler_sessions: u32,
    pub tier1_sessions: u32,
    pub tier2_sessions: u32,
    pub tier3_sessions: u32,
    pub earliest_event: Option<String>,
    pub latest_event: Option<String>,
}
// MIRROR: add `pub tier4_sessions: u32,` and `pub tier5_sessions: u32,`
// BETWEEN tier3_sessions and earliest_event to keep tier fields grouped.
```

#### `query_report_summary` subquery extension (lines 245-264)

```rust
// CURRENT — T1/T2/T3 pattern (identical shape)
let tier1_sessions: u32 = conn.query_row(
    "SELECT COUNT(DISTINCT session_id) FROM events
     WHERE tier = 1 AND extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
    [],
    |row| row.get(0),
)?;
// ... tier2 and tier3 exact same shape ...

// MIRROR: clone this block for tier=4 and tier=5. Same `NOT LIKE` crawler-exclusion.
// Add to `Ok(ReportSummary { ... })` struct literal at line 272.
```

#### `ReportSession` extension (lines 207-219)

```rust
// CURRENT
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
}
// MIRROR: append three Option<_> fields per Research §Pattern 3:
//   pub t4_capability: Option<String>,
//   pub t5_proof: Option<String>,
//   pub t5_proof_valid: Option<bool>,
```

#### `query_report_sessions` SELECT + row mapping extension (lines 288-322)

```rust
// CURRENT SELECT — MAX() aggregation pattern
let mut stmt = conn.prepare(
    "SELECT session_id, tier, payload_id, embedding_loc,
            MIN(first_seen_at) as first_seen_at,
            MAX(last_seen_at) as last_seen_at,
            SUM(fire_count) as total_fires,
            MAX(remote_addr) as remote_addr,
            MAX(user_agent) as user_agent,
            MAX(extra_headers) as extra_headers
     FROM events
     GROUP BY session_id, tier
     ORDER BY MIN(first_seen_at) DESC",
)?;

// CURRENT row mapping — Option<T> NULL-safe pattern (lines 303-317)
let sessions = stmt
    .query_map([], |row| {
        let extra_headers: Option<String> = row.get(9)?;
        let classification = parse_classification(extra_headers.as_deref());
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
            classification,
        })
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?;

// MIRROR per Research §Pattern 3:
// (1) Append `, MAX(t4_capability) as t4_capability, MAX(t5_proof) as t5_proof,
//     MAX(t5_proof_valid) as t5_proof_valid` to SELECT list (indexes 10, 11, 12).
// (2) Append to row mapping struct literal:
//       t4_capability: row.get::<_, Option<String>>(10)?,
//       t5_proof:      row.get::<_, Option<String>>(11)?,
//       t5_proof_valid: row.get::<_, Option<bool>>(12)?,
//     (Do NOT unwrap_or_default — these are pure Option pass-through.)
// D-14-14: unconditional SELECT; relies on Phase 13 migration guarantee.
// Research §Pitfall 1: never use `row.get::<_, String>` on nullable columns.
```

#### Test patterns to mirror

```rust
// Analog: src/store/mod.rs:790-837 — test_schema_t4_columns / test_schema_t5_columns
// These already pass; no changes needed. Still the pattern for any new column test.

// Analog: src/store/mod.rs:756-768 — test_query_report_summary_empty_db
#[test]
fn test_query_report_summary_empty_db() {
    let conn = in_memory_conn();
    let summary = query_report_summary(&conn).unwrap();
    assert_eq!(summary.total_sessions, 0);
    // ...
    assert_eq!(summary.tier1_sessions, 0);
    assert_eq!(summary.tier2_sessions, 0);
    assert_eq!(summary.tier3_sessions, 0);
    // MIRROR: add assert_eq!(summary.tier4_sessions, 0); tier5_sessions as well.
}
```

---

### `src/server/mod.rs` (controller, request-response)

**Analog:** same file — existing `t5_callback_handler` already has `formula` in scope.

#### Populate `t5_formula` on `RawCallbackEvent` (lines 160-194)

```rust
// CURRENT — line 160
let formula = match meta.t5_formula.as_ref() {
    Some(f) => f,
    None => return StatusCode::NO_CONTENT,
};
// ... verification logic ...

// CURRENT — lines 182-193 (RawCallbackEvent construction)
let event = RawCallbackEvent {
    nonce,
    tier: meta.tier,
    payload_id: meta.payload_id.clone(),
    embedding_loc: meta.embedding_loc.clone(),
    fingerprint,
    classification,
    received_at: now_unix_secs(),
    t4_capability: None,
    t5_proof: Some(proof_str),
    t5_proof_valid: Some(proof_valid),
};
// MIRROR per Research §Pattern 5: add `t5_formula: Some(*formula),` as last field.
// `formula` is `&T5Formula` (Copy), so `*formula` dereferences to owned value.
```

**Also update** the other 2 `RawCallbackEvent` construction sites to set `t5_formula: None`:

- `src/server/mod.rs:73` (callback_handler — T1)
- `src/server/mod.rs:123` (t4_callback_handler — T4)

---

### `src/broker/mod.rs` (service, event-driven)

**Analog:** same file — existing Phase 13 propagation pattern at `:18-35`.

#### `broker_task` field extension (lines 32-34)

```rust
// CURRENT
let app_event = AppEvent {
    nonce: raw.nonce,
    tier: raw.tier,
    // ... existing fields ...
    received_at: raw.received_at,
    // Phase 13: propagate T4/T5 payload values so db_writer_task can
    // persist them. Missing any of these would silently drop T4/T5
    // data (RESEARCH Pitfall 4).
    t4_capability: raw.t4_capability,
    t5_proof: raw.t5_proof,
    t5_proof_valid: raw.t5_proof_valid,
};
// MIRROR: append `t5_formula: raw.t5_formula,` — same one-line propagation pattern.
// T5Formula is Copy; no .clone() needed. Update the rustdoc comment to mention
// Phase 14 addition.
```

#### Test patterns to mirror (lines 241-325)

```rust
// Analog: test_broker_task_propagates_t5_proof at src/broker/mod.rs:287-325
// Already asserts t4_capability / t5_proof / t5_proof_valid on AppEvent.
// MIRROR: extend the RawCallbackEvent literal at :301-313 with
// `t5_formula: Some(T5Formula { a: 7, b: 13, modulus: 1000 }),`
// and add `assert_eq!(app_event.t5_formula, Some(...))`.
// ALSO: update make_raw_event helper at :184-204 to include `t5_formula: None`.
```

---

### `tests/test_report.rs` (test, integration)

**Analog:** same file — `insert_event` helper at `:11-43`, `test_report_with_events` at `:91-149`.

#### Insert helper extension

```rust
// CURRENT — tests/test_report.rs:12-43
#[allow(clippy::too_many_arguments)]
fn insert_event(
    conn: &rusqlite::Connection,
    nonce: &str, tier: u8,
    payload_id: &str, embedding_loc: &str,
    session_id: &str, remote_addr: &str, user_agent: &str,
    classification: &str,
    first_seen_epoch: Option<u64>,
) {
    // Insert nonce_map entry ... Insert events entry directly with epoch string ...
    conn.execute(
        "INSERT INTO events (nonce, tier, payload_id, embedding_loc, first_seen_at, last_seen_at, fire_count, is_replay, session_id, remote_addr, user_agent, extra_headers)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, 1, 0, ?6, ?7, ?8, ?9)",
        rusqlite::params![...],
    )
    .unwrap();
}

// MIRROR per CONTEXT §Tests + RESEARCH §Validation Architecture: add a new helper
// `insert_event_t4(conn, ..., capability: &str)` and `insert_event_t5(conn, ...,
// proof: &str, proof_valid: bool)` that use the same shape but extend the INSERT
// statement with the t4_capability / t5_proof / t5_proof_valid columns.
// Use `rusqlite::params![..., capability]` / `[..., proof, proof_valid as i32]`.
```

#### New tests to mirror existing shape

```rust
// Analog: test_report_with_events at :91-149 — pattern:
//   1. temp_conn()
//   2. Multiple insert_event(...) calls
//   3. let md = report::generate_report(&conn).expect(...);
//   4. Assertions on md.contains(...)

// NEW tests per Research §Validation Architecture §Phase Requirements → Test Map:
// - test_report_summary_tier4_tier5_counts — insert T4 and T5 events, assert
//   "| Tier 4 (Capability Introspection) | N |" / "| Tier 5 (Multi-step Compliance) | N |"
// - test_report_backward_compat_v40_db — NO T4/T5 events, assert zero-count rows
//   present AND Evidence column cells are all "—"
// - test_report_sessions_null_t4_for_t1_row — insert T1 only, query_report_sessions
//   directly, assert `t4_capability: None`
// - test_report_evidence_column_t4 — T4 event, assert Evidence cell contains the
//   capability string (possibly md_escape'd)
// - test_report_evidence_column_t5_valid / _invalid — T5 events with both
//   validity states, assert "NNN ✓ VALID" / "NNN ✗ INVALID" in the row
// - test_report_full_5tier_markdown — one event per tier (1..=5); golden-string
//   assertion on overall structure
```

---

### `tests/test_monitor.rs` (test, unit — new file OR extend module tests)

**Analog:** `src/monitor/mod.rs` `#[cfg(test)] mod tests` block — new unit tests live in the module per CLAUDE.md convention ("unit tests in `#[cfg(test)] mod tests` within each module, integration tests in `tests/`"). If behavior crosses module boundaries (e.g. detail pane rendering consuming payload catalog), place in `tests/`.

**Test patterns per Research §Validation Architecture:**

- `test_tier_evidence_cell_t4` — construct `AppEvent` with `t4_capability: Some("web_search,browse_page")`; assert returned `Cell` renders truncated and is styled with `tier_color(4)`.
- `test_tier_evidence_cell_t5_valid` / `_invalid` / `_unknown` — construct T5 `AppEvent` with each validity state; assert color matches Green/Red/DarkGray.
- `test_render_detail_pane_t5_with_formula` / `_without_formula` — construct T5 events with and without `t5_formula`; assert pane text includes "formula=" vs "(unavailable — legacy db)".
- `test_detail_pane_attach_mode_no_formula` (Research Pitfall 3) — attach mode always has `t5_formula: None`; render must not panic.
- `test_command_filter_t4` / `_t5` — simulate command-mode input "filter t4"/"filter t5"; assert `app.filter` matches.

---

## Shared Patterns

### Pattern A — Cell per-cell styling (layered on row style)

**Source:** `src/monitor/mod.rs:337-354`
**Apply to:** EVIDENCE column cell builder (T5 ✓/✗ green/red coloring).

```rust
let cells = vec![
    Cell::from(time_str),
    Cell::from(tier_str).style(Style::default().fg(tier_color(ev.tier))),
    Cell::from(truncate_str(class_str, 12)).style(Style::default().fg(class_color)),
    // ... more cells ...
];
let row_style = if ev.is_replay {
    Style::default().add_modifier(Modifier::DIM)
} else {
    Style::default()
};
Row::new(cells).style(row_style)
```

Per-cell `.style(...)` layers cleanly over row-level `Modifier::DIM`. T5 green/red glyph survives DIM on replay rows as "dimmed green" / "dimmed red" — defender UX is preserved.

### Pattern B — `truncate_str` for column width

**Source:** `src/monitor/mod.rs:238-246`

```rust
fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else if max <= 3 {
        s[..max].to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
```

**Apply to:** T4 EVIDENCE column cell — `truncate_str(t4_capability.as_deref().unwrap_or("—"), 20)`. Matches the same idiom already used for UA (`:331`) and NONCE (`:332`).

Note: `truncate_str` produces `...` (3 ASCII dots), not `…` (Unicode ellipsis). CONTEXT D-14-03 / D-14-04 use `…` in prose but the helper uses `...`. Planner should either (a) extend helper with a param, or (b) document discrepancy and keep `...` for consistency with UA/NONCE.

### Pattern C — `md_escape` applied to every agent-supplied cell

**Source:** `src/report/mod.rs:11-23` (`md_escape` function) + `:127-135` (application sites)

```rust
pub fn md_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '|' => out.push_str(r"\|"),
            '`' => out.push_str(r"\`"),
            '\n' => out.push(' '),
            '\r' => {}
            c => out.push(c),
        }
    }
    out
}

// Application — every agent-supplied string goes through md_escape before
// pipe-separated interpolation:
let ip = md_escape(&s.remote_addr);
let ua = md_escape(&s.user_agent);
let class = md_escape(&s.classification);
let payload = md_escape(&s.payload_id);
```

**Apply to:** `t4_capability` and `t5_proof` in the new Evidence column cells, per D-14-10. Even though server-side D-13-09 already constrains them to `^[a-z0-9_,.\-]{1,256}$` and 3 ASCII digits respectively, running them through `md_escape` is consistent with existing pipeline convention and defense-in-depth.

### Pattern D — SQL aggregate `MAX(col)` with `Option<T>` row mapping

**Source:** `src/store/mod.rs:288-322` (full query + mapping)

Key insight: SQLite `MAX(NULL, NULL, NULL) = NULL`. Combined with D-13-19 first-write-wins, `MAX(t4_capability)` returns the single non-null value within a (session_id, tier=4) group, and `NULL` within a T1/T2/T3 group. Mapping via `row.get::<_, Option<String>>(N)` is NULL-safe by construction.

**Apply to:** new `MAX(t4_capability)`, `MAX(t5_proof)`, `MAX(t5_proof_valid)` aggregations in the extended `query_report_sessions` SELECT.

### Pattern E — Idempotent enum match extension with fallback `_`

**Source:** `src/monitor/mod.rs:248-255` (`tier_color`), `src/report/mod.rs:38-45` (`proof_level`)

```rust
match tier {
    1 => ...,
    2 => ...,
    3 => ...,
    _ => ... // fallback preserved
}
```

**Apply to:** Both `tier_color` and `proof_level` — add `4 =>` and `5 =>` arms BEFORE the `_` fallback. Fallback is kept for defensive coverage of any future unrecognized tier without panic.

### Pattern F — `Option<T>` field propagation through pipeline

**Source:** `src/broker/mod.rs:32-34` (broker_task) + `src/server/mod.rs:73, 123, 182-193` (handler construction)

One-line propagation: `t4_capability: raw.t4_capability,` in broker; explicit assignment in each handler. No conditional logic; tier-specific handlers populate their relevant fields and set the others to `None`.

**Apply to:** `t5_formula: Option<T5Formula>` — same pattern end-to-end. Server T5 handler populates `Some(*formula)`; T1/T4 handlers set `None`; broker propagates field-by-field; monitor's attach-mode AppEvent builder sets `None`.

---

## No Analog Found

None — every Phase 14 change is an extension of an existing pattern in the same file as the analog. This is the strongest possible pattern alignment: no pattern-synthesis required.

---

## Metadata

**Analog search scope:**
- `/home/john/vault/projects/github.com/honeyprompt/src/monitor/mod.rs` (1,301 lines — read at 1-200, 240-450, 680-780, 800-950, 985-1301)
- `/home/john/vault/projects/github.com/honeyprompt/src/report/mod.rs` (full 218 lines)
- `/home/john/vault/projects/github.com/honeyprompt/src/store/mod.rs` (read at 1-120, 190-340, 750-930)
- `/home/john/vault/projects/github.com/honeyprompt/src/types.rs` (full 236 lines)
- `/home/john/vault/projects/github.com/honeyprompt/src/broker/mod.rs` (full 427 lines)
- `/home/john/vault/projects/github.com/honeyprompt/src/server/mod.rs` (lines 140-232 + grep survey)
- `/home/john/vault/projects/github.com/honeyprompt/tests/test_report.rs` (full 278 lines)

**Files scanned:** 7 source files + 1 integration test file
**Pattern extraction date:** 2026-04-24

---

*Every Phase 14 change is an extension of an existing pattern — no new problem domains. The discipline is: find the existing pattern for T1–T3, extend it symmetrically, update its test.*
