---
phase: 5
slug: test-agent-subcommand
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust test harness) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

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
| TBD | TBD | TBD | TEST-01 | integration | `cargo test test_agent` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | TEST-02 | integration | `cargo test test_agent` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | TEST-03 | unit | `cargo test scorecard` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | TEST-04 | integration | `cargo test test_agent` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | TEST-05 | unit | `cargo test json_format` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | REL-01 | CI | `.github/workflows/ci.yml` exists | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_test_agent.rs` — integration test stubs for test-agent subcommand
- [ ] Existing test infrastructure covers unit tests (14 tests already passing)

*Existing infrastructure covers framework requirements — only new test files needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CI badge visible on GitHub | REL-01 | Requires push to GitHub | Push a commit and verify badge appears in repo |
| Exit code 2 on error | TEST-04 | Error paths hard to trigger in CI | Run `honeyprompt test-agent` on invalid dir, verify `echo $?` returns 2 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
