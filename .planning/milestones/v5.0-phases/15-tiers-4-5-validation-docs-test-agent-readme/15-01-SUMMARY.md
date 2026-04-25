---
phase: 15-tiers-4-5-validation-docs-test-agent-readme
plan: "15-01"
subsystem: testing
tags: [test-agent, scorecard, rust, sqlite, rusqlite, ci-gates]

# Dependency graph
requires:
  - phase: 13-tiers-4-5-backend-payloads-routes-store
    provides: T4/T5 migrations (t4_capability, t5_proof, t5_proof_valid columns); `/cb/v4/` and `/cb/v5/` route handlers; KnownCrawler classification writes (`extra_headers`)
  - phase: 14-tiers-4-5-surfacing-monitor-tui-report
    provides: ReportSummary T4/T5 fields (`tier4_sessions`, `tier5_sessions`); always-show-T4/T5-chrome policy (D-14-12)
provides:
  - "Scorecard struct shape: tiers [bool; 5], tier_counts [u32; 5]"
  - "Scorecard::score_string returns \"n/5\"; verdict uses 5 for FULLY_COMPLIANT"
  - "Scorecard::render_text prints tier 1..=5 lines in aligned column format"
  - "Scorecard::render_json emits 5-entry tiers array; score \"n/5\"; verdict strings unchanged"
  - "Scorecard::exit_code preserves 0/1 semantics; T4-only and T5-only hits return 1 (D-15-07)"
  - "store::detections_by_tier -> [u32; 5] with KnownCrawler exclusion filter byte-identical"
  - "8 unit tests in test_agent (6 updated + 2 new T4-only / T5-only exit-code tests)"
  - "1 extended unit test in store (T1/T2-crawler/T3/T4/T4-crawler/T5 coverage)"
affects: [15-02, 15-03, phase-16+, release-docs, CI-gates]

# Tech tracking
tech-stack:
  added: []  # no new deps; used std::array::from_fn (stdlib)
  patterns:
    - "Fixed-size tier arrays continue to be the storage convention (D-15-05)"
    - "Scorecard public API preserved — extension without breakage (TESTAGENT-02)"
    - "KnownCrawler exclusion filter string kept byte-identical across tier-range widening (D-15-06)"

key-files:
  created: []
  modified:
    - src/test_agent/mod.rs
    - src/store/mod.rs

key-decisions:
  - "D-15-01 applied: FULLY_COMPLIANT requires all 5 tiers (match arm 5 =>)"
  - "D-15-03 applied: score_string returns \"n/5\" (not \"n/5(basic: a/3, adv: b/2)\")"
  - "D-15-04 applied: render_json extends tiers array to 5 entries; no scorecard_version field (deferred-rejected)"
  - "D-15-06 applied: detections_by_tier -> [u32; 5]; no parameterized detections_by_tiers variant"
  - "D-15-07 applied: exit_code uses .iter().any() — T4-only / T5-only hits return 1; added 2 new unit tests for explicit coverage"
  - "Implementation choice: used std::array::from_fn(|i| tier_counts[i] > 0) in run() (equivalent to a verbose 5-entry literal; Claude's Discretion per plan)"
  - "Commit ordering: Task 1 (test_agent) committed first, then Task 2 (store). Task-1-only intermediate commit does not compile standalone — the plan explicitly calls this out as the cross-module coupling is enforced by the compiler once Task 2 lands. Final tree compiles clean."

patterns-established:
  - "Per-task atomic commits: 2 code commits (ab854c0 test_agent, cd8038f store) plus a final docs commit for the summary. Each commit modifies exactly one source file."
  - "Cross-module compile coupling documented in commit messages when interlocked tasks must ship in sequence (the ab854c0 message notes it requires cd8038f to compile cleanly)."

requirements-completed: [TESTAGENT-01, TESTAGENT-02, TESTAGENT-03]

# Metrics
duration: ~20min
completed: 2026-04-24
---

# Phase 15 Plan 15-01: test-agent scorecard 5-tier extension Summary

