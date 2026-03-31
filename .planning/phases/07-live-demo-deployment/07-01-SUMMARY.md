---
phase: 07-live-demo-deployment
plan: 01
subsystem: infra
tags: [systemd, caddy, docker, musl, distroless, tls, reverse-proxy, deployment]

# Dependency graph
requires:
  - phase: 06-release-infrastructure
    provides: musl static binary via GitHub Releases (the binary deployed by these configs)
provides:
  - deploy/honeyprompt.service — systemd unit for persistent honeyprompt serve with Restart=always, SIGINT graceful shutdown, sandbox hardening
  - deploy/Caddyfile — Caddy reverse proxy config for honeyprompt.sh with automatic Let's Encrypt TLS
  - Dockerfile — multi-stage distroless container image for local testing
affects: [07-live-demo-deployment, 08-readme-launch]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "KillSignal=SIGINT in systemd unit to match server shutdown_signal() which listens for SIGINT not SIGTERM"
    - "ExecStart landing dir under /var/lib/honeyprompt so derived DB path stays within ReadWritePaths"
    - "Multi-stage Dockerfile: rust:1.87-slim builder + gcr.io/distroless/static-debian12 runtime"
    - "RUSTFLAGS=-C target-feature=+crt-static for fully static musl binary in Docker"

key-files:
  created:
    - deploy/honeyprompt.service
    - deploy/Caddyfile
    - Dockerfile
  modified: []

key-decisions:
  - "ExecStart uses /var/lib/honeyprompt/landing (not /etc/honeyprompt/landing from research example) so DB at /var/lib/honeyprompt/landing/.honeyprompt/events.db stays within ReadWritePaths=/var/lib/honeyprompt — satisfies D-04 persistent evidence store"
  - "KillSignal=SIGINT required because shutdown_signal() in src/server/mod.rs listens for SIGINT (Ctrl+C) not SIGTERM — without this, systemctl stop bypasses graceful shutdown and risks losing in-flight mpsc channel events"
  - "distroless/static-debian12 over scratch: includes CA certs needed by tokio/rusqlite bundled, avoids dynamic linker failure per Pitfall 5"
  - "No CMD in Dockerfile — user passes subcommand at runtime (docker run honeyprompt serve /data)"

patterns-established:
  - "deploy/ directory holds all deployment config artifacts (service, Caddyfile)"
  - "Dockerfile at repo root for container-based local testing"

requirements-completed: [DEPLOY-01, DEPLOY-03]

# Metrics
duration: 1min
completed: 2026-03-31
---

# Phase 7 Plan 01: Deployment Config Files Summary

**systemd unit with KillSignal=SIGINT + Caddyfile reverse proxy + distroless Dockerfile for honeyprompt.sh live deployment**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-31T02:55:08Z
- **Completed:** 2026-03-31T02:56:07Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Created `deploy/honeyprompt.service` with Restart=always, KillSignal=SIGINT (matched to server's shutdown_signal SIGINT listener), sandbox hardening (ProtectSystem=strict, NoNewPrivileges, PrivateTmp), and ExecStart path that keeps SQLite DB within ReadWritePaths
- Created `deploy/Caddyfile` — 3-line config that auto-provisions Let's Encrypt TLS for honeyprompt.sh and reverse-proxies to localhost:8080
- Created `Dockerfile` with multi-stage musl/distroless build targeting sub-20MB image, verified no regressions (88/88 tests pass)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create systemd unit and Caddyfile** - `446ac75` (feat)
2. **Task 2: Create Dockerfile for local testing** - `d502916` (feat)

**Plan metadata:** (docs commit — pending)

## Files Created/Modified
- `deploy/honeyprompt.service` — systemd unit for persistent honeyprompt serve process on DigitalOcean Droplet
- `deploy/Caddyfile` — Caddy TLS reverse proxy config for honeyprompt.sh
- `Dockerfile` — multi-stage build: rust:1.87-slim builder, gcr.io/distroless/static-debian12 runtime, x86_64-unknown-linux-musl target

## Decisions Made
- ExecStart argument is `/var/lib/honeyprompt/landing` (plan resolution of research open question): DB resolves to `/var/lib/honeyprompt/landing/.honeyprompt/events.db`, which is within `ReadWritePaths=/var/lib/honeyprompt` and satisfies the D-04 persistent evidence store requirement
- KillSignal=SIGINT is critical: `src/server/mod.rs shutdown_signal()` uses `tokio::signal::ctrl_c()` (SIGINT only). Without this directive, `systemctl stop` sends SIGTERM which bypasses graceful shutdown, risking loss of in-flight events in the mpsc channel
- `distroless/static-debian12` chosen over `scratch` per Pitfall 5: includes CA certs and avoids dynamic linker failures with bundled rusqlite/tokio

## Deviations from Plan

None — plan executed exactly as written. The open questions documented in RESEARCH.md were pre-resolved by the plan's action section with explicit values (ExecStart path, KillSignal).

## Issues Encountered

None. All 88 cargo tests passed without modification.

## User Setup Required

None — no external service configuration required. The deployment runbook (Plan 02) will document the manual DigitalOcean provisioning steps.

## Next Phase Readiness
- All three deployment config files are committed and verified
- Plan 02 (deployment runbook + UptimeRobot setup) can proceed immediately
- These files are the prerequisites for the deployment runbook (D-07)
- Droplet provisioning remains a human step (D-01) — documented in Plan 02

---
*Phase: 07-live-demo-deployment*
*Completed: 2026-03-31*
