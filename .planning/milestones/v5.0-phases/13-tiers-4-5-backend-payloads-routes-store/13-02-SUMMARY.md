---
phase: 13-tiers-4-5-backend-payloads-routes-store
plan: 02
subsystem: database
tags: [sqlite, rusqlite, migration, additive, user-version, replay, first-write-wins]

requires:
  - phase: 13-01
    provides: N/A — Plan 02 is compilation-independent of Plan 01 (no Tier4/Tier5 enum references in this plan)
provides:
  - "Additive v4.0→v5.0 schema migration gated by PRAGMA user_version"
  - "Three new nullable columns on events: t4_capability TEXT, t5_proof TEXT, t5_proof_valid INTEGER"
  - "insert_callback_event signature extended with three trailing Option<_> parameters"
  - "ON CONFLICT first-write-wins semantics preserved for T4/T5 (D-13-19)"
  - "Compilation-bridge None placeholders in src/broker/mod.rs (Plan 04 replaces with real values)"
  - "tests/test_migration.rs integration tests proving v4.0 DBs open unchanged"
affects: [13-04 (server handlers populate the new columns)]

tech-stack:
  added: []
  patterns: ["PRAGMA user_version migration gating", "additive ALTER TABLE ADD COLUMN only (never DROP)", "compilation-bridge Option placeholders for cross-wave signatures"]

key-files:
  created: ["tests/test_migration.rs"]
  modified: ["src/store/mod.rs", "src/broker/mod.rs"]

key-decisions:
  - "Migration is idempotent via PRAGMA user_version < 1 guard — running run_migrations twice is safe"
  - "T4/T5 columns deliberately excluded from ON CONFLICT DO UPDATE SET clause to preserve first-write-wins per D-13-19"
  - "Broker passes None for all three new Options — real wiring deferred to Plan 04 to keep plan 02 surface minimal"

patterns-established:
  - "Additive migration via user_version gating: all future schema changes append version-gated blocks, never mutate existing columns"
  - "Compilation-bridge placeholder pattern: downstream-wave signature extensions land with None/default values so the crate compiles end-to-end between waves"
  - "Integration test for migration back-compat: programmatically construct prior-version schema, run run_migrations, assert both schema evolution AND existing-row preservation"

requirements-completed: [STORE-01, STORE-02, STORE-03, STORE-04]

duration: ~25min (including sandbox recovery by orchestrator)
completed: 2026-04-24
---

# Phase 13-02: Store migration + insert signature + broker bridge Summary

**Additive v4.0→v5.0 SQLite migration via PRAGMA user_version gating, T4/T5 columns added to events table, insert_callback_event signature extended with first-write-wins replay preservation.**

## Performance

- **Duration:** ~25 min (including orchestrator-side commit recovery after mid-run sandbox block)
- **Tasks:** 3/3 complete
- **Files modified:** 3 (src/store/mod.rs, src/broker/mod.rs, tests/test_migration.rs)
- **Test results:** `cargo test` — 147 tests pass, 0 failures (105 lib + 2 migration + 40 other integration tests)
- **Build:** `cargo build --lib` passes; `cargo clippy --all-targets --lib -- -D warnings` clean; `cargo fmt --check` clean

## Accomplishments

- `run_migrations` extended with a `user_version < 1` guarded block that issues three `ALTER TABLE events ADD COLUMN` statements, then bumps `PRAGMA user_version = 1`. Idempotent across repeated calls.
- `insert_callback_event` signature gains three trailing `Option<_>` parameters: `t4_capability: Option<&str>`, `t5_proof: Option<&str>`, `t5_proof_valid: Option<bool>`. Existing T1–T3 call sites unaffected.
- `ON CONFLICT(nonce) DO UPDATE SET` clause contains only `last_seen_at, fire_count, is_replay` — the three new T4/T5 columns are deliberately excluded (first-write-wins per D-13-19).
- `src/broker/mod.rs::db_writer_task` updated to pass `None, None, None` as the new trailing parameters so the crate compiles end-to-end with no Plan 04 work yet wired.
- `tests/test_migration.rs` — new integration test file with `test_v4_db_opens_unchanged` (programmatically builds v4.0 schema, inserts T1 row, runs migration, asserts row byte-identical and new columns nullable) and `test_v4_db_migration_idempotent_across_reopen` (re-opens migrated DB and confirms second `run_migrations` call is a no-op).