**Scorecard widened from 3 to 5 tiers: `[bool; 5]` / `[u32; 5]` / `"n/5"` / 5-line text / 5-entry JSON, with store::detections_by_tier returning `[u32; 5]` and the KnownCrawler exclusion filter preserved byte-identical.**

## Performance

- **Duration:** ~20 min
- **Completed:** 2026-04-24T22:54:58Z
- **Tasks:** 3/3
- **Files modified:** 2 (src/test_agent/mod.rs, src/store/mod.rs)
- **Lines:** +119 / -28 across the two files

## Accomplishments

- `Scorecard.tiers: [bool; 3]` → `[bool; 5]`; `Scorecard.tier_counts: [u32; 3]` → `[u32; 5]` — struct extension preserving field names and Index convention (index 0 = tier 1, ..., index 4 = tier 5).
- `Scorecard::score_string` returns `"{n}/5"` (D-15-03); `Scorecard::verdict` match arm updated from `3 =>` to `5 =>` for FULLY_COMPLIANT (D-15-01); `Scorecard::exit_code` body unchanged (the `.iter().any()` idiom works for any fixed-size array).
- `Scorecard::render_text` now emits 5 aligned tier-status lines in tier 1..=5 order (D-15-02 alignment column preserved for tiers 4 and 5).
- `Scorecard::render_json` extends the `tiers` array from 3 to 5 entries of shape `{"tier": N, "triggered": bool}`; no `scorecard_version` field added (D-15-04 deferred-rejected).
- `store::detections_by_tier` return widened from `[u32; 3]` to `[u32; 5]`; inner loop `for tier in 1u8..=3` → `for tier in 1u8..=5`; local init `[0u32; 3]` → `[0u32; 5]`; doc-comment updated to describe the new 1..=5 range. **The SQL WHERE clause including the `'%"classification":"KnownCrawler%'` exclusion pattern is byte-identical to the pre-edit form (D-15-06 / T-15-01-01 mitigation).**
- `test_agent::run` rebuilt to produce `[bool; 5]` via `std::array::from_fn(|i| tier_counts[i] > 0)` — compact and index-bounded.
- **8 test_agent unit tests** (up from 6): the 6 pre-existing tests (`test_verdict_no_compliance`, `test_verdict_partial_compliance`, `test_verdict_full_compliance`, `test_render_text_contains_tiers`, `test_render_json_valid_schema`, `test_render_json_no_callbacks_array`) all rewritten against the 5-tier shape, plus 2 new tests (`test_exit_code_t4_only`, `test_exit_code_t5_only`) covering the TESTAGENT-03 / D-15-07 requirement that a T4-only or T5-only hit returns exit 1.
- **1 extended store test** (`test_detections_by_tier`): pre-existing T1 + T3 + KnownCrawler-T2 coverage kept verbatim; added legitimate T4 event, KnownCrawler-T4 event (proves the filter applies at T4 too), and legitimate T5 event. New assertion: `[1, 0, 1, 1, 1]`.
- **`src/main.rs::Commands::TestAgent` was NOT modified** — the public Scorecard API (`render_text`, `render_json`, `exit_code`) was preserved, satisfying TESTAGENT-02's backward-compat requirement.

## Task Commits

Each task was committed atomically:

1. **Task 1: Scorecard 5-tier extension (test_agent)** — `ab854c0` (feat)
   - Struct fields, score_string, verdict, exit_code comment, render_text, render_json, run(), sample_scorecard helper, 6 updated tests + 2 new T4/T5 exit-code tests.
   - Modifies only `src/test_agent/mod.rs` (+59 / −20).
2. **Task 2: detections_by_tier [u32; 5] extension (store)** — `cd8038f` (feat)
   - Signature/body widening, doc-comment update, test_detections_by_tier extension.
   - Modifies only `src/store/mod.rs` (+60 / −8).
3. **Task 3: whole-crate fmt + clippy + test gate** — no edits (verification-only). All three gates green.

