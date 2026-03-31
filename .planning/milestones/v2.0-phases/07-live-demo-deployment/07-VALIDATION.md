---
phase: 7
slug: live-demo-deployment
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-31
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | grep/file existence checks (config files, no code) |
| **Config file** | n/a |
| **Quick run command** | `ls deploy/honeyprompt.service deploy/Caddyfile deploy/README-deploy.md` |
| **Full suite command** | `grep -q "KillSignal=SIGINT" deploy/honeyprompt.service && grep -q "reverse_proxy" deploy/Caddyfile` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Run quick check (file existence)
- **After every plan wave:** Run full grep validation
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 1 second

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | DEPLOY-01 | file+grep | `grep -q "KillSignal=SIGINT" deploy/honeyprompt.service` | ✅ existing infra | ⬜ pending |
| 07-01-02 | 01 | 1 | DEPLOY-01 | file+grep | `test -f Dockerfile && grep -q "ENTRYPOINT" Dockerfile` | ✅ existing infra | ⬜ pending |
| 07-02-01 | 02 | 2 | DEPLOY-02 | file | `test -f deploy/README-deploy.md` | ✅ existing infra | ⬜ pending |
| 07-02-02 | 02 | 2 | DEPLOY-02, DEPLOY-03 | manual | Human deploys and verifies HTTPS | n/a | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements — no new test framework needed. This phase creates config files and documentation, not executable code.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| honeyprompt.sh serves HTTPS | DEPLOY-02 | Requires live droplet with DNS | SSH to droplet, deploy per runbook, curl https://honeyprompt.sh |
| Auto-restart on failure | DEPLOY-03 | Requires systemd on live droplet | `systemctl kill honeyprompt`, wait 5s, verify running again |
| UptimeRobot monitoring | DEPLOY-03 | External service configuration | Set up monitor at uptimerobot.com, verify alert on downtime |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 1s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-03-31
