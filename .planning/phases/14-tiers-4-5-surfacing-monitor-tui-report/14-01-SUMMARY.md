---
phase: 14
plan: 01
subsystem: types/server/broker/monitor
tags: [t5-formula, event-pipeline, bug-fix, monitor-tui, phase-14]
requires:
  - Phase 13 NonceMeta.t5_formula (already shipped; commit 0d6955f)
  - crate::catalog::load_catalog() (already shipped)
  - T5Formula struct (types.rs, Phase 13)
provides:
  - RawCallbackEvent.t5_formula: Option<T5Formula>
  - AppEvent.t5_formula: Option<T5Formula>
  - Broker one-to-one propagation (raw.t5_formula -> app_event.t5_formula)
  - Integrated-mode monitor catalog load + payload_id -> T5Formula lookup
affects:
  - Plan 14-02 (TUI detail pane) — can now consume ev.t5_formula directly
  - T5 integrated-mode happy path — latent Phase-13 bug now fixed
  - Attach-mode legacy-DB reads — explicitly None, detail pane will fall back
tech-stack:
  added: []
  patterns:
    - "Propagate Option<T> field through pipeline (mirror of Phase-13 t4_capability pattern)"
    - "Catalog-driven lookup HashMap built at startup, indexed by payload_id"
key-files:
  created: []
  modified:
    - src/types.rs
    - src/server/mod.rs
    - src/broker/mod.rs
    - src/monitor/mod.rs
decisions:
  - "Propagate via AppEvent field (not AppState catalog cache) — attach mode degrades gracefully to None"
  - "Fix attach-mode AppEvent field in Task 2 (not Task 3) — required for Task-2 build-green criterion (Rule 3 deviation)"
metrics:
  duration_minutes: 6
  tasks_completed: 3
  files_modified: 4
  tests_added: 3
  tests_extended: 2
  completed_date: "2026-04-24"
---

# Phase 14 Plan 01: T5Formula Propagation Foundation Summary

Wave-1 foundation plan. Propagates the Tier-5 `T5Formula` struct end-to-end from the server's `t5_callback_handler` (where the formula is already in scope via `NonceMeta.t5_formula`) through the broker pipeline into every `AppEvent` surfaced in the Monitor TUI — and fixes the latent Phase-13 bug where integrated-mode monitor startup hardcoded `NonceMeta.t5_formula = None`, causing every T5 callback in integrated mode to silently fall through at `src/server/mod.rs:160-163` (NO_CONTENT drop).

## Plan-at-a-glance

| Task | Type | Files | Commit |
|------|------|-------|--------|
| 1 | feat (tdd) | src/types.rs | `0ef8b7f` |
| 2 | feat (tdd) | src/server/mod.rs, src/broker/mod.rs, src/monitor/mod.rs | `3dc3e80` |
| 3 | fix (tdd) | src/monitor/mod.rs, src/broker/mod.rs (fmt) | `c5000b3` |

## Key Changes

### `src/types.rs`
- Added `pub t5_formula: Option<T5Formula>` as the last field of both `RawCallbackEvent` and `AppEvent`, with Phase-14 rustdoc explaining intent and fallback semantics.
- Extended `test_raw_callback_event_fields` and `test_app_event_fields` to initialize the new field as `None`.
- Added `test_app_event_t5_formula_accessible` exercising `Some(T5Formula { a: 7, b: 13, modulus: 1000 })` construction and read-back via `.unwrap()` on the three formula constants.

### `src/server/mod.rs`
- `t5_callback_handler` (line 199): sets `t5_formula: Some(*formula)` from the already-in-scope `&T5Formula` binding at line 160. `T5Formula` derives `Copy`.
- `callback_handler` (T1, line 85): sets `t5_formula: None`.
- `t4_callback_handler` (line 137): sets `t5_formula: None`.
- Server has 2 `t5_formula: None` literals + 1 `t5_formula: Some(*formula)` literal — matches Task 2 acceptance grep.