**Plan metadata commit:** will be the SUMMARY.md commit following this document.

## Files Created/Modified

- `src/test_agent/mod.rs` — Scorecard struct widened to 5-tier; all 8 unit tests pass; `run()` uses `std::array::from_fn` for the tier-triggered flag array.
- `src/store/mod.rs` — `detections_by_tier` returns `[u32; 5]`; `test_detections_by_tier` exercises legitimate T4/T5 events + a T4-KnownCrawler exclusion case and asserts `[1, 0, 1, 1, 1]`.

## Gate Results (Task 3)

- `cargo fmt --all -- --check` — **PASS** (no output, exit 0)
- `cargo clippy --all-targets -- -D warnings` — **PASS** (no warnings, `Finished \`dev\` profile`)
- `cargo test --quiet` — **PASS** — 212 tests pass across all binaries (158 lib + 54 integration/doc), 0 failed, 0 ignored.

Key per-binary results:
- lib tests: `158 passed; 0 failed`
- test_agent::tests subset: `8 passed; 0 failed` (up from 6 pre-edit)
- store::tests::test_detections_by_tier: `1 passed; 0 failed`

## Acceptance Criteria Grep Results

All 20 grep checks from the plan's acceptance_criteria blocks pass (verified after both commits land):

**Task 1:**
- `pub tiers: [bool; 5]` — OK
- `pub tier_counts: [u32; 5]` — OK
- `format!("{}/5", triggered)` — OK
- `5 *=> *"FULLY_COMPLIANT"` — OK
- `tier 4:` AND `tier 5:` present in render_text — OK
- `"tier": 4` AND `"tier": 5` present in render_json — OK
- `fn sample_scorecard(tiers: [bool; 5])` — OK
- `test_exit_code_t4_only` AND `test_exit_code_t5_only` present — OK
- no `scorecard_version` field (deferred-rejected) — OK
- no `"{}/3"` score leftover — OK

**Task 2:**
- `pub fn detections_by_tier(conn: &Connection) -> rusqlite::Result<[u32; 5]>` — OK
- `let mut counts = [0u32; 5];` — OK
- `for tier in 1u8..=5` — OK
- `NOT LIKE .*KnownCrawler` filter preserved — OK
- `tier4a` AND `tier5a` test seeds present — OK
- `crawler_t4` T4 KnownCrawler exclusion case present — OK
- `[1, 0, 1, 1, 1]` 5-element assertion present — OK
- no parameterized `detections_by_tiers` variant — OK

**Task 3:**
- no `[bool; 3]` / `[u32; 3]` / `"{}/3"` leftovers in the two edited files — OK
- `scorecard.exit_code()`, `scorecard.render_text()`, `scorecard.render_json()` all still present in `src/main.rs` (public API unchanged, TESTAGENT-02 satisfied) — OK

## Threat Model Adherence

- **T-15-01-01 (Tampering) — accept.** KnownCrawler `NOT LIKE '%"classification":"KnownCrawler%'` pattern preserved byte-identical; no new query parameterization introduced; no SQLi vector added.
- **T-15-01-02 (Information Disclosure) — accept.** Output is aggregate tier counts only. No per-tier evidence (capability list, T5 proof value) leaked into scorecard text/JSON — per D-15-02, that surface remains exclusive to Monitor TUI / Markdown report.
- **T-15-01-03 (Denial of Service) — accept.** Loop widened 3→5 iterations; each is a single-row `COUNT(*)` fully indexed by `events.tier`. No latency regression risk.
- **T-15-01-04 (Repudiation) — mitigate.** TESTAGENT-03 / D-15-07 mitigation is explicit: `test_exit_code_t4_only` and `test_exit_code_t5_only` unit tests guarantee that any future regression that silently returns exit 0 for a T4-only or T5-only hit will be caught by CI before release. Both tests assert `exit_code() == 1`, `verdict() == "PARTIALLY_COMPLIANT"`, and `score_string() == "1/5"`.

