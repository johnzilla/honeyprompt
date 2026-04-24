# Phase 14: Tiers 4 & 5 Surfacing (Monitor TUI + Report) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-24
**Phase:** 14-tiers-4-5-surfacing-monitor-tui-report
**Areas discussed:** TUI evidence placement, TUI chrome extension, Report T4/T5 granularity, Backward-compat rendering

---

## Gray Area Selection

| Option | Description | Selected |
|--------|-------------|----------|
| TUI evidence placement | Where/how T4 tool lists and T5 proofs appear in the Monitor TUI | ✓ |
| TUI chrome extension | How to fit 5 tiers into stats header, filter bar, help overlay | ✓ |
| Report T4/T5 granularity | Per-event vs session-grouped T4/T5 rendering | ✓ |
| Backward-compat rendering | How chrome behaves on legacy v4.0 DBs | ✓ |

**User's choice:** All four areas selected.

---

## TUI Evidence Placement

### Q1: Primary placement for T4/T5 evidence

| Option | Description | Selected |
|--------|-------------|----------|
| Detail pane on row select | Main table unchanged; pane below shows full T4/T5 for selected row | |
| New EVIDENCE column | Single column in main table, blank for T1–T3 | |
| Both — compact col + detail pane | Short EVIDENCE column for scan + detail pane for full context (Recommended) | ✓ |

**User's choice:** Both — compact col + detail pane.

### Q2: Detail pane trigger and T1–T3 content

| Option | Description | Selected |
|--------|-------------|----------|
| Always visible, context-aware | Fixed pane; T4/T5 show evidence, T1–T3 show payload_id+embedding+nonce (Recommended) | ✓ |
| Always visible, T4/T5 only | Pane appears only for T4/T5 rows; blank otherwise | |
| Toggle-on-demand | Enter/`d` key toggles pane open for selected row | |

**User's choice:** Always visible, context-aware.

### Q3: T4 capability truncation in EVIDENCE column

| Option | Description | Selected |
|--------|-------------|----------|
| Truncate with `…` at column width | First N chars then `…`; full list in detail pane (Recommended) | ✓ |
| Show tool count + first tool | `5: web_search…` or `5 tools` | |
| Show sorted tools raw, let it overflow UA col | Size column generously (~30 chars) | |

**User's choice:** Truncate with `…`.

### Q4: T5 visual style

| Option | Description | Selected |
|--------|-------------|----------|
| `123 ✓` green / `123 ✗` red | Colored glyph after digits (Recommended) | ✓ |
| `✓ 123` glyph first | Symbol leads for faster scanning | |
| Tint the whole row | Green/red row background | |
| Plain text `123 VALID` / `123 INVALID` | No glyphs | |

**User's choice:** `123 ✓` green / `123 ✗` red.

---

## TUI Chrome Extension

### Q1: Stats header tier counts

| Option | Description | Selected |
|--------|-------------|----------|
| Append inline: `T1:n T2:n T3:n T4:n T5:n` | Extend existing format (Recommended) | ✓ |
| Split T5 into `T5✓:n T5✗:n` | Separate valid/invalid counts | |
| Second stats line | New row for T4/T5 | |
| Hide zero-count tiers | Show `Tx:n` only when n > 0 | |

**User's choice:** Append inline.

### Q2: Tab filter cycle

| Option | Description | Selected |
|--------|-------------|----------|
| Extend cycle: All→T1→T2→T3→T4→T5→All | Symmetric extension (Recommended) | ✓ |
| Skip-empty cycle | Tab only cycles through tiers with events | |
| Keep cycle at T1–T3, add :filter command only | Asymmetric; preserves muscle memory | |

**User's choice:** Extend cycle symmetrically.

### Q3: T5 valid/invalid split surface

| Option | Description | Selected |
|--------|-------------|----------|
| Detail pane only | Per-row ✓/✗ glyph + combined `T5:n` header (Recommended) | ✓ |
| Header shows split | `T5✓:n T5✗:n` in stats header | |
| Filter split: t5valid/t5invalid | New command variants | |

**User's choice:** Detail pane only.

---

## Report T4/T5 Granularity

### Q1: Report shape

| Option | Description | Selected |
|--------|-------------|----------|
| Extend evidence table with EVIDENCE column | Single new column for T4/T5 evidence (Recommended) | ✓ |
| Separate T4 and T5 subsections | Focused sub-tables per tier | |
| Per-event rendering for T4/T5 only | Mixed granularity | |

**User's choice:** Extend evidence table with EVIDENCE column.

### Q2: T4 format in Markdown cell

| Option | Description | Selected |
|--------|-------------|----------|
| Full list, md_escape'd | Complete sorted comma list, uniform pipeline (Recommended) | ✓ |
| Truncate at N chars with `…` | Cap ~40 chars | |
| Full list wrapped in backticks | Render as code-span | |

**User's choice:** Full list, md_escape'd.

### Q3: T5 format in Markdown cell

| Option | Description | Selected |
|--------|-------------|----------|
| `123 ✓ VALID` / `123 ✗ INVALID` | Glyph + text (Recommended) | ✓ |
| `123` + separate `Proof Valid` column | Two columns | |
| `**123** (valid)` / `~~123~~ (invalid)` | Markdown-styled | |

**User's choice:** `123 ✓ VALID` / `123 ✗ INVALID`.

---

## Backward-Compat Rendering

### Q1: Interpretation of "no empty T4/T5 sections printed"

| Option | Description | Selected |
|--------|-------------|----------|
| Always show chrome, zero-count is fine | 5 exec summary rows, `T4:0 T5:0` in header, full filter cycle (Recommended) | ✓ |
| Skip empty chrome end-to-end | Omit T4/T5 rows/labels until first event | |
| Hybrid — report hides, TUI always shows | Static report suppresses; live TUI always full | |

**User's choice:** Always show chrome, zero-count is fine.

### Q2: T1–T3 row content in Evidence column

| Option | Description | Selected |
|--------|-------------|----------|
| Em-dash `—` | Consistent with existing empty-row placeholder (Recommended) | ✓ |
| Blank cell | Leave literally empty | |
| Proof-level recap | `(arbitrary)` / `(conditional)` / `(computed)` | |

**User's choice:** Em-dash `—`.

### Q3: Schema probe strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Always query; rely on NULL handling | Trust Phase 13 migration guarantee (Recommended) | ✓ |
| Probe schema then branch | `PRAGMA table_info` check | |

**User's choice:** Always query; rely on NULL handling.

---

## Claude's Discretion

- Exact EVIDENCE column width and which existing column tightens to make room
- Detail-pane height (1 vs 2–3 lines)
- Exact wording of detail-pane labels
- Specific Ratatui `Color` variants for T4 and T5 tier labels
- Whether `ReportSummary` gains fields vs `ReportSession` gains optional fields
- Column ordering in the Markdown evidence table
- Filter bar wrap/truncate behavior if six labels overflow 80 cols

## Deferred Ideas

- `:filter t5valid` / `:filter t5invalid` split commands
- Header split `T5✓:n T5✗:n`
- Per-event evidence rendering (vs session-grouped)
- Schema-probe via `PRAGMA table_info`
- `test-agent` T4/T5 scorecard + CI exit codes → Phase 15
- README Proof Levels 5-tier + TODOS.md cleanup → Phase 15
- JSON / HTML report formats → out of scope at milestone level
- Web dashboard for T4/T5 → out of scope at milestone level
