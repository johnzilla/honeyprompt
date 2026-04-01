---
phase: 11-setup-wizard-zero-config-serve
plan: "02"
subsystem: cli
tags: [rust, clap, axum, tempdir, zero-config, serve, precedence-chain]

# Dependency graph
requires:
  - phase: 11-01
    provides: Config struct in src/config/mod.rs, updated cli/mod.rs with Setup variant
  - phase: prior phases
    provides: server::serve, generator::generate, store::open_or_create_db, tempfile pattern in test_agent

provides:
  - "--domain flag on ServeArgs enabling zero-config single-command honeypot deployment"
  - "config_with_overrides function implementing flag > domain-defaults > base-config precedence chain"
  - "Tempdir serve mode: honeyprompt serve --domain example.com generates and serves without config file"
  - "Integration tests validating SERVE-01 and SERVE-02 requirements end-to-end"

affects:
  - deploy documentation (zero-config serve is the primary quick-start path)
  - future UX guides referencing --domain flag

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "config_with_overrides pure function: flag > domain-defaults > base-config — applies same pattern as test_agent tempdir build"
    - "_keep binding pattern: assign TempDir to _keep variable to keep it alive until blocking async call exits"
    - "as_os_str() comparison for PathBuf default detection (avoids cmp_owned clippy warning)"

key-files:
  created:
    - tests/serve_domain.rs
  modified:
    - src/cli/mod.rs
    - src/config/mod.rs
    - src/main.rs

key-decisions:
  - "config_with_overrides takes Option<&str> not Option<String> to minimize allocations at call sites"
  - "Tempdir mode triggers on: domain set AND path=='.'' AND no honeyprompt.toml at '.'' — explicit --path always uses standard mode"
  - "Domain implies bind=0.0.0.0:8080 and tiers=[1,2,3] as defaults, but explicit --bind/--tiers override them"

patterns-established:
  - "Pure config override function (config_with_overrides) separates precedence logic from I/O for easy unit testing"
  - "Tempdir mode reuses existing generate pipeline (same pattern as test_agent::run) — no duplication"

requirements-completed: [SERVE-01, SERVE-02, SERVE-03]

# Metrics
duration: 12min
completed: 2026-03-31
---

# Phase 11 Plan 02: Zero-Config Serve Mode Summary

**`--domain` flag on `honeyprompt serve` generates an ephemeral tempdir honeypot and serves it immediately with https://{domain} callback URLs, bind 0.0.0.0:8080, and all tiers enabled — no init/generate steps required**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-31T00:00:00Z
- **Completed:** 2026-03-31T00:12:00Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- `config_with_overrides` implements the full flag > domain-defaults > base-config precedence chain with 4 unit tests
- ServeArgs gains `--domain`, `--bind`, and `--tiers` optional flags for zero-config and override workflows
- `honeyprompt serve --domain example.com` generates a honeypot in a tempdir and serves it without any config file
- Integration tests in `tests/serve_domain.rs` prove SERVE-01 and SERVE-02 end-to-end (HTML callback URLs, GET / returns 200)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add --domain/--bind/--tiers flags and config_with_overrides** - `5643cfe` (feat)
2. **Task 2: Wire --domain serve dispatch with tempdir mode in main.rs** - `d89df5d` (feat)
3. **Task 3: Integration tests for tempdir serve with --domain** - `29d6958` (test)

## Files Created/Modified
- `src/cli/mod.rs` - ServeArgs gains `domain: Option<String>`, `bind: Option<String>`, `tiers: Option<Vec<u8>>` flags
- `src/config/mod.rs` - Added `config_with_overrides` function + 4 new precedence unit tests (6 total)
- `src/main.rs` - `Commands::Serve` arm replaced with domain-aware tempdir/standard dispatch
- `tests/serve_domain.rs` - Integration tests: callback URL correctness and HTTP 200 for GET /

## Decisions Made
- `config_with_overrides` takes `Option<&str>` for domain/bind (not owned String) to avoid unnecessary allocation at call sites
- Tempdir mode detection: domain set AND `args.path.as_os_str() == "."` AND no `./honeyprompt.toml` — ensures explicit `--path` always uses standard mode even with `--domain`
- `_keep = tmp` binding pattern (same as test_agent) prevents TempDir drop until `server::serve` completes

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed cmp_owned clippy warning for PathBuf comparison**
- **Found during:** Task 2 (main.rs dispatch wiring)
- **Issue:** `args.path == std::path::PathBuf::from(".")` created an owned PathBuf just for comparison — clippy warning
- **Fix:** Changed to `args.path.as_os_str() == "."` which compares against OsStr directly
- **Files modified:** src/main.rs
- **Verification:** `cargo clippy --all-targets` produces zero warnings
- **Committed in:** d89df5d (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - clippy correctness)
**Impact on plan:** Minor clippy fix only. No scope changes.

## Issues Encountered
None — all three tasks compiled and tested cleanly on first build after implementation.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 11 complete: setup wizard (11-01) + zero-config serve (11-02) both delivered
- `honeyprompt setup` creates honeyprompt.toml interactively
- `honeyprompt serve --domain example.com` deploys without any config file
- Ready for Phase 12 (deploy templates) or documentation updates

## Self-Check: PASSED

- FOUND: tests/serve_domain.rs
- FOUND: src/cli/mod.rs
- FOUND: src/config/mod.rs
- FOUND: src/main.rs
- FOUND: .planning/phases/11-setup-wizard-zero-config-serve/11-02-SUMMARY.md
- FOUND commit: 5643cfe (feat: --domain/--bind/--tiers flags and config_with_overrides)
- FOUND commit: d89df5d (feat: wire --domain serve dispatch with tempdir mode)
- FOUND commit: 29d6958 (test: integration tests for tempdir serve with --domain)

---
*Phase: 11-setup-wizard-zero-config-serve*
*Completed: 2026-03-31*
