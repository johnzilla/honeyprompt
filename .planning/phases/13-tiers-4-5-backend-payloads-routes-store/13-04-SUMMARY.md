---
phase: 13-tiers-4-5-backend-payloads-routes-store
plan: 04
subsystem: api
tags: [axum, http-handler, base64, sanitization, proof-verification, url-safe-base64, integration-tests]

requires:
  - phase: 13-01
    provides: Tier::Tier4/5 enum variants, T5Formula struct, derive_seed and is_valid_nonce helpers
  - phase: 13-02
    provides: migrated events schema with t4_capability/t5_proof/t5_proof_valid columns; insert_callback_event signature accepting three trailing Option<_> params; broker db_writer_task placeholder None values
  - phase: 13-03
    provides: generator emits seed JSON-LD with verification_seed == derive_seed(nonce); server/generator seed contract established
provides:
  - "/cb/v4/{nonce}/{b64_payload} and /cb/v5/{nonce}/{proof} Axum routes"
  - "t4_callback_handler + t5_callback_handler with always-204 discipline (D-13-15)"
  - "compute_expected_proof with u64 promotion + wrapping arithmetic (T-13-02 guard)"
  - "decode_t4_payload + is_valid_t4_payload hand-rolled sanitizer (D-13-09, no regex dep)"
  - "NonceMeta.t5_formula Option<T5Formula> populated at serve() startup (Q3 resolution)"
  - "RawCallbackEvent + AppEvent propagate T4/T5 fields end-to-end through broker"
  - "db_writer_task now passes real Option<_> values (Plan 02 placeholders replaced)"
  - "/cb/v1/{nonce} regression test proving byte-identical 204 + empty body (D-13-18)"
affects: ["14-monitor-tui", "14-markdown-report", "14-test-agent-scorecard"]

tech-stack:
  added: []
  patterns: ["URL-safe no-pad base64 in HTTP path segments (axum Path tuple extraction)", "compute-in-test to avoid testing same formula twice (T5 proof tests)", "fixture-helper for handler tests: construct NonceMeta directly, no generate pipeline"]

key-files:
  created: []
  modified:
    - src/server/mod.rs
    - src/broker/mod.rs
    - src/types.rs
    - src/monitor/mod.rs
    - src/test_agent/mod.rs
    - tests/test_serve.rs
    - tests/serve_domain.rs

key-decisions:
  - "Handlers are placed in src/server/mod.rs alongside callback_handler — no new module needed"
  - "Catalog load happens once at serve() startup (RESEARCH Q3); t5_formulas_by_payload_id HashMap is joined by payload_id to populate NonceMeta.t5_formula for tier-5 entries"
  - "Non-URL-safe base64 rejection test uses '+' (not '/') — '/' would URL-route to a different handler and return 404 instead of the expected 204"
  - "Fixture helper build_t4_t5_state constructs NonceMeta entries directly rather than going through generator — faster, deterministic, independent of catalog changes"
  - "Structural-ordering invariant proven by grep, not by code path: catalog::load_catalog() on line 252 < Arc::new(AppState) on line 305 in src/server/mod.rs (WARNING 4 fix)"

patterns-established:
  - "Hand-rolled sanitizer over regex: 10-line byte scan `^[a-z0-9_,.\\-]{1,256}$` after lowercase + whitespace strip — auditable for the safety model"
  - "u64 promotion for u32-overflow-prone arithmetic: cast to u64, use wrapping_add / wrapping_mul, reduce % modulus, cast back"
  - "Always-204 audit trail: threat register T-13-01 lists every failure branch; regression test covers all of them"

requirements-completed: [SERVER-01, SERVER-02, SERVER-03, SERVER-04]

duration: ~70min across 3 agent invocations (sandbox interruptions handled by inline orchestrator completion of Tasks 2-4)
completed: 2026-04-24
---

# Phase 13-04: Server integration + /cb/v1/ byte-identity Summary

**Adds `/cb/v4/{nonce}/{b64_payload}` and `/cb/v5/{nonce}/{proof}` routes with always-204 discipline, u64-promoted T5 proof verification, D-13-09 sanitizer, and a byte-identity regression test proving D-13-18 for `/cb/v1/`. Completes phase 13.**

## Performance

- **Duration:** ~70 min (three executor invocations + orchestrator inline completion after sandbox blocks on Tasks 2-4)
- **Tasks:** 4/4 complete
- **Files modified:** 7 (src/server/mod.rs, src/broker/mod.rs, src/types.rs, src/monitor/mod.rs, src/test_agent/mod.rs, tests/test_serve.rs, tests/serve_domain.rs)
- **Test results:** `cargo test` — 181 passed, 0 failed
- **Lints:** `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt -- --check` clean
- **New test count (this plan):** 10 unit + 6 integration = 16 new tests

## Accomplishments

