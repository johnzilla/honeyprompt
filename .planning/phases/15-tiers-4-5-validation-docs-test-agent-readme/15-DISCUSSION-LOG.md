# Phase 15: Tiers 4 & 5 Validation & Docs (test-agent + README) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-24
**Phase:** 15-tiers-4-5-validation-docs-test-agent-readme
**Areas discussed:** Scorecard verdict & layout, Tier storage shape, README Proof Levels examples, README Ethics T4/T5 callouts (+ Project Status + TODOS.md)

---

## Gray Area Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Scorecard verdict & layout | 5-tier verdict thresholds + text/JSON rendering | ✓ |
| Tier storage shape | Fixed `[_; 5]` vs Vec vs BTreeMap | ✓ |
| README Proof Levels examples | Where concrete examples live | ✓ |
| README Ethics T4/T5 callouts | How to explicitly call out T4/T5 no-secrets | ✓ |

**User's choice:** All four areas.

---

## Scorecard Verdict & Layout

### Verdict thresholds

| Option | Description | Selected |
|--------|-------------|----------|
| Symmetric extension | FULLY=5, NO=0, PARTIAL=1–4. Preserves contract. | ✓ |
| Add nuance verdicts | BASIC_COMPLIANT / ADVANCED_COMPLIANT / FULLY_COMPLIANT | |
| Drop verdict, keep counts | Remove verdict string, keep per-tier booleans + score | |

**User's choice:** Symmetric extension.
**Notes:** Preserves existing CI-parser contracts; the only observable shift is FULLY_COMPLIANT is now harder to reach.

### Text scorecard layout

| Option | Description | Selected |
|--------|-------------|----------|
| Flat 5-line list | 5 `tier N:` lines, same alignment as today | ✓ |
| Grouped basic/advanced | Sub-headers for T1–T3 and T4–T5 | |
| Add evidence detail per tier | Show decoded T4 list / T5 proof status | |

**User's choice:** Flat 5-line list.
**Notes:** Keeps `tier N:` grep pattern stable for CI scripts.

### Score string

| Option | Description | Selected |
|--------|-------------|----------|
| `n/5` | Direct extension of today's `n/3` | ✓ |
| `n/5 (basic: a/3, advanced: b/2)` | Inline basic/advanced breakdown | |

**User's choice:** `n/5`.
**Notes:** Preserves `n/m` regex pattern for CI dashboards.

### JSON output

| Option | Description | Selected |
|--------|-------------|----------|
| Extend to 5 entries | Same shape, 5 tier entries instead of 3 | ✓ |
| Add version field | Bump schema with `scorecard_version: 2` | |

**User's choice:** Extend to 5 entries.
**Notes:** Minimal breakage; consumers parsing `tiers[N]` see two new rows and still work.

---

## Tier Storage Shape

### Scorecard tier field shape

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed `[u32; 5]` / `[bool; 5]` | Minimal churn, compile-time bounds | ✓ |
| `Vec<bool>` / `Vec<u32>` | Runtime-sized, future-proof | |
| `BTreeMap<u8, bool>` / `BTreeMap<u8, u32>` | Keyed by tier number | |

**User's choice:** Fixed `[u32; 5]` / `[bool; 5]`.
**Notes:** Matches the prior pattern; v5.0 is the terminal tier-expansion milestone so a future-proof shape isn't needed.

### `detections_by_tier` signature

| Option | Description | Selected |
|--------|-------------|----------|
| Extend loop to `1..=5` | One-line change, return `[u32; 5]` | ✓ |
| Parameterize over tier range | `detections_by_tiers(&[u8])` | |

**User's choice:** Extend loop to `1..=5`.
**Notes:** No caller today needs a filtered range; simplest change that satisfies the requirement.

---

## README Proof Levels Examples

| Option | Description | Selected |
|--------|-------------|----------|
| Inline parenthetical per bullet | `*(e.g., GET /cb/vX/{nonce}/…)*` appended to each bullet | ✓ |
| Sub-bullet with example URL + decoded value | Two-level list, nested example bullet | |
| Separate examples table | 3-column table below the bullet list | |
| Fenced code block per tier | Code block under each bullet | |

**User's choice:** Inline parenthetical per bullet.
**Notes:** Preserves the existing 5-bullet structure; minimal density increase. T5 example uses a worked formula showing the chain `seed 137 → (137+42)*7 %1000 → 253` so the mechanic is self-evident.

---

## README Ethics T4/T5 Callouts

### Ethics section structure

| Option | Description | Selected |
|--------|-------------|----------|
| Two new bullets in existing list | Extend the 5-bullet list with T4 and T5 bullets | ✓ |
| Dedicated paragraph after the list | Prose block explaining T4/T5 specifically | |
| 'What each tier does and does not collect' subsection | Table mapping each tier to what it does and doesn't request | |

**User's choice:** Two new bullets in existing list.
**Notes:** Minimal structural change. Fits the existing bullet cadence. User preview-selected exact wording, preserved in CONTEXT.md `<specifics>`.

### Project Status row

| Option | Description | Selected |
|--------|-------------|----------|
| Add Phase 15 row after Phase 14 | One new row at table bottom | ✓ |
| Combine Phase 13–15 into single v5.0 row | Collapse for brevity | |

**User's choice:** Add Phase 15 row after Phase 14.
**Notes:** Per-phase rows match v1.0–v4.0 style and preserve v5.0 sub-milestone granularity.

### TODOS.md

| Option | Description | Selected |
|--------|-------------|----------|
| Add `## Shipped` section listing T4/T5 | New section above existing security-email TODO | ✓ |
| Leave TODOS.md alone | Criterion vacuously satisfied | |
| Point TODOS.md at CHANGELOG/PROJECT.md | Stub file pointing elsewhere | |

**User's choice:** Add `## Shipped` section listing T4/T5.
**Notes:** Literally satisfies criterion #4 ("T4/T5 appear under shipped with v5.0 phase references"). Keeps existing security-email TODO intact.

---

## Claude's Discretion

- Exact prose around T1/T2/T3 inline examples (URL pattern is fixed; description prose is flexible)
- Specific T5 formula constants chosen for README worked example (must match a shipped catalog entry or be marked illustrative)
- Whether `## Shipped` uses sub-headers or inline bold prefixes
- Precise placement of the two new Ethics bullets within the existing list
- Whether Phase 15 Project Status row is `In Progress` vs `Complete` at doc-update time

## Deferred Ideas

- Adding a `scorecard_version` field to JSON output
- Parameterized `detections_by_tiers(&[u8])` store function
- Per-tier evidence inside the scorecard text
- Basic/advanced nuance verdicts (`BASIC_COMPLIANT`, `ADVANCED_COMPLIANT`)
- Basic/advanced score breakdown inline
- Collapsing Phase 13/14/15 into a single v5.0 Project Status row
- Replacing TODOS.md with a stub pointing at ROADMAP.md
- New CI workflow additions for T4/T5
- Dedicated CHANGELOG.md file
