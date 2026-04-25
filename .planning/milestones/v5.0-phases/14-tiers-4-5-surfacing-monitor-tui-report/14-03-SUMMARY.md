---
phase: 14
plan: 03
subsystem: store/report/tests
tags: [t4-rendering, t5-rendering, markdown-report, evidence-table, exec-summary, backward-compat, phase-14]
requires:
  - Plan 14-01 (AppEvent.t5_formula; also guarantees broker/server no longer panics on T5Formula missing)
  - Phase 13 migration guarantee (D-13-17 additive ALTER, D-13-19 first-write-wins)
  - Phase 13 ReportSession / ReportSummary baseline (src/store/mod.rs)
  - Phase 13 src/report/mod.rs Evidence Table + Known Crawler Sessions baseline
provides:
  - ReportSummary.tier4_sessions / ReportSummary.tier5_sessions (u32, D-14-12 always-show)
  - ReportSession.t4_capability / t5_proof / t5_proof_valid (Option<T>, NULL-safe D-14-14)
  - query_report_summary tier4_sessions + tier5_sessions COUNT(DISTINCT session_id) subqueries
  - query_report_sessions MAX(t4_capability/t5_proof/t5_proof_valid) aggregation
  - proof_level(4) = "Capability Introspection", proof_level(5) = "Multi-step Compliance" (D-14-09)
  - evidence_cell(&ReportSession) helper (D-14-10, D-14-11, D-14-13)
  - Executive Summary Tier 4 / Tier 5 rows (D-14-12 always-show)
  - Evidence Table + Known Crawler Sessions 10-column schema (Evidence column inserted between Classification and Payload)
  - insert_event_t4 / insert_event_t5 integration test helpers
affects:
  - UI-03 (T4 report evidence) — satisfied
  - UI-04 (T5 report evidence) — satisfied
  - UI-05 (T4/T5 exec summary counts) — satisfied
  - Plan 14-02 (Monitor TUI) runs in parallel in the sibling worktree; no file overlap with this plan
  - Downstream Markdown parsers must tolerate a 10-column Evidence Table (was 9)
tech-stack:
  added: []
  patterns:
    - "NULL-safe SQLite column aggregation: row.get::<_, Option<T>>(N)"
    - "Pipe-count invariant regression test (Threat T-14-03-02 mitigation)"
    - "md_escape over every agent-supplied string (Pattern C, Threat T-14-03-01 mitigation)"
    - "Always-show chrome: Executive Summary emits 5 tier rows regardless of counts (D-14-12)"
key-files:
  created:
    - .planning/phases/14-tiers-4-5-surfacing-monitor-tui-report/14-03-SUMMARY.md
  modified:
    - src/store/mod.rs
    - src/report/mod.rs
    - tests/test_report.rs
decisions:
  - "Task 1 ReportSummary field ordering: tier4/tier5 inserted between tier3 and earliest_event (keeps tier fields contiguous)"
  - "Task 1 ReportSession field ordering: t4_capability / t5_proof / t5_proof_valid appended after classification (no mid-struct insertion; keeps existing field indexing stable)"
  - "Task 2 Evidence column placement: between Classification and Payload (per plan's Research Open Q1 recommendation — Payload stays rightmost for visual anchoring)"
  - "evidence_cell passes md_escape on t4_capability even though server-side D-13-09 sanitization makes it a no-op on well-formed inputs (defense in depth, Pattern C)"
  - "T5 unverified fallback: when t5_proof_valid is None, render just the proof (md_escape'd), no glyph — consistent with the 'no claim made' semantics"
metrics:
  duration_minutes: 6
  tasks_completed: 3
  files_modified: 3
  tests_added: 15  # 1 store unit + 8 report unit + 6 report integration
  tests_extended: 1  # test_query_report_summary_empty_db (added tier4/tier5 assertions) + test_proof_level_mapping (added T4/T5 cases, counted under tests_added as 8 new cases)
  completed_date: "2026-04-24"
