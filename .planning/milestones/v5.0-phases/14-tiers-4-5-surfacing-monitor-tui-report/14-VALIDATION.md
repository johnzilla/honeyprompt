---
phase: 14
slug: tiers-4-5-surfacing-monitor-tui-report
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-24
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (Rust stable) |
| **Config file** | `Cargo.toml` (already present) |
| **Quick run command** | `cargo test --lib -- <module>::tests` |
| **Full suite command** | `cargo test && cargo clippy -- -D warnings && cargo fmt --check` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib -- <touched_module>::tests`
- **After every plan wave:** Run `cargo test && cargo clippy -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green (`cargo test && cargo clippy -- -D warnings && cargo fmt --check`)
- **Max feedback latency:** ~30 seconds

---

## Per-Task Verification Map

*Populated by the planner during PLAN.md generation. See plans in this directory for the binding task→test map.*

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| *TBD* | — | — | UI-01..05 | — | additive rendering, no regression in T1–T3 | unit + integration | `cargo test` | ⬜ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/integration_report_t4_t5.rs` — end-to-end report test: seed DB with T1+T4+T5 rows, run `generate_report`, assert Evidence column values per tier
- [ ] `tests/integration_report_legacy.rs` — backward-compat test: seed DB with only T1–T3 rows (NULLs in T4/T5 columns), run `generate_report`, assert output has exec-summary T4/T5 rows with count 0 and no empty "sections" (per D-14-12 interpretation of success criterion #5)
- [ ] `src/monitor/mod.rs::tests` — new unit tests for `tier_counts()` returning 5 tiers, `TierFilter::next()` cycle, and EVIDENCE cell content helpers
- [ ] `src/report/mod.rs::tests::test_proof_level_t4` / `test_proof_level_t5` — extend existing `test_proof_level_mapping` analog
- [ ] `src/store/mod.rs::tests` — `query_report_summary` extension tests (tier4_sessions / tier5_sessions counts) and NULL-safe `query_report_sessions` extensions

*If the planner can fold any of these into existing test files, preferred.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| TUI visual rendering — EVIDENCE column truncation, detail pane layout, color glyphs | UI-01, UI-02 | Ratatui render inspection requires a live terminal; automated snapshot tests would couple to terminal backend | 1. Run `honeyprompt setup` (test config). 2. Seed DB with sample T1–T5 events (see `tests/fixtures/mixed_tiers.sql`). 3. Run `honeyprompt monitor --db <path>`. 4. Verify: EVIDENCE column shows `web_search,…` for T4 and `123 ✓` (green) / `123 ✗` (red) for T5; detail pane updates on `j`/`k` navigation with T4 full list, T5 proof+formula, T1–T3 payload metadata. 5. Press Tab to cycle All→T1→T2→T3→T4→T5→All; confirm filter label highlights advance correctly. 6. `?` to view help overlay; confirm Tab description lists all 5 tiers and `:filter t4`/`:filter t5` commands appear. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags (`cargo test` is one-shot)
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
