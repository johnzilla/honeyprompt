---
phase: 15-tiers-4-5-validation-docs-test-agent-readme
plan: "15-02"
subsystem: docs
tags: [readme, proof-levels, ethics, project-status, tier-4, tier-5]

# Dependency graph
requires:
  - phase: 13-tiers-4-5-backend-payloads-routes-store
    provides: shipped tier5.toml::t5-semantic-prose constants (42, 17, 1000) for README T5 worked example
  - phase: 14-tiers-4-5-surfacing-monitor-tui-report
    provides: T4/T5 surfacing patterns referenced by Proof Levels description prose
provides:
  - README §Proof Levels 5-bullet structure with inline italic parenthetical examples for each tier
  - README §Ethics and Safety T4/T5 no-secrets explicit callouts
  - README §Project Status Phase 15 row (In Progress)
affects: [15-03-todos-shipped, phase-15-completion-docs-sync]

# Tech tracking
tech-stack:
  added: []
  patterns: [inline italic parenthetical callback examples with literal {nonce} placeholder]

key-files:
  created: []
  modified:
    - README.md

key-decisions:
  - "T5 worked example uses t5-semantic-prose constants (42, 17, 1000) sourced from shipped catalog — replacing CONTEXT.md draft constants (42, 7, 1000) per D-15-09 planner-verify directive"
  - "Phase 15 Project Status row uses 'In Progress' status (phase-completion sync will flip to Complete)"
  - "Two new Ethics bullets placed at bottom of existing 5-bullet list per D-15-11 recommended placement"

patterns-established:
  - "Inline italic parenthetical example style `*(e.g., ...)*` for tier documentation bullets"
  - "T5 worked examples in README must reference real shipped catalog template constants, not illustrative values"

requirements-completed: [DOCS-01, DOCS-02, DOCS-03]

# Metrics
duration: 2min
completed: 2026-04-24
---

# Phase 15 Plan 02: README 5-tier documentation Summary

**README Proof Levels, Ethics, and Project Status updated with 5-tier inline examples, T4/T5 no-secrets callouts, and a Phase 15 row — documenting the v5.0 proof model with a T5 worked formula sourced from the shipped `t5-semantic-prose` catalog template.**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-04-24T22:49:30Z
- **Completed:** 2026-04-24T22:51:22Z
- **Tasks:** 3
- **Files modified:** 1 (README.md: +8 / -5)

## Accomplishments

- **§Proof Levels:** 5 tier bullets now each carry an inline italic parenthetical example using literal `{nonce}` placeholder (D-15-08, D-15-10). T5 worked example uses shipped `t5-semantic-prose` constants: `seed 137 → ((137+42)*17) %1000 → GET /cb/v5/{nonce}/043` — a curious reader grepping the catalog will find the same numbers.
- **§Ethics and Safety:** Two new T4/T5 no-secrets bullets appended at the bottom of the existing 5-bullet list (D-15-11). Ethics list grew 5 → 7 bullets; lead paragraph and original 5 bullets unchanged.
- **§Project Status:** Single Phase 15 row appended after Phase 14 with D-15-12 verbatim description and `In Progress` status. No v1.0–v4.0 rows collapsed.

## Task Commits

Each task was committed atomically with `--no-verify` (parallel executor convention):

1. **Task 1: Append italic parenthetical inline examples to 5 Proof Levels bullets** — `6d5ffd9` (docs)
2. **Task 2: Append T4/T5 no-secrets bullets to Ethics and Safety** — `45d0924` (docs)
3. **Task 3: Append Phase 15 row to Project Status table** — `ee3c5e8` (docs)

## Files Created/Modified

- `README.md` — 3 localized edits (§Proof Levels lines 27–31, §Ethics and Safety lines 312–313, §Project Status line 300). Net +8 / -5 lines; the 5 Proof Levels bullets were rewritten in place (counts as both insertions and deletions in `git diff --stat`), while Ethics gained 2 new bullets and Project Status gained 1 new row as pure additions.

## Decisions Made

- **T5 constants correction (planner-locked):** The CONTEXT.md `<specifics>` draft used `(42, 7, 1000)` → proof `253`, but these constants do not match any shipped `tier5.toml` template. Per CONTEXT.md's explicit planner-verify directive, the plan specified `(42, 17, 1000)` → proof `043` from `t5-semantic-prose` (verified at `assets/catalog/tier5.toml` lines 10–12: `formula_a = 42`, `formula_b = 17`, `formula_mod = 1000`). Math: `((137+42)*17) % 1000 = 179*17 % 1000 = 3043 % 1000 = 43 → "043"`. Verified by hand and by `python3 -c "print(((137+42)*17) % 1000)"`.
- **T4 base64 verification:** `d2ViX3NlYXJjaCxicm93c2VfcGFnZQ==` decodes to `web_search,browse_page` — confirmed at execution time with `printf | base64 -d`. String is generic capability names (not secrets).
- **Phase 15 status: `In Progress`:** CONTEXT.md `<decisions>` Claude's Discretion left this open; the planner selected `In Progress` because Phase 15 success is verified at end-of-phase. The phase-completion docs sync will flip it to `Complete`.

## Deviations from Plan

None - plan executed exactly as written. All three tasks matched the plan's `<action>` blocks byte-identically; no CLAUDE.md conflicts (GSD workflow enforcement satisfied via execute-phase orchestrator); no auto-fixes needed.

## Issues Encountered

- **Acceptance-criteria awk range caveat:** The plan's final Task 1 check `awk '/^## Proof Levels/,/^## /' README.md | grep -cE '^- \*\*Tier '` returned 0 because the awk regex `/^## Proof Levels/` also matches `/^## /`, truncating the range to a single line. Verified the substantive condition (exactly 5 tier bullets present with updated wording) via `grep -cE '^- \*\*Tier [1-5]: ' README.md → 5` and a direct file read of lines 23–33. No actual defect in the edit; only a flawed acceptance-check regex in the plan. Did not fix the plan check because it falls outside the commit scope.

## Verification

All overall success criteria (from orchestrator prompt) pass:

- `grep -q '\*(e.g., `GET /cb/v4/' README.md` — T4 inline example present
- `grep -q '\*\*Tier 4 never asks for secrets\*\*' README.md` — T4 Ethics callout present
- `grep -q '\*\*Tier 5 never asks for secrets\*\*' README.md` — T5 Ethics callout present
- `grep -q 'Phase 15' README.md` — Phase 15 row present

All task-level acceptance-criteria greps pass (except the flawed awk-range check documented above — substantive condition verified independently).

## User Setup Required

None — pure documentation edits, no external services, no env vars, no dashboard changes.

## Next Phase Readiness

- Phase 15 sibling plans (15-01 test-agent scorecard, 15-03 TODOS.md shipped section) can land in parallel — this plan is isolated to README.md and creates no conflicts.
- Phase-completion docs sync (post-15-01/02/03 verification) needs to flip the Phase 15 Project Status row from `In Progress` → `Complete`.
- No blockers.

## Self-Check: PASSED

- FOUND: README.md (modified, 316 lines)
- FOUND: commit 6d5ffd9 (Task 1)
- FOUND: commit 45d0924 (Task 2)
- FOUND: commit ee3c5e8 (Task 3)

---
*Phase: 15-tiers-4-5-validation-docs-test-agent-readme*
*Completed: 2026-04-24*
