---
phase: 12-documentation-deploy-templates
plan: "01"
subsystem: docs
tags: [readme, documentation, self-hosted, deploy, docker, systemd, caddy]

requires:
  - phase: 11-setup-wizard-zero-config-serve
    provides: honeyprompt setup wizard and --domain zero-config serve mode that this README documents

provides:
  - README.md with complete Deploy Your Own guide (Install, Configure, Deploy, Verify)
  - Persona separation between live demo (honeyprompt.sh) and self-hosted path
  - Project Status table updated through Phase 12
  - Usage section updated with honeyprompt setup and --domain flag

affects: [12-02, future docs, project discoverability]

tech-stack:
  added: []
  patterns:
    - "Deploy Your Own section pattern: four numbered subsections (Install, Configure, Deploy, Verify)"
    - "Persona separation: Live Demo callout frames honeyprompt.sh as 'see it in action', Deploy Your Own uses {your-domain.com} exclusively"
    - "{DOMAIN} placeholder pattern referenced in deploy/templates/ copy-paste workflow"

key-files:
  created: []
  modified:
    - README.md

key-decisions:
  - "deploy/templates/ referenced as canonical location for Docker Compose, Caddyfile, and systemd unit files"
  - "Zero-config (--domain) documented as Option B in Configure step, framed as 'quick trial' vs wizard for production"
  - "honeyprompt.sh appears only in Live Demo callout and Phase 7 Project Status row — not in Deploy Your Own section"

patterns-established:
  - "Deploy Your Own structure: 1-Install, 2-Configure (wizard + zero-config), 3-Deploy (3 paths), 4-Verify"

requirements-completed: [DOCS-01, DOCS-02]

duration: 12min
completed: 2026-03-31
---

# Phase 12 Plan 01: README Deploy Your Own Guide Summary

**README rewritten with four-subsection Deploy Your Own guide, clear persona separation between honeyprompt.sh demo and self-hosted path, and Project Status table extended through Phase 12.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-03-31T00:00:00Z
- **Completed:** 2026-03-31T00:12:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Added complete Deploy Your Own section covering Install, Configure (wizard + zero-config), Deploy (3 paths), and Verify
- Documented `honeyprompt setup` and `honeyprompt serve --domain` as the v4.0 self-hosted entry points
- Referenced `deploy/templates/` for Docker Compose, Caddyfile, and systemd unit file copy-paste workflows
- Separated live demo persona (honeyprompt.sh) from self-hosted persona — honeyprompt.sh appears only in the Live Demo callout and Phase 7 status row
- Extended Project Status table with phases 9-12 marked Complete
- Added `honeyprompt setup` and `--domain` flag to Usage section for discoverability

## Task Commits

1. **Task 1: Rewrite README with Deploy Your Own guide and persona separation** - `bfa3403` (feat)

**Plan metadata:** (see final docs commit below)

## Files Created/Modified

- `README.md` — Added Deploy Your Own section, updated Usage and Project Status, persona separation throughout

## Decisions Made

- Referenced `deploy/templates/` throughout (consistent with plan 12-02 which will populate that directory)
- Framed zero-config `--domain` as "Option B" (quick trial) and wizard as "Option A" (production), matching UX intent from Phase 11

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Plan 12-01 complete: README Deploy Your Own guide is live
- Plan 12-02 (deploy templates) can now create `deploy/templates/` directory with the Docker Compose, Caddyfile, and systemd files that this README references
- All DOCS-01 and DOCS-02 requirements satisfied

---
*Phase: 12-documentation-deploy-templates*
*Completed: 2026-03-31*