### `src/broker/mod.rs`
- `broker_task` (line 37): one-line propagation `t5_formula: raw.t5_formula` — mirrors the existing T4/T5 propagation pattern from Phase 13.
- `make_raw_event` helper (line 208): defaults `t5_formula: None`.
- `test_broker_task_propagates_t5_proof` extended: T5 literal sets `t5_formula: Some(T5Formula { a: 7, b: 13, modulus: 1000 })` and asserts the field round-trips.
- New test `test_broker_task_t5_formula_none_when_raw_is_none` — verifies the broker propagates `None` faithfully (matters for T1/T4 handlers and attach mode).
- Two additional `AppEvent { ... }` literals in `test_db_writer_task_persists_event` and `test_stdout_logger_task_formats_log_line` extended with `t5_formula: None,` to keep the build green.

### `src/monitor/mod.rs`
- **Attach-mode** AppEvent (`run_loop_attach`, line 830): sets `t5_formula: None` — documented as legacy-DB safe fallback.
- **Integrated-mode nonce_map** (lines 908-930, the bug fix): replaces the hard-coded `t5_formula: None` block with the correct pattern mirrored from `src/server/mod.rs:249-274`:
  1. `crate::catalog::load_catalog()?` loads all payloads at startup.
  2. A `HashMap<payload_id, T5Formula>` is built via `filter_map`.
  3. Per-mapping lookup: `if m.tier == Tier::Tier5 { map.get(&m.payload_id).copied() } else { None }`.
- Imports extended: `crate::types::{T5Formula, Tier}` added to the existing types import line.
- `make_test_event` test helper: extended with `t5_formula: None,`.
- New regression test `test_integrated_mode_nonce_map_loads_t5_formula`: exercises the catalog-lookup round-trip in isolation. Asserts (a) at least one tier-5 payload exists in the catalog with a formula, and (b) each tier-5 payload's formula round-trips through the HashMap correctly.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Attach-mode `AppEvent` field addition moved from Task 3 to Task 2**

- **Found during:** Task 2 `cargo build --lib` verification
- **Issue:** The plan's Task 2 acceptance criterion requires `cargo build --lib` to exit 0. However, `src/monitor/mod.rs:811` (attach-mode production code, NOT test code) has an `AppEvent { ... }` literal that must also include the new `t5_formula` field or the whole crate fails to compile. The plan bundles that edit into Task 3 (Edit 3a).
- **Fix:** Applied the single-line `t5_formula: None,` addition to the attach-mode AppEvent literal during Task 2 rather than waiting for Task 3. Plan's full Edit 3a commentary (legacy-db fallback rustdoc) was applied verbatim. No downstream impact: Task 3 simply had less work to do.
- **Files modified:** src/monitor/mod.rs (line 830)
- **Commit:** `3dc3e80` (Task 2)

**2. [Rule 2 - Correctness] Extended two additional broker-test AppEvent literals not explicitly listed in the plan**

- **Found during:** Task 2 `cargo build --lib` verification
- **Issue:** Plan's Task 2 action text covers `make_raw_event`, `test_broker_task_propagates_t5_proof`, and the new `test_broker_task_t5_formula_none_when_raw_is_none` test. It did not explicitly list `test_db_writer_task_persists_event` (line 391) or `test_stdout_logger_task_formats_log_line` (line 440), both of which construct `AppEvent { ... }` literals directly.
- **Fix:** Added `t5_formula: None,` to both literals. Plan's Edit 2d already anticipated this with "also update any other `RawCallbackEvent { ... }` literals or `AppEvent { ... }` literals elsewhere in `src/broker/mod.rs` tests if rustc reports missing-field errors".
- **Files modified:** src/broker/mod.rs (lines 409, 459)
- **Commit:** `3dc3e80` (Task 2)

**3. [Rule 2 - Correctness] Extended `make_test_event` helper in monitor tests**

- **Found during:** Task 2 `cargo build --lib` verification
- **Issue:** `src/monitor/mod.rs:1042` helper `make_test_event` constructs an `AppEvent { ... }` literal. Plan's Task 3 doesn't explicitly cover this but it has to be extended for the library tests to compile after Task 1 introduces the new struct field.
- **Fix:** Added `t5_formula: None,` to the helper.
- **Files modified:** src/monitor/mod.rs (line 1042)
- **Commit:** `3dc3e80` (Task 2)

**4. [fmt auto-apply] `cargo fmt` reformatted the new nonce_map block and new broker test**

