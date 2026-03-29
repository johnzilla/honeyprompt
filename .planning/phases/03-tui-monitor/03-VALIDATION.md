---
phase: 03
slug: tui-monitor
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 03 — Validation Strategy

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
| TBD | TBD | TBD | CLI-04 | integration | `cargo test --test test_monitor` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | TUI-01 | unit | `cargo test tui` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | TUI-02 | unit | `cargo test tui::filter` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_monitor.rs` — integration test stubs for CLI-04
- [ ] `src/tui/mod.rs` — unit test stubs for TUI-01, TUI-02

*Existing test infrastructure (cargo test) covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Live event table visual updates | TUI-01 | Requires running server + visual confirmation | Start monitor, trigger callbacks, observe table updates |
| Replay row visual flagging | TUI-02 | Visual appearance requires human judgment | Trigger replay event, toggle replay visibility, confirm visual treatment |
| Screenshot-worthy appearance | Phase goal | Subjective visual quality | Take screenshot, evaluate demo readiness |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
