---
phase: 12-documentation-deploy-templates
plan: "02"
subsystem: infra
tags: [docker-compose, caddy, systemd, deployment, templates]

requires: []
provides:
  - deploy/templates/docker-compose.yml with {DOMAIN} placeholder and serve --domain command
  - deploy/templates/Caddyfile with {DOMAIN} replacing hardcoded honeyprompt.sh
  - deploy/templates/honeyprompt.service with --domain flag in ExecStart and KillSignal=SIGINT
affects:
  - documentation
  - self-hosted-ux

tech-stack:
  added: []
  patterns:
    - "{DOMAIN} placeholder pattern for parameterized deployment templates"

key-files:
  created:
    - deploy/templates/docker-compose.yml
    - deploy/templates/Caddyfile
    - deploy/templates/honeyprompt.service
  modified: []

key-decisions:
  - "Templates use {DOMAIN} placeholder that users globally-replace — no templating engine required"
  - "docker-compose.yml command uses serve --domain {DOMAIN} for zero-config tempdir mode"
  - "honeyprompt.service drops positional path arg in favor of --domain flag, matching v4.0 zero-config UX"
  - "KillSignal=SIGINT retained and documented with explanation of why SIGTERM is insufficient"

patterns-established:
  - "{DOMAIN} placeholder: all three deploy templates use {DOMAIN} consistently for domain substitution"
  - "Inline comment documentation: each template explains non-obvious configuration with comments"

requirements-completed: [DEPLOY-01]

duration: 5min
completed: 2026-04-01
---

# Phase 12 Plan 02: Deploy Templates Summary

**Three parameterized deployment templates in deploy/templates/ — Docker Compose + Caddy + systemd — each using {DOMAIN} placeholder users replace once to get a working self-hosted honeyprompt instance**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-04-01T23:11:00Z
- **Completed:** 2026-04-01T23:11:11Z
- **Tasks:** 1
- **Files modified:** 3 (created)

## Accomplishments

- Created deploy/templates/docker-compose.yml with `command: ["serve", "--domain", "{DOMAIN}"]` and named hp-db volume for SQLite persistence
- Created deploy/templates/Caddyfile replacing hardcoded honeyprompt.sh with {DOMAIN}, with note on Caddy auto-TLS provisioning
- Created deploy/templates/honeyprompt.service updating ExecStart to use --domain flag, correcting Documentation URL to johnzilla, and documenting KillSignal=SIGINT rationale

## Task Commits

1. **Task 1: Create deploy/templates/ with parameterized deployment files** - `97895cf` (feat)

**Plan metadata:** (pending docs commit)

## Files Created/Modified

- `deploy/templates/docker-compose.yml` - Docker Compose template with {DOMAIN} in command, hp-db volume, and usage comments
- `deploy/templates/Caddyfile` - Caddy reverse proxy template with {DOMAIN} replacing honeyprompt.sh
- `deploy/templates/honeyprompt.service` - Systemd unit template with --domain flag, KillSignal=SIGINT, and all sandbox hardening

## Decisions Made

- Templates use the {DOMAIN} literal placeholder pattern — users do a global find-and-replace rather than a setup wizard. Simpler, editor-agnostic, requires no tooling.
- docker-compose.yml uses `command: ["serve", "--domain", "{DOMAIN}"]` to match the new v4.0 zero-config serve mode (tempdir generation) rather than the old positional path argument.
- honeyprompt.service ExecStart was updated from the old `serve /var/lib/honeyprompt/landing` form to `serve --domain {DOMAIN}`, aligning with v4.0 UX. A comment shows how to add `--path` for persistence.
- Documentation URL corrected from `jthwho/honeyprompt` (old/incorrect) to `johnzilla/honeyprompt`.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All three template files exist in deploy/templates/ with {DOMAIN} placeholders
- Templates complement the README "Deploy Your Own" guide from Plan 12-01
- A user can: copy the three files, run `sed -i 's/{DOMAIN}/my.domain.com/g' *`, and have working configs
- Phase 12 is now complete (both plans executed)

---
*Phase: 12-documentation-deploy-templates*
*Completed: 2026-04-01*
