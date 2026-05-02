---
quick_id: 260502-f48
slug: move-how-it-works-section-up-to-follow-p
date: 2026-05-02
status: in-progress
---

# Quick Task: Hoist "How It Works" above the ops content

## Problem

`README.md`'s "How It Works" section sits at line 272 — buried beneath *FAQ*, *Deploy Your Own*, *Installation*, and *Usage*. By the time a reader reaches the mechanism explanation, they've already scrolled past 250 lines of ops content. The mental model arrives after the operating manual.

## Fix

Move the `## How It Works` block (currently lines 272–280) to immediately after the `## Proof Levels` block (after line 33), so it sits directly before `## FAQ`. Body content unchanged.

New section order:

1. What This Is
2. Why This Matters for Product Security Teams
3. Proof Levels
4. **How It Works** ← moved here
5. FAQ
6. Deploy Your Own
7. Installation
8. Usage
9. Project Status
10. Ethics and Safety
11. License

Reading flow becomes: *what it is → why it matters → what proof looks like → how it actually works → now go run it.*

## Implementation

Two surgical edits in `README.md`:

1. Remove the existing block (the `## How It Works` header through its 7-step list and trailing blank line) from its current position between `## Usage` and `## Project Status`.
2. Insert the same block between the closing paragraph of `## Proof Levels` ("Each tier's callback URL carries only a unique cryptographic nonce…") and `## FAQ`.

No body changes; pure relocation.

## Verification

- `grep -c "^## How It Works" README.md` → 1 (exactly one occurrence after the move)
- `grep -nE "^## (Proof Levels|How It Works|FAQ|Deploy Your Own)" README.md` → must show that order
- The 7 numbered steps from the original block survive unchanged (`grep -c "^[1-7]\. " README.md` should be ≥ 7 for the Proof block)

## Out of scope

- Trimming the Usage subcommand block (mentioned as a separate consideration; not requested).
- Splitting "How It Works" into a short overview + deeper dive (separate decision).
- Any rewriting of the "How It Works" body.
