---
phase: 14-tiers-4-5-surfacing-monitor-tui-report
verified: 2026-04-24T21:21:09Z
status: passed
score: 5/5 must-haves verified
overrides_applied: 0
---

# Phase 14: Tiers 4 & 5 Surfacing (Monitor TUI + Report) Verification Report

**Phase Goal:** A defender watching the Monitor TUI or reading a Markdown disclosure report can see the decoded T4 capability list and the T5 proof with its server-verified validity, alongside existing T1–T3 evidence and with T4/T5 counts included in the executive summary.

**Verified:** 2026-04-24T21:21:09Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Monitor TUI Tier 4 event shows decoded sorted tool list in detail/row view | VERIFIED | `tier_evidence_cell` in `src/monitor/mod.rs:282` (T4 branch truncates and styles via `tier_color(4)` Magenta); `render_detail_pane` T4 arm at `src/monitor/mod.rs:423-440` renders full capability list with `tier_color(4)` bold label; cell wired in `render_event_table` via `tier_evidence_cell(ev)` at `src/monitor/mod.rs:391`; attach-mode SELECT (gap fix c475e00) now reads `t4_capability` column from DB at `src/monitor/mod.rs:939`; 8 unit tests pass. |
| 2 | Monitor TUI Tier 5 event shows submitted proof with visible validity indicator reflecting server's `proof_valid` check | VERIFIED | `validity_glyph(Option<bool>)` at `src/monitor/mod.rs:270-277` returns `("✓", Green)` / `("✗", Red)` / `("?", Gray)`; `tier_evidence_cell` T5 branch composes `{proof} {glyph}`; `render_detail_pane` T5 arm renders `T5 proof: NNN ✓ VALID` with formula line; `AppEvent.t5_formula` populated end-to-end (server `Some(*formula)` at `src/server/mod.rs:199`, broker `raw.t5_formula` at `src/broker/mod.rs:37`, integrated-mode catalog load at `src/monitor/mod.rs:1086-1099`); attach-mode gap fix 60a75d0 reads `t5_proof_valid` as `Option<i64>` (rusqlite Option<bool> was silently dropping rows); user visually verified `861 ✓` rendering. |
| 3 | `honeyprompt report` produces Markdown with per-event T4 decoded tool list and T5 proof+verification result, interleaved with T1–T3 in same format | VERIFIED | `evidence_cell` at `src/report/mod.rs:53` handles all 5 tiers (T4 `md_escape(t4_capability)`, T5 `NNN ✓ VALID` / `NNN ✗ INVALID`, T1-T3 em-dash); 10-column Evidence Table header inserted in both Evidence Table and Known Crawler Sessions tables; `query_report_sessions` selects `MAX(t4_capability/t5_proof/t5_proof_valid)` with NULL-safe `Option<T>` mapping at `src/store/mod.rs:330-332`; 6 integration tests in `tests/test_report.rs` pass (test_report_full_5tier_markdown asserts all 5 tiers interleave correctly with per-tier Evidence cells). |
| 4 | Report executive summary extends to list Tier 4 and Tier 5 totals alongside T1–T3 | VERIFIED | `ReportSummary.tier4_sessions` / `tier5_sessions` at `src/store/mod.rs:212-213`; `query_report_summary` COUNT(DISTINCT session_id) subqueries for tier=4/5 with KnownCrawler exclusion; executive summary emits `| Tier 4 (Capability Introspection) | {} |` and `| Tier 5 (Multi-step Compliance) | {} |` rows at `src/report/mod.rs:128-135` (always-show per D-14-12); end-to-end `honeyprompt report --stdout` against empty DB shows both rows with count 0 interleaved between Tier 3 and Known Crawler Sessions. |
| 5 | All UI changes are purely additive — v4.0 database (T1–T3 only) still produces sensible output with no empty T4/T5 sections | VERIFIED | `test_report_backward_compat_v40_db` seeds only T1 and asserts exec summary rows for Tier 4/5 present with count 0 (D-14-12) AND T1 evidence cell em-dash; `test_query_report_sessions_null_t4_t5_for_legacy_rows` confirms Option<T> mapping returns None for legacy rows (no panic); TUI tier_counts now returns `[usize; 5]` — header shows `T4:0 T5:0` gracefully; detail-pane T5 arm has `formula=(unavailable — legacy db)` fallback for `t5_formula: None`; live `report --stdout` on empty DB confirmed renders cleanly with no empty sections. |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/types.rs` | `pub t5_formula: Option<T5Formula>` on both RawCallbackEvent and AppEvent | VERIFIED | 2 matches on production types (lines 58, 110) + 1 test literal (136); field positioned last in both structs |
| `src/server/mod.rs` | T5 handler `Some(*formula)`; T1/T4 `None` | VERIFIED | Line 199: `t5_formula: Some(*formula)`; Lines 85, 137: `None`; parent-dir auto-create at line 289 (gap fix 2d1ca63) |
| `src/broker/mod.rs` | Propagate `raw.t5_formula` on AppEvent | VERIFIED | Line 37: `t5_formula: raw.t5_formula`; test helpers + 2 test AppEvent literals extended with `t5_formula: None` |
| `src/monitor/mod.rs` | TierFilter 6 variants, tier_evidence_cell, render_detail_pane, validity_glyph, integrated-mode catalog load, attach-mode SELECT of T4/T5 columns | VERIFIED | `TierFilter::T4` / `::T5` enum variants; `fn validity_glyph` line 270; `fn tier_evidence_cell` line 282; `fn render_detail_pane` line 414; integrated-mode `load_catalog` + `t5_formulas_by_payload_id` lines 1086-1099; attach-mode SELECT includes `t4_capability, t5_proof, t5_proof_valid` line 939 (gap fix c475e00); `t5_proof_valid` read as `Option<i64>` then mapped to bool line 972 (gap fix 60a75d0); Color::DarkGray→Gray bump throughout (gap fix 170e97b) |
| `src/store/mod.rs` | ReportSummary tier4/tier5_sessions; ReportSession t4_capability/t5_proof/t5_proof_valid; MAX() aggregation with Option<T> | VERIFIED | Lines 212-213 struct fields; Lines 330-332 SQL MAX() aggregation; parent-dir auto-create line 10 (gap fix f4f8e70) |
| `src/report/mod.rs` | proof_level(4/5), evidence_cell helper, 10-column tables, exec summary Tier 4/5 rows always-show | VERIFIED | Line 43-44 proof_level arms; Line 53 `fn evidence_cell`; Lines 129, 133 exec summary rows; two Evidence Table instances (Evidence Table + Known Crawler Sessions) with matching 10-column schema |
| `src/setup/mod.rs` | Setup wizard offers all 5 tiers | VERIFIED | Lines 55-61 tier_labels array includes T4/T5 (gap fix be646d2); `defaults(&[true; 5])` |
| `tests/test_report.rs` | insert_event_t4/t5 helpers + 6 integration tests | VERIFIED | Lines 47, 77 helpers; 6 new test functions present (lines 343, 386, 430, 459, 483, 509) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `server/mod.rs::t5_callback_handler` | `RawCallbackEvent.t5_formula` | `Some(*formula)` | WIRED | Exactly 1 match at line 199 |
| `broker/mod.rs::broker_task` | `AppEvent.t5_formula` | `raw.t5_formula` | WIRED | Exactly 1 match at line 37 (one-line propagation) |
| integrated-mode monitor startup | `NonceMeta.t5_formula` | `catalog::load_catalog()` + `t5_formulas_by_payload_id` HashMap | WIRED | Load at line 1086, lookup at line 1096 — fixes latent Phase-13 bug where T5 callbacks were silently dropped with NO_CONTENT |
| `render_event_table` | `tier_evidence_cell(ev)` | Cell inserted between FIRES and REPLAY | WIRED | 1 match at line 391 |
| `render` Layout | `render_detail_pane(frame, chunks[3], app)` | chunks[3] slot, hint bar shifted to chunks[4] | WIRED | Call site at line 638; chunks[3] only used by detail pane; hint bar uses chunks[4] |
| `render_detail_pane` | `ev.t5_formula` | Option<T5Formula> match with legacy fallback | WIRED | T5 arm checks `match ev.t5_formula { Some(f) => formula=(seed+{a})*{b} % {mod}, None => formula=(unavailable — legacy db) }` |
| attach-mode `run_loop_attach` | DB T4/T5 columns | `SELECT ... t4_capability, t5_proof, t5_proof_valid` | WIRED | Line 939 SELECT; gap fix c475e00 |
| `generate_report` exec summary | `summary.tier4_sessions` / `tier5_sessions` | format! calls always-show per D-14-12 | WIRED | Lines 128-135 |
| `generate_report` Evidence Table row | `evidence_cell(s)` | `let evidence = evidence_cell(s);` + `{evidence}` | WIRED | 2 call sites (Evidence Table + Known Crawler Sessions); both data-loop format! strings include `{evidence}` |
| `proof_level` | T4/T5 labels | match arms | WIRED | `4 => "Capability Introspection"`, `5 => "Multi-step Compliance"` — labels match exec summary literals verbatim |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `render_detail_pane` | `ev.t5_formula` | AppEvent from broker; populated by `t5_callback_handler` via catalog-sourced NonceMeta | Integrated mode: yes (catalog load populates formula); Attach mode: no but gracefully falls back | FLOWING |
| `render_detail_pane` | `ev.t4_capability` | AppEvent from broker OR attach-mode DB SELECT | Yes — integrated mode populates via `t4_callback_handler`, attach mode reads from DB (c475e00) | FLOWING |
| `render_detail_pane` | `ev.t5_proof` / `ev.t5_proof_valid` | AppEvent from broker OR attach-mode DB SELECT | Yes — integrated mode sets via server-verified math; attach mode reads t5_proof_valid as Option<i64> to avoid rusqlite silent-drop (60a75d0) | FLOWING |
| `render_event_table` EVIDENCE cell | `tier_evidence_cell(ev)` | AppEvent tier + t4_capability/t5_proof/t5_proof_valid | Yes — all sources populated above | FLOWING |
| stats header T4/T5 spans | `tier_counts()[3]`, `[4]` | AppState.events iterator | Yes — events come from broker broadcast or attach-mode DB read | FLOWING |
| Evidence Table row | `evidence_cell(s)` | ReportSession loaded via `query_report_sessions` SELECT | Yes — MAX(t4_capability/t5_proof/t5_proof_valid) with Option<T> mapping; verified end-to-end via integration tests | FLOWING |
| Executive Summary T4/T5 rows | `summary.tier4_sessions` / `tier5_sessions` | query_report_summary COUNT(DISTINCT) subqueries | Yes — confirmed via live `report --stdout` against empty DB (both rows render with count 0) | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Report exec summary includes Tier 4 & Tier 5 rows with correct labels | `/tmp/hp-verify$ honeyprompt report --stdout` on empty DB | Output contains `| Tier 4 (Capability Introspection) | 0 |` and `| Tier 5 (Multi-step Compliance) | 0 |` | PASS |
| Evidence Table has 10-column schema with Evidence column between Classification and Payload | Same report output | Header: `\| Session \| Tier \| Proof Level \| First Seen \| Source IP \| User Agent \| Fire Count \| Classification \| Evidence \| Payload \|` | PASS |
| Known Crawler Sessions table has identical 10-column schema | Same report output | Matching header + em-dash empty-state row | PASS |
| Empty-state rows have 10 em-dashes (column-count invariant) | Same report output | `\| — \| — \| — \| — \| — \| — \| — \| — \| — \| — \|` | PASS |
| Full test suite passes | `cargo test` | 156 lib + 15 test_report + 14 test_serve + 4 test_test_agent + 6 other = all green | PASS |
| Clippy clean | `cargo clippy --all -- -D warnings` | 0 warnings, 0 errors | PASS |
| Formatting clean | `cargo fmt --all -- --check` | Silent exit 0 | PASS |
| Release binary builds | `cargo build --release` | Finished release profile | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| UI-01 | 14-01, 14-02 | Monitor TUI renders Tier 4 capability summaries (decoded tool list) in detail/row view | SATISFIED | EVIDENCE column truncated list (Magenta) + detail pane full list; attach-mode reads `t4_capability` from DB via c475e00; 8 unit tests + user visual verification |
| UI-02 | 14-01, 14-02 | Monitor TUI renders Tier 5 chain proofs with visible `proof_valid` indicator (✓/✗) | SATISFIED | `validity_glyph` helper; EVIDENCE `{proof} ✓/✗/?`; detail pane `T5 proof: NNN ✓ VALID` + formula line; user visually verified `861 ✓` rendering |
| UI-03 | 14-03 | Markdown report shows per-event T4 evidence alongside T1–T3 | SATISFIED | `evidence_cell` T4 branch (md_escape full capability); 10-column Evidence Table; 2 integration tests (`test_report_evidence_column_t4`, `test_report_full_5tier_markdown`) |
| UI-04 | 14-03 | Markdown report shows per-event T5 evidence (submitted proof + server verification) | SATISFIED | `evidence_cell` T5 branch produces `NNN ✓ VALID` / `NNN ✗ INVALID`; 3 integration tests (valid/invalid/unverified) |
| UI-05 | 14-03 | Executive summary counts extend to Tier 4 and Tier 5 totals | SATISFIED | `ReportSummary.tier4/tier5_sessions` + always-show exec rows; verified end-to-end via live `report --stdout` |

All 5 phase requirements SATISFIED. No orphaned requirements — REQUIREMENTS.md maps UI-01..05 exclusively to Phase 14, and all 5 are claimed in the plans' `requirements:` frontmatter (14-01 → UI-02; 14-02 → UI-01, UI-02; 14-03 → UI-03, UI-04, UI-05).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | None — clippy clean with `-D warnings` | — | — |

No TODO/FIXME/XXX/HACK/PLACEHOLDER comments introduced by Phase 14 changes. No stub returns, no empty handlers, no console-log-only implementations. All hardcoded `None` values flagged by grep are intentional (T1/T4 handlers correctly set `t5_formula: None`; attach-mode AppEvent correctly degrades to `None` when catalog is unavailable; both paths are semantically correct, not stubs).

### Gap-Closure Commits Validated

Six post-Task-4 fixes exposed during live human verification are all present in the final state:

| Commit | File | Fix | Validated |
|--------|------|-----|-----------|
| `be646d2` | `src/setup/mod.rs` | Setup wizard tier list extended from 3 to 5 entries | YES — Lines 55-61 include all 5 tier labels, `defaults(&[true; 5])` |
| `2d1ca63` | `src/server/mod.rs::serve` | Auto-create `.honeyprompt/` parent dir before SQLite open | YES — Line 289 `std::fs::create_dir_all(parent)` |
| `f4f8e70` | `src/store/mod.rs::open_or_create_db` | Auto-create DB parent dir for all callers | YES — Line 10 `std::fs::create_dir_all(parent)` |
| `c475e00` | `src/monitor/mod.rs::run_loop_attach` | SELECT now includes `t4_capability, t5_proof, t5_proof_valid` columns | YES — Line 939 SELECT includes all three new columns |
| `60a75d0` | `src/monitor/mod.rs::run_loop_attach` | T5 proof_valid read as `Option<i64>` to avoid rusqlite `Option<bool>` silent row-drop; DB errors surface to status line | YES — Line 972 `let t5_proof_valid: Option<bool> = t5_proof_valid_int.map(|v| v != 0);` |
| `170e97b` | `src/monitor/mod.rs` | `Color::DarkGray` → `Color::Gray` throughout for readability on dark terminals | YES — 17 `Color::Gray` usages, 0 `Color::DarkGray` remaining |

### Human Verification Required

None outstanding. Plan 14-02 Task 4 (checkpoint:human-verify) was completed in-flight by the user — 5-tier stats header, EVIDENCE column glyphs, detail pane formula rendering, filter cycle, and help overlay all visually validated. The gap-closure commits above were driven by that live verification session; verifier confirms all fixes are in the final tree.

### Gaps Summary

No gaps. Phase 14 achieves its goal completely:

- **Monitor TUI (UI-01, UI-02):** 5-tier filter cycle, EVIDENCE column with tier-appropriate glyphs/colors, always-visible detail pane with T4 full capability list and T5 proof+formula (with legacy-DB fallback), stats header showing T1–T5 counts, updated help overlay. Integrated-mode catalog loading fixes the latent Phase-13 bug that silently dropped T5 callbacks. Attach-mode now correctly reads T4/T5 columns from DB (previously hardcoded None).
- **Markdown Report (UI-03, UI-04, UI-05):** 10-column Evidence Table with per-tier `evidence_cell` (T4 md-escaped capability, T5 `NNN ✓ VALID` / `NNN ✗ INVALID`, T1–T3 em-dash); Executive Summary always emits Tier 4 / Tier 5 rows even with count 0; backward-compatible with v4.0 legacy DBs (NULL-safe Option<T> row mapping).
- **CI gates:** `cargo test` (156 lib + 15 integration report + 14 integration serve + 4 test-agent), `cargo clippy -- -D warnings`, `cargo fmt --check` all green.
- **Gap-closure fixes from live verification:** all 6 commits validated present in final state.

---

*Verified: 2026-04-24T21:21:09Z*
*Verifier: Claude (gsd-verifier)*
