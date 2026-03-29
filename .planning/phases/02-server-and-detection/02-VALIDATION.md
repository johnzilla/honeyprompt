---
phase: 2
slug: server-and-detection
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust test framework) |
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
| TBD | TBD | TBD | CLI-03 | integration | `cargo test serve` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SRV-01 | integration | `cargo test server` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SRV-03 | unit | `cargo test fingerprint` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SRV-04 | unit | `cargo test crawler` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SRV-05 | unit | `cargo test suppression` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SRV-06 | unit | `cargo test session` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SRV-07 | unit | `cargo test metadata` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/test_serve.rs` — integration test stubs for serve command
- [ ] Test fixtures for crawler catalog and session model

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Graceful shutdown with stats | D-11 | Requires Ctrl+C signal handling | Run `honeyprompt serve`, send callback, press Ctrl+C, verify summary stats printed |
| Startup output shows all details | D-09 | Visual output verification | Run `honeyprompt serve`, verify bind address, payload count, nonce count, DB path displayed |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
