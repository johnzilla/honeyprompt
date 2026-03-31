---
phase: 05-test-agent-subcommand
plan: 03
subsystem: cli
tags: [rust, clap, serde_json, scorecard, test-agent]

# Dependency graph
requires:
  - phase: 05-02
    provides: Scorecard struct with tiers/tier_counts/listened_secs/url, OutputFormat enum, TestAgentArgs, run() orchestrator

provides:
  - Scorecard::render_text() — human-readable tier summary per D-03
  - Scorecard::render_json() — structured JSON output per D-04 (no callbacks array)
  - Commands::TestAgent dispatch wired to call render_text/render_json per --format flag
  - D-05-compliant exit codes (0/1/2) wired in main dispatch

affects: [test-agent-scorecard, ci-integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Scorecard rendering split from lifecycle: render_text/render_json are pure methods on Scorecard, called in main dispatch
    - Test module at end of file to satisfy clippy::items_after_test_module
    - Config struct initializer with ..Config::default() instead of field mutation (satisfies clippy::field_reassign_with_default)

key-files:
  created: []
  modified:
    - src/test_agent/mod.rs (render_text, render_json, 6 unit tests)
    - src/main.rs (Commands::TestAgent dispatch wired to render + exit)

key-decisions:
  - "Moved test module to end of file to satisfy clippy::items_after_test_module lint"
  - "Used Config struct initializer with ..Config::default() to fix clippy::field_reassign_with_default lint from Plan 02"
  - "Pre-existing clippy/fmt errors in monitor/mod.rs, store/mod.rs, broker/mod.rs are out-of-scope — logged to deferred-items"

patterns-established:
  - "render_text/render_json are pure Scorecard methods; main dispatch is the only call site"
  - "exit_code() returns 0 (no callbacks), 1 (any triggered), main exits 2 on error (D-05)"

requirements-completed: [TEST-03, TEST-04, TEST-05]

# Metrics
duration: 12min
completed: 2026-03-30
---

# Phase 05 Plan 03: Scorecard Rendering Summary

**Per-tier text and JSON scorecard rendering wired to honeyprompt test-agent with D-05 exit codes (0/1/2)**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-03-30T13:30:00Z
- **Completed:** 2026-03-30T13:42:00Z
- **Tasks:** 2 of 2
- **Files modified:** 2 (src/test_agent/mod.rs, src/main.rs)

## Accomplishments

- Added `render_text()` to `Scorecard` — prints tier 1/2/3 pass/fail, score fraction, and verdict string per D-03
- Added `render_json()` to `Scorecard` — emits `{listened_secs, url, tiers[], score, verdict}` JSON per D-04 (no callbacks key)
- Wired `Commands::TestAgent` in `main.rs` to call render based on `--format` flag and exit with D-05 exit codes
- 6 unit tests covering all verdict variants, text tier lines, JSON schema, and D-04 no-callbacks constraint

## Task Commits

1. **Task 1: Implement scorecard text and JSON rendering** - `55202e7` (feat)
2. **Task 2: Wire scorecard output and exit codes into main dispatch** - `4575573` (feat)

## Files Created/Modified

- `src/test_agent/mod.rs` - Added render_text(), render_json(), and 6 unit tests; moved tests module to end; fixed clippy field_reassign_with_default
- `src/main.rs` - Updated Commands::TestAgent arm to render scorecard per --format and exit with correct codes

## Decisions Made

- Moved the `#[cfg(test)] mod tests` block to the end of `src/test_agent/mod.rs` to satisfy the `clippy::items_after_test_module` lint (which was triggered by placing tests between the Scorecard impl and the run/run_async functions)
- Fixed pre-existing `clippy::field_reassign_with_default` warning in the `run()` function by replacing `Config::default()` + field mutation with a struct initializer using `..Config::default()`
- Pre-existing clippy errors in `src/monitor/mod.rs` (too_many_arguments, map_or, complex_type, unnecessary_closure) and `src/store/mod.rs` (too_many_arguments) are out-of-scope for this plan — logged to deferred-items

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed clippy::items_after_test_module — tests block was before run_async**
- **Found during:** Task 1 (render_text/render_json implementation)
- **Issue:** The test module was inserted between the Scorecard impl and the run()/run_async() functions, triggering clippy::items_after_test_module which is a hard error with -D warnings
- **Fix:** Rewrote file with test module moved to the very end (after run_async)
- **Files modified:** src/test_agent/mod.rs
- **Verification:** cargo clippy --all-targets exits without the items_after_test_module error
- **Committed in:** 4575573 (Task 2 commit)

**2. [Rule 1 - Bug] Fixed clippy::field_reassign_with_default in run()**
- **Found during:** Task 2 (clippy check after wiring main.rs)
- **Issue:** Plan 02 code used `let mut cfg = Config::default(); cfg.field = ...; cfg.field = ...;` which triggers clippy::field_reassign_with_default with -D warnings
- **Fix:** Changed to struct initializer `Config { field1: ..., field2: ..., ..Config::default() }`
- **Files modified:** src/test_agent/mod.rs
- **Verification:** cargo clippy no longer emits field_reassign_with_default for test_agent/mod.rs
- **Committed in:** 4575573 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 - Bug, fixing clippy hard errors introduced by Plan 02)
**Impact on plan:** Both fixes necessary for clippy -D warnings compliance. No scope creep.

## Issues Encountered

- `cargo clippy --all-targets -- -D warnings` has 4 pre-existing errors in `src/monitor/mod.rs` and `src/store/mod.rs` (too_many_arguments x2, map_or, complex_type, unnecessary_closure, bool_assert_comparison). These existed before Phase 05 changes and are out-of-scope. Logged to deferred-items.
- `cargo fmt --all -- --check` has widespread pre-existing formatting differences across broker, catalog, cli, crawler_catalog, fingerprint, generator, monitor modules. None are in files I modified. Logged to deferred-items.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `honeyprompt test-agent --timeout N` now prints a formatted scorecard to stdout and exits with D-05 exit codes
- `honeyprompt test-agent --timeout N --format json` emits machine-readable JSON for CI pipelines
- Phase 05 plan set is complete — TEST-01 through TEST-05 requirements fulfilled
- Pre-existing clippy/fmt errors across monitor, store, broker need attention before CI enforcement

## Known Stubs

None — all scorecard fields are wired from actual SQLite query results (`detections_by_tier()` → `tier_counts` → `tiers`). No placeholder data flows to output.

## Self-Check: PASSED

- FOUND: src/test_agent/mod.rs contains render_text (confirmed)
- FOUND: src/test_agent/mod.rs contains render_json (confirmed)
- FOUND: src/main.rs calls render_text() and render_json() via OutputFormat match
- FOUND: src/main.rs calls std::process::exit(scorecard.exit_code()) and exit(2)
- FOUND: git commit 55202e7 exists (Task 1)
- FOUND: git commit 4575573 exists (Task 2)
- All 6 unit tests pass: cargo test --lib -- test_agent::tests (6 passed)

---
*Phase: 05-test-agent-subcommand*
*Completed: 2026-03-30*