- **Found during:** Task 3 `cargo fmt --all -- --check` verification
- **Issue:** `cargo fmt --check` reported the new HashMap type annotation as wanting a different indentation pattern. Also reformatted a multi-line `tokio::time::timeout(...)` call in the new `test_broker_task_t5_formula_none_when_raw_is_none` test.
- **Fix:** Ran `cargo fmt --all` once and recommitted. No semantic change.
- **Files modified:** src/monitor/mod.rs, src/broker/mod.rs
- **Commit:** `c5000b3` (Task 3)

## Threat Model Status

All four items in the plan's `<threat_model>` STRIDE register addressed:

- **T-14-01-01 (DoS — integrated-mode T5 silent drop):** mitigated by Task 3's catalog-loading fix. Regression-tested via `test_integrated_mode_nonce_map_loads_t5_formula`.
- **T-14-01-02 (Tampering — attach-mode formula access):** mitigated by Task 2's `t5_formula: None` on attach-mode AppEvent. Plan 02 pattern-matches on `Option<T5Formula>`.
- **T-14-01-03 (Info Disclosure — formula leaks):** accepted per plan; formula already public in catalog and JSON-LD.
- **T-14-01-04 (Tampering — partial catalog):** accepted per plan; `|| None` fallback matches existing server behavior.

## Verification Results

- `cargo build --lib`: green
- `cargo fmt --all -- --check`: green
- `cargo clippy --all -- -D warnings`: green (no warnings introduced)
- `cargo test --all`: **184 tests passed, 0 failed**
  - Unit tests: 136 (lib) + others
  - Integration tests: 14 in `test_serve.rs` (including existing T4/T5 happy-path tests), 9 in `test_report.rs`, 3 in `test_monitor.rs`, 4 in `test_test_agent.rs`, etc.
- Phase-level grep gates:
  - `grep -n "t5_formula: Some(\*formula)" src/server/mod.rs` -> 1 match (T5 handler)
  - `grep -c "t5_formula: None" src/server/mod.rs` -> 2 (T1 + T4)
  - `grep -n "t5_formula: raw.t5_formula" src/broker/mod.rs` -> 1 match (broker_task)
  - `grep -rn "t5_formulas_by_payload_id" src/` -> 2 call sites (src/server/mod.rs, src/monitor/mod.rs) as required by the plan's `<verification>` block

## TDD Gate Compliance

All three tasks had `tdd="true"`. Strict RED/GREEN separation was demonstrated for Task 1 via the `test_app_event_t5_formula_accessible` test — run first (RED, compile-fail on missing field), then the struct change applied (GREEN). Tasks 2 and 3 combine the RED state (existing build failures from Task 1) with the GREEN implementation; tests were added or extended alongside the implementation within each task's commit. All commits follow conventional-commit-style `feat(14-01)` / `fix(14-01)` prefixes per plan spec.

## Follow-up for Plan 14-02

Plan 02 (the TUI detail pane) can now consume `ev.t5_formula` directly:

```rust
let formula_line = match ev.t5_formula {
    Some(f) => format!("formula=(seed+{})*{} % {}", f.a, f.b, f.modulus),
    None => "formula=(unavailable — legacy db)".to_string(),
};
```

Attach-mode will hit the `None` branch (explicitly wired); integrated mode will hit `Some(_)` (via the Task-3 catalog-loading fix). No further wiring required from the types / broker / server side.

## Self-Check: PASSED

- Commit `0ef8b7f` (Task 1): FOUND in `git log --all`
- Commit `3dc3e80` (Task 2): FOUND in `git log --all`
- Commit `c5000b3` (Task 3): FOUND in `git log --all`
- `src/types.rs` modified: FOUND (t5_formula field present on both structs)
- `src/server/mod.rs` modified: FOUND (Some(*formula) at line 199; None at 85, 137)
- `src/broker/mod.rs` modified: FOUND (t5_formula: raw.t5_formula at line 37)
- `src/monitor/mod.rs` modified: FOUND (t5_formulas_by_payload_id + load_catalog at line 912-913)
- `.planning/phases/14-tiers-4-5-surfacing-monitor-tui-report/14-01-SUMMARY.md` written: this file