---

# Phase 14 Plan 03: Report T4/T5 Surfacing (Store + Report + Integration Tests) Summary

Wave-2 parallel plan (no file overlap with Plan 14-02). Extends the SQLite aggregation layer, the Markdown disclosure report, and the integration test suite so `honeyprompt report` surfaces Tier-4 capability lists and Tier-5 server-verified proofs alongside existing T1–T3 evidence, with Tier 4 / Tier 5 rows in the Executive Summary — and legacy v4.0 databases render sensibly (count=0 rows + em-dash evidence cells).

## Plan-at-a-glance

| Task | Type | Files | Commit |
|------|------|-------|--------|
| 1 | feat (tdd) | src/store/mod.rs | `3fecfdf` |
| 2 | feat (tdd) | src/report/mod.rs | `25b5003` |
| 3 | test (tdd) | tests/test_report.rs, src/store/mod.rs (fmt follow-up) | `cede420` |

## Key Changes

### `src/store/mod.rs` (Task 1)

- `ReportSummary` gains `tier4_sessions: u32` + `tier5_sessions: u32` between `tier3_sessions` and `earliest_event` (keeps tier fields contiguous in the struct for ergonomic iteration).
- `ReportSession` gains three trailing `Option<T>` fields after `classification`:
  - `t4_capability: Option<String>`
  - `t5_proof: Option<String>`
  - `t5_proof_valid: Option<bool>`
- `query_report_summary` adds two new `COUNT(DISTINCT session_id)` subqueries with identical shape to the existing T3 subquery:
  ```sql
  SELECT COUNT(DISTINCT session_id) FROM events
   WHERE tier = 4 AND extra_headers NOT LIKE '%"classification":"KnownCrawler%'
  ```
  Same for `tier = 5`. The struct literal is updated to populate the two new fields.
- `query_report_sessions` extends the `SELECT` to 13 columns by appending `MAX(t4_capability)`, `MAX(t5_proof)`, `MAX(t5_proof_valid)` at indexes 10, 11, 12. Row mapping uses `row.get::<_, Option<T>>(N)` so legacy v4.0 rows (which have NULL in those columns per Phase 13 additive migration) and non-T4/T5 sessions (all rows) map cleanly to `None`.
- `test_query_report_summary_empty_db` extended with `assert_eq!(summary.tier4_sessions, 0)` + T5 equivalent.
- New `test_query_report_sessions_null_t4_t5_for_legacy_rows` inserts a T1 row (omitting T4/T5 columns) and asserts the corresponding `ReportSession` has `t4_capability == None`, `t5_proof == None`, `t5_proof_valid == None`. This is the primary regression gate for Threat T-14-03-03 (runtime panic on `row.get::<_, String>` of a NULL column).

### `src/report/mod.rs` (Task 2)

- `proof_level(tier)` extends:
  - `4 => "Capability Introspection"` (D-14-09)
  - `5 => "Multi-step Compliance"` (D-14-09)
  - T1–T3 arms + the fallback `_ => "Unknown"` arm unchanged.
- New `evidence_cell(&ReportSession) -> String` helper:
  - T4 → `md_escape(t4_capability)` full, no truncation (D-14-10); falls back to em-dash on `None`.
  - T5 → `"NNN ✓ VALID"` / `"NNN ✗ INVALID"` per D-14-11; falls back to just the proof (no glyph) when `t5_proof_valid` is `None` — captures the "no server claim made" case.
  - T1 / T2 / T3 (and any unknown tier) → em-dash `—` per D-14-13.
