---
phase: 1
slug: generation-pipeline
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust test framework) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | CLI-01 | integration | `cargo test init` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | CLI-02 | integration | `cargo test generate` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | GEN-01 | unit | `cargo test template` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | GEN-03 | unit | `cargo test nonce` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | SRV-02 | unit | `cargo test schema` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/` directory — integration test setup
- [ ] `src/lib.rs` — library root for unit tests
- [ ] Test fixtures for payload catalog verification

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visible human warning renders correctly | GEN-02 | Visual HTML rendering | Open generated index.html in browser, verify banner and inline notice visible |
| Payloads in correct embedding locations | GEN-06 | HTML structure inspection | View page source, verify payloads in HTML comments, meta tags, JSON-LD, etc. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