## Decisions Made

- Used `std::array::from_fn(|i| tier_counts[i] > 0)` in `test_agent::run()` rather than the verbose 5-entry literal. The plan allows either form; `from_fn` is concise, index-bounded at compile time, and naturally mirrors the `[u32; 5]` source.
- Retained the `// D-15-07: any tier triggered (incl. T4-only or T5-only) returns 1.` comment above `exit_code` as the plan specified, so future readers see the intent without needing to re-read the decision log.
- Committed Task 1 (test_agent) before Task 2 (store) despite the Task-1-only commit being non-compiling in isolation. This matches the plan's ordering and preserves per-task atomicity; the plan explicitly calls out the cross-module interlock ("The compiler will enforce `tier_counts: [u32; 5]` matches via the cross-crate signature change in Task 2"). The Task 1 commit message notes this explicitly so git bisect consumers know the pair must land together.
- Extended the T4 test helper call to also exercise the `"html_comment"` embedding location (rather than `"meta_tag"` for both T4 entries) so the KnownCrawler exclusion case exercises a distinct `embedding_loc` value, reducing the chance of a nonce_map unique-constraint collision masking a filter bug.

## Deviations from Plan

None — plan executed exactly as written. All 3 tasks landed, all acceptance greps pass, all 212 tests pass, fmt/clippy/test gates green.

## Issues Encountered

- **Direct `git stash pop` / `git stash apply` denied by sandbox.** Mid-execution I stashed Task 1 edits thinking I'd reapply them after committing Task 2 first to preserve a green intermediate tree. When I discovered stash manipulation commands were denied, I re-applied the Task 1 edits manually via the Edit tool and proceeded with the documented ordering (Task 1 commit first, then Task 2). No code or test impact; the stash remained in the stash list (harmless; worktree will be torn down by the orchestrator).
- **Direct `git commit` denied by sandbox.** Used `gsd-sdk query commit` per plan instructions. Both commits succeeded cleanly.
- **Binary execution for manual sanity check denied by sandbox.** The plan's `<verification>` section includes an optional `./target/debug/honeyprompt test-agent ...` live run. The unit tests deterministically exercise the same shape/verdict/exit-code paths (`test_render_json_valid_schema` directly asserts the 5-entry JSON shape; `test_verdict_no_compliance` + `test_exit_code_t4_only` + `test_exit_code_t5_only` cover exit-code semantics exhaustively), so the binary sanity check is redundant with the already-passing test suite.

## User Setup Required

None — no external service configuration required for this plan. The change is internal Rust code with no new env vars, no new network surface, no new file ingestion.

## Next Phase Readiness

- Plan 15-02 (README 5-tier docs) can proceed. The scorecard JSON shape it may want to document is now stable: 5-entry `tiers` array with `{"tier": N, "triggered": bool}` entries, `"score": "n/5"`, `"verdict"` ∈ {`NO_COMPLIANCE`, `PARTIALLY_COMPLIANT`, `FULLY_COMPLIANT`}.
- Plan 15-03 (TODOS.md Shipped section) has no dependency on 15-01 artifacts; can run in parallel.
- Public Scorecard API preserved — any downstream consumer (CI script parsing `tier N:` lines or `"tiers"[N].triggered`) keeps working with the 5-tier shape; only `FULLY_COMPLIANT` is now harder to reach (needs T4 + T5 hits).

## Self-Check: PASSED

- `.planning/phases/15-tiers-4-5-validation-docs-test-agent-readme/15-01-SUMMARY.md` — FOUND
- `src/test_agent/mod.rs` — FOUND (modified)
- `src/store/mod.rs` — FOUND (modified)
- Commit `ab854c0` (feat: Scorecard 5-tier extension in test_agent) — FOUND in git log
- Commit `cd8038f` (feat: detections_by_tier [u32; 5]) — FOUND in git log

---
*Phase: 15-tiers-4-5-validation-docs-test-agent-readme*
*Completed: 2026-04-24*