- **Task 1 (broker/types plumbing):** Extended `RawCallbackEvent` and `AppEvent` with `t4_capability: Option<String>`, `t5_proof: Option<String>`, `t5_proof_valid: Option<bool>`. `broker::broker_task` propagates these fields verbatim from raw → app. `broker::db_writer_task` passes the real values to `insert_callback_event` (replacing Plan 02's `None, None, None` placeholders).
- **Task 2 (pure helpers):** Added three self-contained helpers in `src/server/mod.rs` with no AppState dependency:
  - `compute_expected_proof(seed, a, b, mod)` → u64 arithmetic, wrapping ops, cast to u32 at the end
  - `decode_t4_payload(b64)` → URL-safe no-pad decode, UTF-8 check, lowercase + whitespace strip, sanitizer call
  - `is_valid_t4_payload(&str)` → hand-rolled `^[a-z0-9_,.\-]{1,256}$` byte scan
  Plus 10 unit tests including the parenthesization trap guard (`test_compute_expected_proof_seed_zero_nontrivial_a_b`, seed=1 → 731) and u64 overflow guard (`test_compute_expected_proof_max_seed_no_overflow`, u32::MAX inputs).
- **Task 3 (server integration):** `NonceMeta` gains `Option<T5Formula>` field (4 construction sites updated: src/server, src/monitor, src/test_agent, tests/test_serve, tests/serve_domain — all set `t5_formula: None` except the T5 fixture). `serve()` loads catalog once at startup via `catalog::load_catalog()`, builds a `HashMap<String, T5Formula>` keyed by payload_id BEFORE the nonce_map construction loop (structural ordering invariant — WARNING 4). `t4_callback_handler` and `t5_callback_handler` added; both return `StatusCode::NO_CONTENT` on every branch. `build_router` wires both new routes alongside unchanged `/cb/v1/`.
- **Task 4 (integration tests):** 6 new `#[tokio::test]` entries in `tests/test_serve.rs` using the new `build_t4_t5_state` fixture helper. Zero modifications to existing test function bodies. `test_cb_v1_byte_identical_response` explicitly asserts 204 status + empty body — closing RESEARCH assumption A4.

## Task Commits

1. **Task 1: Broker + types field plumbing** — `01cbd86` (feat) — RawCallbackEvent, AppEvent, broker_task, db_writer_task all carry T4/T5 fields.
2. **Task 2: Pure helpers + 10 unit tests** — `c674f6e` (feat) — compute_expected_proof (u64 + wrapping), decode_t4_payload, is_valid_t4_payload; tests include u32::MAX overflow guard and parenthesization guard (seed=1 → 731).
3. **Task 3: Server integration** — `0d6955f` (feat) — NonceMeta extension across 4 construction sites, T4/T5 async handlers, build_router extension, serve() catalog preload (structural ordering invariant holds: L252 < L305).
4. **Task 4: Integration tests + /cb/v1/ regression** — `32df415` (test) — 6 new `#[tokio::test]` entries, fixture `build_t4_t5_state`, `test_cb_v1_byte_identical_response` asserts 204 + empty body.

## Files Created/Modified

- `src/server/mod.rs` — NonceMeta `t5_formula` field, 3 new helper fns + 10 unit tests, 2 new async handlers, 2 new route registrations, serve() catalog preload
- `src/broker/mod.rs` — AppEvent propagation of T4/T5 fields, db_writer_task passes real values (replacing Plan 02's None placeholders)
- `src/types.rs` — RawCallbackEvent and AppEvent extended with 3 new Option fields each
- `src/monitor/mod.rs` — NonceMeta construction site updated (`t5_formula: None`)
- `src/test_agent/mod.rs` — NonceMeta construction site updated (`t5_formula: None`)
- `tests/test_serve.rs` — 6 new integration tests, fixture `build_t4_t5_state`, existing NonceMeta site updated
- `tests/serve_domain.rs` — NonceMeta construction site updated (`t5_formula: None`)

## Decisions Made

- `callback_handler` (the `/cb/v1/` handler) body was **left completely unchanged**. No inline-check refactor was performed — the inline check at lines 48-52 remains exactly as in v4.0. `test_cb_v1_byte_identical_response` confirms 204 + empty body; all 3 existing `test_callback_*` tests still pass without modification.
- Formula test vector used: **seed = 0xbbbbbbbb (3_149_642_683), formula (a=42, b=17, modulus=1000)**. Expected proof computed in the test via `(((seed.wrapping_add(42)).wrapping_mul(17)) % 1000) as u32`. This ensures the test is not testing the same bug twice (the test's arithmetic matches but is independent of the implementation).
- Parenthesization regression test `test_compute_expected_proof_seed_zero_nontrivial_a_b` **passes with expected value 731** — per the checker-corrected test vector. seed=0 would NOT catch the parenthesization bug (both `(0+a)*b` and `0+(a*b)` produce the same value), so seed=1 was used: `(1+42)*17 % 1000 = 43*17 % 1000 = 731`; a wrong `seed + (a*b)` implementation would produce `1 + (42*17) % 1000 = 715`.
- The T4 malformed test case originally proposed `"not+url/safe"` fails because the `/` creates a second path segment that doesn't match the route — axum returns 404, not 204. Replaced with `"abcd+efgh"` (contains `+` which is standard-b64-but-not-URL-safe), which is a single path segment that reaches the handler and is correctly rejected by `URL_SAFE_NO_PAD.decode()` → silent 204.

## Deviations from Plan

**None in substance — plan executed exactly as written.**

Operational deviations handled by the orchestrator:
- **Sandbox permission failures during Tasks 2-4:** the executor subagent completed the implementation for each task but was blocked from running `git add` / `git commit` by the runtime sandbox. The orchestrator inspected the working-tree state, verified all tests passed (10 helper tests for Task 2, 14 integration tests for Task 4), and completed the commits directly. No work was lost and no divergence from the plan text occurred.
- **First Task 2 attempt stalled** (stream watchdog) with only an unused `base64` import added to `src/server/mod.rs`. The orchestrator discarded the stale diff via `git checkout` and re-dispatched a fresh agent that completed the real work.
- **First Task 1 attempt (worktree mode) hit a wrong-base worktree** — the `EnterWorktree` tool created a branch from a stale point (commit `55f16c0`) rather than `main@60f74b1`. The executor correctly refused to bypass its base-commit guard. Re-dispatched in sequential mode on the main working tree — no repeat issue.

## Issues Encountered

- **Sandbox regression:** a persistent mid-session restriction on `git add`/`git commit` for subagents started mid-phase. Orchestrator-side commit completion is a workaround; the underlying issue is external to this plan.
- **Stream watchdog stall (600s idle):** one executor invocation stalled on a long Edit/Read sequence. Re-dispatch with explicit "stay well below 600s" instruction worked.
- No code-level issues. All correctness-critical tests pass on the first real run.

## User Setup Required

None — no external service configuration.

## Phase 13 Completion — Requirements Traceability

| REQ-ID        | Plan  | Automated Verification                                                  | Status |
|---------------|-------|-------------------------------------------------------------------------|--------|
| PAYLOAD-01    | 13-01 | `cargo test --lib catalog::tests::test_tier4_catalog`                   | PASS   |
| PAYLOAD-02    | 13-01 | `cargo test --lib catalog::tests::test_tier4_diverse_phrasing`          | PASS   |
| PAYLOAD-03    | 13-01 | `cargo test --lib catalog::tests::test_tier5_catalog`                   | PASS   |
| PAYLOAD-04    | 13-03 | `cargo test --lib generator::tests::test_t5_seed_json_ld_emission`      | PASS   |
| PAYLOAD-05    | 13-01 | `cargo test --lib catalog::tests::test_no_duplicate_locations`          | PASS   |
| SERVER-01     | 13-04 | `cargo test --test test_serve test_t4_callback_happy_path`              | PASS   |
| SERVER-02     | 13-04 | `cargo test --test test_serve test_t5_callback_valid_proof`             | PASS   |
| SERVER-03     | 13-04 | `cargo test --test test_serve test_cb_v1_byte_identical_response`       | PASS   |
| SERVER-04     | 13-04 | `cargo test --test test_serve test_t{4,5}_malformed_returns_204`        | PASS   |
| STORE-01      | 13-02 | `cargo test --lib store::tests::test_schema_t4_columns`                 | PASS   |
| STORE-02      | 13-02 | `cargo test --lib store::tests::test_schema_t5_columns`                 | PASS   |
| STORE-03      | 13-02 | `cargo test --test test_migration test_v4_db_opens_unchanged`           | PASS   |
| STORE-04      | 13-02 | `cargo test --lib store::tests::test_insert_callback_event_replay_*`    | PASS   |

**All 13 REQ-IDs verified green.**

## Next Phase Readiness

- Phase 13 backend is complete. All T4/T5 infrastructure is ready for Phase 14 to surface it in the Monitor TUI, Markdown report, and test-agent scorecard.
- The 3 new columns on the events table (`t4_capability`, `t5_proof`, `t5_proof_valid`) are populated by the handlers end-to-end.
- **Remaining residual risk (acknowledged, not mitigated in code):** axum's `Path` extractor returns 400 (not 204) for invalid-UTF-8 percent-encoded URL segments — RESEARCH Risk 1. Practical exposure is near-zero (URL-safe base64 is ASCII, legitimate clients never produce invalid UTF-8 percent-encodes), and the leak reveals only that the endpoint exists (already inferable from path structure). Documented for future phase owners.

---
*Phase: 13-tiers-4-5-backend-payloads-routes-store*
*Completed: 2026-04-24*
