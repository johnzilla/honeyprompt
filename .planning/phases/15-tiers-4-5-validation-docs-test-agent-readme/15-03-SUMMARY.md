---
phase: 15-tiers-4-5-validation-docs-test-agent-readme
plan: "15-03"
subsystem: docs
tags: [todos, shipped-log, v5.0, tier4, tier5, markdown]

# Dependency graph
requires:
  - phase: 13-tiers-4-5-backend-payloads-routes-store
    provides: /cb/v4/{nonce}/{b64_list} and /cb/v5/{nonce}/{proof} callback routes (referenced verbatim in Shipped entries)
  - phase: 14-tiers-4-5-surfacing-monitor-tui-report
    provides: Phase-level surfacing context that makes the Shipped log entries coherent
provides:
  - TODOS.md ## Shipped section with Tier 4 and Tier 5 v5.0 entries
  - In-repo browsable record that T4/T5 shipped in v5.0 (Phases 13–15)
  - Closure of DOCS-04 requirement (T4/T5 moved from "future" framing to "shipped")
affects: [future TODOS.md entries, future shipped-log additions, CHANGELOG promotion if later added]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TODOS.md ## Shipped section uses inline **bold** tier prefixes (per D-15-13 + Claude's Discretion compactness choice)"
    - "v5.0 shipped entries reference both phase range (Phases 13–15) and concrete callback route"

key-files:
  created: []
  modified:
    - TODOS.md (added ## Shipped section above existing security-email TODO)

key-decisions:
  - "Used inline **bold** tier prefixes rather than ### sub-headers (D-15-13 Claude's Discretion) — chose compactness for a 2-entry section"
  - "Preserved em-dash (U+2014) as bold-to-description separator and en-dash (U+2013) in Phases 13–15 range to match planning-artifact convention"
  - "Existing security-email TODO preserved byte-identical (git diff shows 0 deletions)"

patterns-established:
  - "Shipped section format: `- **Tier N: Name** — shipped in v5.0 (Phases X–Y). <mechanism + callback route>.`"
  - "Minimal-churn doc updates: add sections above existing content rather than restructuring"

requirements-completed: [DOCS-04]

# Metrics
duration: 1min
completed: 2026-04-24
---

# Phase 15 Plan 03: TODOS.md ## Shipped Section Summary

**Added a `## Shipped` section to TODOS.md with two entries — Tier 4 (Capability Introspection) and Tier 5 (Multi-step Compliance Chain) — both marked shipped in v5.0 (Phases 13–15) with their canonical callback routes referenced inline.**

## Performance

- **Duration:** ~1 min (46s wall clock)
- **Started:** 2026-04-24T22:49:37Z
- **Completed:** 2026-04-24T22:50:23Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- TODOS.md now has a `## Shipped` H2 section above the existing security-email TODO
- Tier 4 entry references `/cb/v4/{nonce}/{b64_list}` with "sorted, base64-encoded tool lists" mechanism
- Tier 5 entry references `/cb/v5/{nonce}/{proof}` with "page-visible seed + deterministic formula + 3-digit proof" mechanism
- Pre-existing `## Set up project-specific disclosure email` block is byte-identical to the pre-edit form (preserved all 8 lines)
- DOCS-04 ("T4/T5 appear under shipped with v5.0 phase references") is fully closed by this single edit

## Task Commits

1. **Task 1: Insert ## Shipped section above existing TODO with Tier 4 and Tier 5 bullets** — `da4962f` (docs)

_No plan-metadata commit yet — parent orchestrator owns STATE/ROADMAP writes after all worktree agents complete._

## Files Created/Modified

- `TODOS.md` — Added `## Shipped` H2 section (5 net lines: 1 H2 header + 1 blank + 2 tier bullets + 1 trailing blank before existing H2) above the pre-existing `## Set up project-specific disclosure email` section. Existing content unchanged.

## Decisions Made

- **Format choice — inline `**bold**` prefixes (not `### Tier N` sub-headers).** D-15-13 Claude's Discretion permitted either; inline bold is more compact for a 2-entry list and matches the existing security-email TODO bullet style.
- **Em-dash (U+2014) kept literal.** Matches README, CONTEXT.md, and planning-artifact style. No ASCII hyphen substitution.
- **En-dash (U+2013) in `Phases 13–15` kept literal.** Matches CONTEXT.md and ROADMAP.md phase-range convention. The acceptance criterion regex `[–-]` accepts either Unicode dash or ASCII hyphen, so both forms satisfy; we picked the en-dash for consistency.
- **Exact entry wording copied verbatim from CONTEXT.md `<specifics>` D-15-13.** Zero rewording — per the plan's explicit instruction to use user-preferred wording.

## Deviations from Plan

None — plan executed exactly as written. Single-task doc edit with no surprises. The Edit tool's exact-match replacement contract enforced the preservation of the existing TODO block (threat T-15-03-02 mitigation), and the verifiable-claim mitigation (T-15-03-01) holds via ROADMAP.md which shows Phases 13–14 complete.

## Issues Encountered

None.

## Verification

All 12 acceptance criteria passed:

| # | Check | Result |
|---|-------|--------|
| 1 | `grep -q '^## Shipped$' TODOS.md` | PASS |
| 2 | `grep -q '\*\*Tier 4: Capability Introspection\*\*' TODOS.md` | PASS |
| 3 | `grep -q '\*\*Tier 5: Multi-step Compliance Chain\*\*' TODOS.md` | PASS |
| 4 | `grep -qE 'shipped in v5\.0 \(Phases 13[–-]15\)' TODOS.md` | PASS |
| 5 | `grep -q '/cb/v4/{nonce}/{b64_list}' TODOS.md` | PASS |
| 6 | `grep -q '/cb/v5/{nonce}/{proof}' TODOS.md` | PASS |
| 7 | `grep -q '## Set up project-specific disclosure email' TODOS.md` | PASS |
| 8 | `grep -q 'security@honeyprompt.dev' TODOS.md` | PASS |
| 9 | `grep -q 'Added:\*\* 2026-03-31 via /plan-eng-review' TODOS.md` | PASS |
| 10 | `grep -c '^## Shipped$' TODOS.md` returns `1` | PASS (count=1) |
| 11 | `grep -cE '^- \*\*Tier [45]:' TODOS.md` returns `2` | PASS (count=2) |
| 12 | `git diff TODOS.md` deletions = 0 | PASS (0 deletions, 5 insertions) |

**Diff stats:** `1 file changed, 5 insertions(+)` — exactly matches the verification spec of ~4–5 net additions with 0 deletions.

**Dash-character choice:** em-dash (U+2014) as bold-to-description separator; en-dash (U+2013) inside `Phases 13–15`. Both Unicode, no ASCII fallback applied.

## User Setup Required

None — no external service configuration required. Pure documentation edit.

## Next Phase Readiness

- DOCS-04 closed; all four Phase 15 doc-update success criteria progress continues to accumulate in wave 1.
- No downstream blockers introduced; the Shipped entries are self-contained and verifiable against ROADMAP.md Phase 13/14 completion rows.
- Future additions to the Shipped section (e.g., after v6.0) can follow the same `- **Tier/Feature: Name** — shipped in vX.Y (Phases A–B). <mechanism + route>.` pattern.

## Self-Check: PASSED

- FOUND: TODOS.md (H2 `## Shipped` present, both tier bullets present, existing TODO preserved)
- FOUND commit: da4962f in git log --oneline

---
*Phase: 15-tiers-4-5-validation-docs-test-agent-readme*
*Completed: 2026-04-24*
