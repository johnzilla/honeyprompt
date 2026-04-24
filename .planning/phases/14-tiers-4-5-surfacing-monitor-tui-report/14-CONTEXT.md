# Phase 14: Tiers 4 & 5 Surfacing (Monitor TUI + Report) - Context

**Gathered:** 2026-04-24
**Status:** Ready for planning

<domain>
## Phase Boundary

A defender watching the Monitor TUI or reading a Markdown disclosure report can see the decoded T4 capability list and the T5 proof with its server-verified validity, alongside existing T1–T3 evidence, with T4/T5 counts included in the executive summary — all purely additive so a v4.0 (T1–T3-only) database still produces sensible output.

In scope for Phase 14: Monitor TUI extension (event table, detail pane, stats header, filter cycle, help overlay), Markdown disclosure report extension (evidence table column, executive summary rows, proof-level mapping), store query extension to surface T4/T5 columns.

Out of scope (later phases): `honeyprompt test-agent` per-tier scorecard for T4/T5, CI exit-code semantics for T4/T5, README 5-tier model documentation, TODOS.md updates.

</domain>

<decisions>
## Implementation Decisions

### TUI Evidence Placement

- **D-14-01:** Monitor TUI gains (a) a compact `EVIDENCE` column in the event table and (b) a fixed always-visible detail pane below the table (above the hint bar) showing full context for the selected row. Both coexist — the column is for at-a-glance scanning, the pane for full evidence.
- **D-14-02:** The detail pane is context-aware:
  - **T4 row selected** — show the full decoded, sorted capability list (no truncation).
  - **T5 row selected** — show `proof=NNN ✓ VALID` or `proof=NNN ✗ INVALID` plus the formula that produced the expected value (e.g. `formula=(seed+A)*B % M`). Formula constants come from the payload catalog via `AppState`'s nonce/payload association.
  - **T1–T3 row selected** — show `payload_id`, `embedding_loc`, and the full `nonce` (not padding; actually useful metadata that's currently truncated in the event table).
- **D-14-03:** T4 capability is truncated with `…` to fit the EVIDENCE column width in the event table; the detail pane always renders the full string.
- **D-14-04:** T5 renders as `NNN ✓` styled green / `NNN ✗` styled red in the EVIDENCE column. Same glyph convention in the detail pane with explicit `VALID` / `INVALID` text appended.

### TUI Chrome Extension

- **D-14-05:** Stats header appends `T4:n T5:n` inline to the existing `T1:n T2:n T3:n` pattern. If the 80-col minimum becomes tight, the replay-indicator label is shortened before any tier label is dropped. Header colors extend — T4 and T5 each get a distinct color (Claude's Discretion).
- **D-14-06:** Tab filter cycle extends symmetrically: `All → T1 → T2 → T3 → T4 → T5 → All`. `:filter t4` and `:filter t5` command variants added. Filter bar shows all six labels: `All | T1 | T2 | T3 | T4 | T5`.
- **D-14-07:** T5 valid/invalid split surfaces **only** in the detail pane. Header shows combined `T5:n`. No `:filter t5valid` / `:filter t5invalid` split command. Per-row `✓`/`✗` glyph in the EVIDENCE column carries the validity signal at scan-time; aggregates stay simple.
- **D-14-07a:** Help overlay (`?` key) updated: Tab description lists all five tiers; help documents the always-visible detail pane; `:filter t4` / `:filter t5` appear under Commands. No new key bindings are added (detail pane is always visible per D-14-01, so no toggle key is needed).

### Report T4/T5 Granularity

- **D-14-08:** Evidence Table stays session-grouped (one row per `(session_id, tier)` pair). A single new `Evidence` column is added that renders:
  - **Tier 4** — the full decoded tool list (from `t4_capability`).
  - **Tier 5** — `NNN ✓ VALID` or `NNN ✗ INVALID` (from `t5_proof` + `t5_proof_valid`).
  - **Tiers 1–3** — em-dash `—` (per D-14-13).
  Same schema applies to both the Evidence Table and the Known Crawler Sessions table. D-13-19 first-write-wins semantics make a single session+tier row's T4/T5 value unambiguous.
- **D-14-09:** `proof_level()` in `src/report/mod.rs` extends:
  - `4 → "Capability Introspection"`
  - `5 → "Multi-step Compliance"`
  - Existing T1–T3 labels unchanged.
- **D-14-10:** T4 capability is rendered **in full** inside the Markdown cell (not truncated). It is passed through `md_escape()` for consistency with the existing pipeline, even though server sanitization (D-13-09) already restricts it to `^[a-z0-9_,.\-]{1,256}$` and makes escaping a no-op.
- **D-14-11:** T5 rendered as `NNN ✓ VALID` / `NNN ✗ INVALID` (glyph + text), matching the TUI convention in D-14-04.

### Backward-Compat Rendering

- **D-14-12:** Always-show chrome policy. The Executive Summary table always has 5 tier rows (T1..T5), with count=0 shown when empty. The TUI stats header always shows `T4:0 T5:0`. The filter cycle always includes T4 and T5. Interpretation of roadmap success criterion #5 ("no empty T4/T5 sections printed"): "sections" refers to *dedicated headings* (e.g. a hypothetical `## Tier 4 Evidence` block), which this design doesn't have. Zero-count rows in an existing table are not sections and do not violate the criterion.
- **D-14-13:** T1–T3 rows in the Markdown Evidence table's new `Evidence` column show em-dash `—`. This matches the existing empty-state row pattern (`| — | — | ... |`) in the report and makes the column's intentional blankness clear.
- **D-14-14:** `query_report_summary` and `query_report_sessions` always `SELECT` the T4/T5 columns (`t4_capability`, `t5_proof`, `t5_proof_valid`). No `PRAGMA table_info` probe. Relies on Phase 13 migration guarantee (D-13-17: every DB open runs the additive `ALTER TABLE` migration). Legacy rows return `NULL`, which code handles as `None` / em-dash.

### Claude's Discretion

- Exact EVIDENCE column width in the TUI table (within 80x20 minimum), and which existing column tightens to make room (likely UA — already truncated).
- Detail-pane height (1 line vs 2–3 lines) and whether it takes over the current status-line row.
- Exact wording of detail-pane labels (e.g. `Selected (T4):` prefix vs header-style).
- Specific `Color` variants for T4 and T5 tier labels (Ratatui palette: Magenta, Blue, LightRed, etc.).
- Whether `ReportSummary` gains fields (`tier4_sessions`, `tier5_sessions`, `tier5_valid_sessions`) or whether `ReportSession` gains optional fields — shape choice is a planner decision given D-14-12's session-grouped row model.
- Column ordering in the Markdown evidence table (Evidence before or after Payload, Classification, etc.).
- Whether the filter bar wraps/truncates its label row if all six labels + "Sort: ..." overflow 80 cols.
- Any minor refactoring of `tier_color(tier: u8)` / `proof_level(tier: u8)` helpers (extending the match arms is required either way).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone & Requirements

- `.planning/PROJECT.md` §Current Milestone, §Proof Levels — v5.0 T4/T5 framing, graduated proof model
- `.planning/REQUIREMENTS.md` §v5.0 — UI-01..UI-05 (5 requirements owned by Phase 14)
- `.planning/ROADMAP.md` §Phase 14 — Goal, 5 success criteria including backward-compat constraint

### Prior Phase Decisions (carry forward)

- `.planning/phases/13-tiers-4-5-backend-payloads-routes-store/13-CONTEXT.md` — all D-13-* decisions. Relevant in particular: D-13-09 (T4 server sanitization), D-13-14 (T5 server-side proof verification), D-13-17 (additive migration guarantee), D-13-18 (`/cb/v1/` byte-identical), D-13-19 (first-write-wins replay semantics for T4/T5)
- `.planning/milestones/v1.0-phases/03-tui-monitor/03-UI-SPEC.md` — original Monitor TUI design contract; layout, key bindings, color conventions

### Existing Code (Reuse-First)

- `src/monitor/mod.rs` — `AppState`, `TierFilter`, `SortField`, `render_event_table`, stats-header rendering, help overlay, `tier_color(tier: u8)`. All extension points for T4/T5 are here (single-file TUI, 1,301 lines).
- `src/report/mod.rs` — `generate_report`, `proof_level(tier: u8)`, `md_escape`, session-grouped Evidence Table + Known Crawler Sessions layout
- `src/store/mod.rs` §`query_report_summary`, `query_report_sessions`, `ReportSummary`, `ReportSession` — query/shape extension points. Migration already added `t4_capability`, `t5_proof`, `t5_proof_valid` columns (lines 60–62).
- `src/types.rs` §`AppEvent`, `RawCallbackEvent`, `Tier` — event structs already carry `t4_capability: Option<String>`, `t5_proof: Option<String>`, `t5_proof_valid: Option<bool>` (Phase 13, lines 97–127). `Tier::Tier4 = 4`, `Tier::Tier5 = 5` already defined (lines 9–10).
- `src/broker/mod.rs` — event pipeline that propagates raw → enriched `AppEvent` (tier-agnostic; no changes needed beyond verifying T4/T5 payload data rides through)

### Tests

- `src/store/mod.rs::tests::test_schema_t4_columns` / `test_schema_t5_columns` — schema validation already shipped
- `src/store/mod.rs::tests::test_insert_callback_event_replay_t4_first_write_wins` / `test_insert_callback_event_replay_t5_first_write_wins` — replay semantics already shipped
- `src/report/mod.rs::tests::test_proof_level_mapping` — MUST extend with T4/T5 cases
- Existing integration tests in `tests/` — still must pass unmodified (regression gate on T1–T3 behavior)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- **`src/monitor/mod.rs::AppState`** — single central state. Gains methods: `tier_counts()` → extend to return `(t1, t2, t3, t4, t5)` or return a struct/array; add detail-pane rendering helpers.
- **`src/monitor/mod.rs::TierFilter`** enum — add `T4`, `T5` variants and extend `next()` cycle.
- **`src/monitor/mod.rs::tier_color(tier: u8)`** — extend match to add colors for 4 and 5.
- **`src/monitor/mod.rs::render_event_table`** — add EVIDENCE column; adjust `widths` array; extend cell-building logic with a `tier_evidence_cell(&AppEvent) -> Cell` helper.
- **`src/monitor/mod.rs::render`** — adjust `Layout::vertical` constraint list to add a detail-pane row between the table and hint bar.
- **`src/report/mod.rs::proof_level(tier: u8)`** — extend match.
- **`src/report/mod.rs::generate_report`** — adjust Evidence Table and Known Crawler Sessions table to add the `Evidence` column; extend Executive Summary to include Tier 4 / Tier 5 rows.
- **`src/store/mod.rs::query_report_summary`** — extend with `tier4_sessions` / `tier5_sessions` (and optionally `tier5_valid_sessions` — Claude's Discretion per above) via additional `SELECT COUNT(DISTINCT ...)` calls on the `tier = 4` / `tier = 5` predicates.
- **`src/store/mod.rs::query_report_sessions`** — add `t4_capability`, `t5_proof`, `t5_proof_valid` to the `SELECT` list and `ReportSession` struct; pass through to the report layer.
- **`src/types.rs::AppEvent`** — already has the T4/T5 fields (Phase 13). No schema changes needed.

### Established Patterns

- **Ratatui styled cells** (`Cell::from(x).style(Style::default().fg(Color::X))`) — already used extensively. Extends cleanly for ✓/✗ coloring.
- **`truncate_str(s, max)`** helper in `src/monitor/mod.rs` — use for EVIDENCE column T4 truncation; same pattern as UA and NONCE truncation today.
- **`md_escape()`** — applied uniformly to every user-supplied string in the report. Apply identically to `t4_capability` and `t5_proof` even though they're already server-constrained.
- **Session-grouped Markdown rows** — `GROUP BY session_id, tier` aggregation + MAX/MIN/SUM across replay fire_counts. Add `MAX(t4_capability)`, `MAX(t5_proof)`, `MAX(t5_proof_valid)` to the aggregation; per D-13-19 first-write-wins, `MAX` picks a consistent value within a session.
- **`tier_color(tier: u8)` match** — add colors (Claude's Discretion) for T4 and T5.
- **Color semantics already in use**: Cyan (T1), Green (T2 and success), Yellow (T3 and warning), Red (error, crawlers). T4/T5 picks should not collide — e.g. Magenta (T4) and Blue (T5), or similar.

### Integration Points

- **`render_event_table`** widths array and cell list — add one new cell per row for EVIDENCE
- **`render` layout constraint list** — add the detail-pane row (`Constraint::Length(2)` or `Length(3)`)
- **New function** `render_detail_pane(frame, area, app: &AppState)` in `src/monitor/mod.rs` — switch on selected event's tier
- **`AppState::tier_counts()`** signature — extend tuple or switch to array
- **`generate_report`** Evidence Table header + row format — add the Evidence column
- **`generate_report`** Executive Summary — two new rows (`Tier 4 (Capability Introspection)` and `Tier 5 (Multi-step Compliance)`)
- **`query_report_summary` return struct `ReportSummary`** — add new count fields
- **`ReportSession` struct** — add `t4_capability: Option<String>`, `t5_proof: Option<String>`, `t5_proof_valid: Option<bool>` fields
- **Help overlay text in `render`** — update Tab description and add `:filter t4` / `:filter t5` to commands list
- **Command-mode `:filter` parser** in `handle_key_event` / command dispatch — accept `t4`, `t5` in addition to `t1`/`t2`/`t3`

</code_context>

<specifics>
## Specific Ideas

- Detail pane is always visible, not a toggle — discoverability matters for a defender-facing tool.
- T5 detail-pane formula line is for auditability: a defender reading the monitor should be able to mentally verify that `(seed + A) * B % M` matches the submitted proof. Use the payload's `T5Formula` from `src/types.rs`.
- "Readable at a glance" (success criterion #1): the EVIDENCE column gives the scan-time signal; the detail pane gives the proof-of-work.
- T1–T3 detail-pane content (payload_id + embedding_loc + full nonce) turns the pane from "only useful for T4/T5" into "always useful" — eliminates layout jitter complaint and provides genuine utility.
- Exec summary T4/T5 rows match roadmap language: "Capability Introspection" and "Multi-step Compliance" (these are also the new `proof_level()` returns, keeping the labels consistent between Executive Summary and Evidence Table).
- Known Crawler Sessions table gets the same Evidence column treatment for consistency — a known crawler firing a T4/T5 callback is still informative evidence.

</specifics>

<deferred>
## Deferred Ideas

- **`:filter t5valid` / `:filter t5invalid` split commands** — considered and deferred (D-14-07). If defenders later need to isolate invalid-proof events, add as a small follow-up rather than ship speculative UI.
- **Header split `T5✓:n T5✗:n`** — considered and deferred (D-14-07). Same reasoning; combined count is simpler until evidence shows defenders need the split.
- **Per-event evidence rendering (instead of session-grouped)** — considered and deferred (D-14-08). First-write-wins + session grouping gives the same information with less noise; revisit if D-13-19 semantics change or if per-event distinctions become meaningful.
- **Schema-probe via `PRAGMA table_info`** — considered and deferred (D-14-14). Phase 13 migration guarantee makes it redundant; add only if we ever open DBs created outside our migration pipeline.
- **`test-agent` T4/T5 scorecard + CI exit codes** — milestone-deferred to Phase 15 (TESTAGENT-01..03).
- **README Proof Levels 5-tier documentation + TODOS.md cleanup** — milestone-deferred to Phase 15 (DOCS-01..04).
- **JSON / HTML report formats** — out of scope at milestone level (PROJECT.md).
- **Web dashboard for T4/T5** — out of scope at milestone level.

</deferred>

---

*Phase: 14-tiers-4-5-surfacing-monitor-tui-report*
*Context gathered: 2026-04-24*