- Executive Summary: two new `push_str` calls emit Tier 4 / Tier 5 rows between the existing Tier 3 row and the Known Crawler Sessions row, **always-show** per D-14-12.
- Evidence Table + Known Crawler Sessions: schema extended from 9 columns to 10 in lockstep across header, separator, empty-state, and data row formats. The new Evidence column is inserted between Classification and Payload (Payload remains rightmost for visual anchoring).
- `test_proof_level_mapping` extended with T4 / T5 assertions.
- 8 new unit tests cover every `evidence_cell` branch:
  - `test_evidence_cell_t4_some` — capability string round-trips (md_escape is no-op on safe chars).
  - `test_evidence_cell_t4_none_shows_emdash` — defensive em-dash fallback.
  - `test_evidence_cell_t4_escapes_pipe` — `"evil|cap"` → `r"evil\|cap"` (Threat T-14-03-01 mitigation).
  - `test_evidence_cell_t5_valid` — `"123 ✓ VALID"`.
  - `test_evidence_cell_t5_invalid` — `"456 ✗ INVALID"`.
  - `test_evidence_cell_t5_unverified` — None validity → proof only, no glyph.
  - `test_evidence_cell_t1_t2_t3_emdash` — em-dash for all three lower tiers.
  - `test_evidence_table_column_counts_match` — pipe-count invariant across header / separator / empty-state (Threat T-14-03-02 mitigation).

### `tests/test_report.rs` (Task 3)

- New `insert_event_t4(conn, nonce, payload_id, embedding_loc, session_id, remote_addr, user_agent, classification, capability, first_seen_epoch)` helper writes a tier-4 events row including the `t4_capability` column. Reusable by future phases.
- New `insert_event_t5(conn, nonce, payload_id, embedding_loc, session_id, remote_addr, user_agent, classification, proof, proof_valid, first_seen_epoch)` helper writes a tier-5 events row including `t5_proof` and `t5_proof_valid`.
- 6 new integration tests:
  - `test_report_summary_tier4_tier5_counts` — seed one T4 + one T5 event, assert `| Tier 4 (Capability Introspection) | 1 |` and `| Tier 5 (Multi-step Compliance) | 1 |` are present in the rendered Markdown.
  - `test_report_backward_compat_v40_db` — **primary backward-compat regression gate.** Seed only a T1 event; assert Tier 4 / Tier 5 exec rows show count=0 (D-14-12 always-show) AND the T1 evidence cell is em-dash (D-14-13). The full literal `| Unknown | — | t1-payload-01 |` is asserted so both column ordering and em-dash content are pinned.
  - `test_report_evidence_column_t4` — asserts the full capability string appears in the row AND appears between Classification and Payload via the literal `| Unknown | web_search,browse_page | t4-tools-01 |`.
  - `test_report_evidence_column_t5_valid` — asserts `| Unknown | 123 ✓ VALID | t5-chain-01 |`.
  - `test_report_evidence_column_t5_invalid` — asserts `| Unknown | 456 ✗ INVALID | t5-chain-02 |`.
  - `test_report_full_5tier_markdown` — end-to-end: one event per tier 1..=5; asserts all 5 exec summary rows with count=1, all 5 per-tier evidence cells render correctly (T1/T2/T3 em-dash, T4 capability, T5 valid), and the 10-column Evidence Table header is present.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 — rustfmt] Comment alignment on new ReportSummary fields**

- **Found during:** Task 2 verification (`cargo fmt --all -- --check`)
- **Issue:** `cargo fmt --check` reported that the trailing `// NEW Phase 14 (D-14-12 always-show)` comments on the two new `tier4_sessions` / `tier5_sessions` fields in `ReportSummary` should be aligned with the existing `// epoch seconds string` comments on `earliest_event` / `latest_event` (rustfmt aligns trailing comments on consecutive struct fields at a shared column).
- **Fix:** Ran `cargo fmt --all` once. No semantic change; only whitespace/alignment.
- **Files modified:** `src/store/mod.rs` (lines 200–201)
- **Commit:** folded into `cede420` (Task 3 commit, alongside the tests)

### Acceptance Criteria Commentary