## Task Commits

TDD style with explicit RED/GREEN sequencing per GSD TDD mode:

1. **Task 1 RED: failing schema migration tests** — `ac312d3` (test) — new store-module tests asserting the three new columns exist after migration; failed against unmodified `run_migrations`.
2. **Task 1 GREEN: additive migration** — `b0b3634` (feat) — extended `run_migrations` with `user_version`-gated `ALTER TABLE ADD COLUMN` block; RED tests now pass.
3. **Task 2 RED: failing first-write-wins replay tests** — `9cceb68` (test) — tests asserting T4/T5 columns are NOT overwritten by replay upserts; failed until signature extension landed.
4. **Task 2 GREEN: insert signature + broker placeholders** — `c6f4bcd` (feat) — extended `insert_callback_event` signature with three trailing `Option<_>` params, broker updated to pass `None` placeholders, SET clause left unchanged so replay tests pass.
5. **Task 3: migration integration tests** — `0218436` (test) — `tests/test_migration.rs` with two integration tests proving v4.0 DB back-compat; also includes `cargo fmt` whitespace normalization of `src/store/mod.rs` from prior task commits.

## Files Created/Modified

- `src/store/mod.rs` — `run_migrations` extended with user_version-gated ALTER TABLE block; `insert_callback_event` signature + binds extended; 4 new unit tests for schema + idempotency + replay semantics; fmt normalization.
- `src/broker/mod.rs` — `db_writer_task` call to `insert_callback_event` updated with three trailing `None` placeholders.
- `tests/test_migration.rs` (new) — 2 integration tests proving v4.0 DBs open unchanged and migration is idempotent across reopen.

## Decisions Made

- Used `PRAGMA user_version` gating rather than `ALTER TABLE ADD COLUMN IF NOT EXISTS` (SQLite does not support the latter — confirmed in RESEARCH.md).
- Chose `t5_proof_valid INTEGER` over `BOOLEAN` to match SQLite's canonical boolean storage (0/1 INTEGER) already used by `is_replay` elsewhere in the schema.
- Programmatically constructed v4.0 schema in the integration test rather than shipping a binary fixture to avoid repo bloat (per RESEARCH.md Wave 0 Gaps recommendation).

## Deviations from Plan

**None — plan executed exactly as written.** The only off-plan activity was orchestrator-side completion of the final commit (Task 3 + SUMMARY.md) after the executor subagent hit a mid-run Bash sandbox restriction on `git add`/`git commit`. All test outputs, file contents, and TDD sequencing followed the plan verbatim.

## Issues Encountered

- **Sandbox regression mid-session:** The executor subagent successfully ran `git add`/`git commit` for the first 4 of 5 commits, then — partway through Task 3 — those commands began returning "Permission to use Bash has been denied." Read-only git commands continued working. The executor surfaced the block cleanly, presented verified state (all tests passing), and the orchestrator completed the Task 3 commit and authored this SUMMARY. No work was lost.

## User Setup Required

None — no external service configuration changed.

## Next Phase Readiness

- Plan 04 can now populate the three new `Option<_>` parameters with real values from the T4/T5 handlers, replacing the `None` placeholders currently in `src/broker/mod.rs::db_writer_task`.
- Plan 04 must take care to preserve the SET-clause exclusion of T4/T5 columns when wiring handlers (existing first-write-wins invariant already locked by this plan's tests).

---
*Phase: 13-tiers-4-5-backend-payloads-routes-store*
*Completed: 2026-04-24*
