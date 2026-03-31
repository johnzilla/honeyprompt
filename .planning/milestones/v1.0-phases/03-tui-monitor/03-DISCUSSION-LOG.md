# Phase 3: TUI Monitor - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 03-tui-monitor
**Areas discussed:** Layout and information density, Filtering and sorting UX, Visual treatment of replays, Monitor invocation model

---

## Layout and Information Density

| Option | Description | Selected |
|--------|-------------|----------|
| Event table with header stats | top-style: stats header bar + scrolling event table | ✓ |
| Dashboard with panels | Split screen: event table + stats/charts side by side | |
| Full-width event table | Maximizes event visibility, stats in compact status bar | |

**User's choice:** Event table with header stats, modeled after `top`
**Notes:** All 9 AppEvent fields as columns (timestamp, tier, classification, IP, UA, session ID, nonce, fire count, replay flag). User noted top shows ~12 columns so 9 is reasonable.

---

## Filtering and Sorting UX

| Option | Description | Selected |
|--------|-------------|----------|
| Single-key shortcuts | top-style: t for tier, s for sort, / for search | |
| Vim-style keys | j/k scroll, tab cycle filters, : for commands | ✓ |
| Visible filter bar | Persistent filter row with key hints | |

**User's choice:** Vim-style keys
**Notes:** Fits the security researcher audience who are comfortable with terminal tools.

---

## Visual Treatment of Replays

| Option | Description | Selected |
|--------|-------------|----------|
| Dimmed rows | Replays shown with muted/gray text | |
| Hidden by default | Replays filtered out, toggle to show | ✓ |
| Inline badge | Replays shown with [REPLAY] tag and color | |

**User's choice:** Hidden by default, with indicator in header stats showing replay count
**Notes:** User wanted replays hidden but with a visible counter so users know to toggle when needed.

---

## Monitor Invocation Model

| Option | Description | Selected |
|--------|-------------|----------|
| Integrated mode | monitor starts the server itself, one process | |
| Attach to running server | monitor connects to separate serve process | |
| Both | Integrated by default, --attach for existing server | ✓ |

**User's choice:** Both — integrated default, attach option for headless setups
**Notes:** Zero-friction for demos (integrated), production flexibility (attach).

---

## Claude's Discretion

- Ratatui widget selection and layout grid
- Exact key bindings beyond j/k/tab/:
- Stats header formatting
- Color scheme
- Attach mode connection mechanism
- : command vocabulary

## Deferred Ideas

None