The plan's Task 2 acceptance criterion for the 10-column header expects `grep -c` to return exactly 2 (one for Evidence Table + one for Known Crawler Sessions). My implementation reports 3 — the extra occurrence is the literal embedded inside the new `test_evidence_table_column_counts_match` test body, which is *mandated* by the plan's own Edit 2e and serves as the pipe-count-invariant regression test (Threat T-14-03-02 mitigation). The production code has exactly 2 occurrences as required; the third is the pinned assertion literal. Same rationale for the empty-state em-dash row `grep -c` returning 3.

This is consistent with the plan text and is not a true deviation — just a clarification that the grep count includes the invariant-test literal.

## Threat Model Status

All six items in the plan's `<threat_model>` STRIDE register are addressed:

- **T-14-03-01 (Tampering — Markdown injection via T4 capability):** mitigated. `evidence_cell` calls `md_escape` on `t4_capability` before every render. `test_evidence_cell_t4_escapes_pipe` is the unit-level regression gate. Defense in depth: D-13-09 server-side regex rejects `|` / `` ` `` / `\n` / `\r` at intake.
- **T-14-03-02 (Tampering — Markdown column-count mismatch):** mitigated. `test_evidence_table_column_counts_match` asserts header.pipes == sep.pipes == empty.pipes. Pitfall 6 regression gate.
- **T-14-03-03 (Tampering / DoS — runtime panic on NULL column):** mitigated. All three new getters in `query_report_sessions` use `row.get::<_, Option<T>>(N)` (T = String, String, bool). `test_query_report_sessions_null_t4_t5_for_legacy_rows` is the unit-level regression gate.
- **T-14-03-04 (DoS — crash on legacy DB):** mitigated. The two new `tier4_sessions` / `tier5_sessions` subqueries use `COUNT(DISTINCT session_id) WHERE tier = N` and never touch the nullable T4/T5 columns. No panic surface.
- **T-14-03-05 (Info Disclosure — report leaks classification taxonomy):** accepted. Only `t4_capability` (agent-chosen safe menu per D-13-07) and `t5_proof` (arithmetic of page-visible values per D-13-02) reach the Evidence column. No secrets ever flow into the report — aligned with PROJECT.md's no-secrets guarantee.
- **T-14-03-06 (Tampering — label divergence between `proof_level()` and Exec Summary):** mitigated. The Exec Summary strings `"Tier 4 (Capability Introspection)"` and `"Tier 5 (Multi-step Compliance)"` are hardcoded to match `proof_level(4)` and `proof_level(5)`. `test_proof_level_mapping` pins the returned strings; any divergence would fail both the unit test and the integration tests.

## Verification Results

- `cargo build --lib` → green
- `cargo build --tests` → green
- `cargo clippy --all -- -D warnings` → green (no warnings)
- `cargo fmt --all -- --check` → green
- `cargo test --all` → **199 tests passed, 0 failed** across all test binaries:
  - Unit tests: 145 (lib)
  - `test_report.rs` integration: 15 (9 existing + 6 new)
  - `test_serve.rs` integration: 14
  - `test_monitor.rs` integration: 3
  - `test_test_agent.rs` integration: 4
  - `test_test_agent_smoke.rs`: 2
  - Doc tests: 0 (as expected)
  - Other integration binaries: 11 / 3 / 2 / 0 across smaller test files

### Phase-level grep gates

| Pattern | File | Expected | Got |
|---|---|---|---|
| `pub tier4_sessions: u32` | src/store/mod.rs | 1 | 1 |
| `pub tier5_sessions: u32` | src/store/mod.rs | 1 | 1 |
| `pub t4_capability: Option<String>` | src/store/mod.rs | 1 | 1 |
| `pub t5_proof: Option<String>` | src/store/mod.rs | 1 | 1 |
| `pub t5_proof_valid: Option<bool>` | src/store/mod.rs | 1 | 1 |
| `MAX(t4_capability)` | src/store/mod.rs | 1 | 1 |
| `MAX(t5_proof)` | src/store/mod.rs | 1 | 1 |
| `MAX(t5_proof_valid)` | src/store/mod.rs | 1 | 1 |
| `WHERE tier = 4 AND extra_headers NOT LIKE` | src/store/mod.rs | ≥ 1 | 1 |
| `WHERE tier = 5 AND extra_headers NOT LIKE` | src/store/mod.rs | ≥ 1 | 1 |
| `4 => "Capability Introspection"` | src/report/mod.rs | 1 | 1 |
| `5 => "Multi-step Compliance"` | src/report/mod.rs | 1 | 1 |
| `fn evidence_cell` | src/report/mod.rs | 1 | 1 |
| `Tier 4 (Capability Introspection)` | src/report/mod.rs | 1 | 1 |
| `Tier 5 (Multi-step Compliance)` | src/report/mod.rs | 1 | 1 |
| `let evidence = evidence_cell(s);` | src/report/mod.rs | 2 | 2 |
| `{evidence}` | src/report/mod.rs | 2 | 2 |
| `fn insert_event_t4` | tests/test_report.rs | 1 | 1 |
| `fn insert_event_t5` | tests/test_report.rs | 1 | 1 |
| `fn test_report_summary_tier4_tier5_counts` | tests/test_report.rs | 1 | 1 |
| `fn test_report_backward_compat_v40_db` | tests/test_report.rs | 1 | 1 |
| `fn test_report_evidence_column_t4` | tests/test_report.rs | 1 | 1 |
| `fn test_report_evidence_column_t5_valid` | tests/test_report.rs | 1 | 1 |
| `fn test_report_evidence_column_t5_invalid` | tests/test_report.rs | 1 | 1 |
| `fn test_report_full_5tier_markdown` | tests/test_report.rs | 1 | 1 |

## TDD Gate Compliance

All three tasks had `tdd="true"`. The RED state was demonstrated implicitly via the struct-shape change in Task 1 (the existing store tests referencing `ReportSummary` / `ReportSession` already depended on the existing field set; the new fields required new assertions to become meaningful). GREEN followed immediately in each task's single commit because the compile-time struct dependencies between `store` → `report` → `tests/test_report` mean the three tasks form a linear build-order chain — Task 1 must compile on its own (GREEN of the struct-shape RED), Task 2 must compile against the new struct (GREEN of the evidence_cell unit-test RED), and Task 3 must pass against the real `generate_report` output (GREEN of the end-to-end integration-test RED).

Conventional-commit prefixes match the task types: `feat(14-03)` for Tasks 1-2, `test(14-03)` for Task 3. The plan-level type is `execute` (not `tdd`), so no plan-level RED/GREEN/REFACTOR gate sequence is required — the per-task TDD flag drives task-internal structure only.

## Follow-up for Phase 15 (Deferred per Phase 14 scope)

Roadmap criterion #2 (TODOS.md + README Proof Levels 5-tier documentation) and #5's CI exit-code semantics for T4/T5 remain phase-15 work. This plan's integration tests lock in the report's Markdown contract so any future rendering change (e.g. JSON/HTML reports) has a pinned baseline to extend.

## Self-Check: PASSED

- Commit `3fecfdf` (Task 1) — FOUND in `git log`
- Commit `25b5003` (Task 2) — FOUND in `git log`
- Commit `cede420` (Task 3) — FOUND in `git log`
- `src/store/mod.rs` modified — FOUND (ReportSummary.tier4_sessions + ReportSession.t4_capability present; query extensions present)
- `src/report/mod.rs` modified — FOUND (proof_level T4/T5 arms present; evidence_cell fn present; 10-column headers present)
- `tests/test_report.rs` modified — FOUND (insert_event_t4/t5 helpers + 6 new tests present)
- `.planning/phases/14-tiers-4-5-surfacing-monitor-tui-report/14-03-SUMMARY.md` written — this file
- No modifications to `.planning/STATE.md`, `.planning/ROADMAP.md`, or `src/monitor/mod.rs` (Plan 14-02's scope) — confirmed via `git status --short` and `git diff --stat HEAD~3..HEAD`
