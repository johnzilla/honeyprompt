---
phase: 04
slug: report-and-landing
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 04 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | CLI-05 | integration | `cargo test --test test_report` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | RPT-01 | unit | `cargo test report` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | RPT-02 | unit | `cargo test report` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | LAND-01 | integration | `cargo test landing` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_report.rs` — integration test stubs for CLI-05
- [ ] `src/report/mod.rs` — unit test stubs for RPT-01, RPT-02

*Existing test infrastructure (cargo test) covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Report readability | RPT-01 | Subjective document quality | Open generated report.md, verify it reads as a professional disclosure artifact |
| Landing page canaries | LAND-01 | Requires running server + agent trigger | Deploy landing page, run serve, verify canary callbacks work |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
