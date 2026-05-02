---
quick_id: 260502-f48
slug: move-how-it-works-section-up-to-follow-p
date: 2026-05-02
status: complete
---

# Summary: Hoist "How It Works" above the ops content

## Outcome

Moved the `## How It Works` block in `README.md` from line 272 (between *Usage* and *Project Status*) to line 35 (between *Proof Levels* and *FAQ*). Pure relocation — body unchanged.

## Section order before → after

| # | Before | After |
|---|--------|-------|
| 1 | What This Is | What This Is |
| 2 | Why This Matters | Why This Matters |
| 3 | Proof Levels | Proof Levels |
| 4 | FAQ | **How It Works** |
| 5 | Deploy Your Own | FAQ |
| 6 | Installation | Deploy Your Own |
| 7 | Usage | Installation |
| 8 | **How It Works** | Usage |
| 9 | Project Status | Project Status |
| 10 | Ethics and Safety | Ethics and Safety |
| 11 | License | License |

## Verification

- `grep -c "^## How It Works" README.md` → `1` (no duplication)
- H2 grep confirms `Proof Levels (23) → How It Works (35) → FAQ (45) → Deploy Your Own (49)`
- `git diff --stat`: 10 insertions, 10 deletions on `README.md` — symmetrical, consistent with pure relocation

## Commits

- `b37bf36` — docs: hoist "How It Works" above the ops content in README

## Notes

Doc-only change; skipped Rust fmt/clippy/test. Deferred (not done in this task): trimming the *Usage* subcommand block, splitting *How It Works* into a short overview + deeper dive — both flagged as separate decisions.
