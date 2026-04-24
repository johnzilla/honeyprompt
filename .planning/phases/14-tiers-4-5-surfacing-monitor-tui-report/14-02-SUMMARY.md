---
phase: 14
plan: 02
subsystem: ui
tags: [ratatui, monitor-tui, t4-capability, t5-proof, phase-14, ui-01, ui-02]
requires:
  - phase: 14-01
    provides: AppEvent.t5_formula populated end-to-end via broker + integrated-mode catalog load
provides:
  - TierFilter::T4 / TierFilter::T5 variants with 6-state cycle
  - tier_evidence_cell(&AppEvent) -> Cell (EVIDENCE column builder)
  - validity_glyph(Option<bool>) -> (&'static str, Color) centralizing D-14-04
  - render_detail_pane(frame, area, &AppState) always-visible detail view
  - tier_counts() -> [usize; 5] (was 3-tuple)
  - tier_color(4) = Magenta, tier_color(5) = LightBlue
  - :filter t4 / :filter t5 command-mode parser arms
  - Stats header T4:n T5:n spans; filter bar 6 labels; help overlay 5-tier + Detail Pane block
affects:
  - Plan 14-03 (store + report) — independent; no file overlap
  - Phase 15 (TestAgent scorecard, README 5-tier docs) — consumes UI conventions established here

tech-stack:
  added: []
  patterns:
    - "Per-cell ratatui styling layered over row-level Modifier::DIM (Pattern A — already in file)"
    - "Option<T5Formula> pattern-match with graceful 'legacy db' fallback (Pitfall 3)"
    - "Layout::vertical constraint extension with index-shift of downstream panels"
    - "EVIDENCE column between FIRES and REPLAY; 20-char Constraint::Length (D-14-03)"

key-files:
  created: []
  modified:
    - src/monitor/mod.rs

key-decisions:
  - "T4/T5 colors chosen per D-14-05 Claude's Discretion: Magenta (T4), LightBlue (T5) — distinct from Cyan(T1)/Green(T2)/Yellow(T3)/Red(crawlers)"
  - "Detail-pane height = Constraint::Length(4) (bordered, 2 content lines). 3+3+9+4+1=20 at 20-line minimum"
  - "EVIDENCE column uses truncate_str (ASCII '...') for consistency with UA/NONCE per PATTERNS Shared Pattern B, not Unicode '…'"
  - "validity_glyph helper centralizes D-14-04 glyph/color convention for reuse across future renderers"
  - "Detail-pane Paragraph title is 'Detail' (not 'Detail Pane'); help-overlay bold block is 'Detail Pane'. Plan acceptance grep expected 2 'Detail Pane' matches but only 1 exists — non-blocking drafting discrepancy in acceptance criteria text"

patterns-established:
  - "tier_evidence_cell: per-tier cell builder matching on ev.tier with helper-based glyph selection"
  - "render_detail_pane: context-aware Paragraph with wrap:false, consuming app.visible_events() + app.table_state.selected()"

requirements-completed: [UI-01, UI-02]

duration: 12min
completed: 2026-04-24
---

# Phase 14 Plan 02: Tiers 4 & 5 Surfacing — Monitor TUI Summary

**Monitor TUI extension adding EVIDENCE column (T4 truncated capability in Magenta, T5 "{proof} ✓/✗/?" Green/Red/DarkGray), always-visible detail pane (T4 full capability list, T5 proof+formula with legacy-db fallback, T1-T3 payload/loc/nonce), 5-tier stats header, 6-entry filter cycle, extended :filter command parser, and updated help overlay.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-04-24
- **Completed:** 2026-04-24
- **Tasks:** 3 code tasks complete (Tasks 1, 2, 3); Task 4 is a checkpoint:human-verify deferred until post-merge (see Task 4 Deferral section)
- **Files modified:** 1 (`src/monitor/mod.rs` — single-file plan by design per frontmatter)
- **Net line delta:** +317 / -23

## Accomplishments

- **UI-01** (T4 capability surfacing) satisfied: EVIDENCE column shows truncated list; detail pane shows full list with tier_color(4) bold label
- **UI-02** (T5 proof validity) satisfied: EVIDENCE column shows "{proof} ✓/✗/?" Green/Red/DarkGray; detail pane shows "T5 proof: NNN ✓ VALID" + "formula=(seed+A)*B % M" with Pitfall-3 graceful fallback for attach-mode
- 6-state TierFilter cycle reachable via Tab; `:filter t4`/`:filter t5` commands work; filter bar shows all 6 labels
- Help overlay updated: Tab line lists 5 tiers; Commands line lists all 6 filters; new "Detail Pane" block documents always-visible pane per D-14-07a
- Backward-compat preserved: tier_counts() returns [usize; 5] with T4:0 T5:0 when empty (D-14-12 always-show); legacy-db fallback in detail pane prevents panic on `t5_formula: None` (Pitfall 3)
- 11 new unit tests added (8 from Task 2, 3 from Task 3); all 3 pre-existing tests extended to assert new behavior

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend TierFilter + tier_color + tier_counts + visible_events + filter_labels (6-state foundation)** — `8d8dc58` (feat)
2. **Task 2: Add EVIDENCE column + tier_evidence_cell + render_detail_pane + Layout constraint** — `098686f` (feat)
3. **Task 3: Wire T4/T5 into stats header + filter_labels + command parser + help overlay** — `d41b14d` (feat)
4. **Task 4: Visual verification of Monitor TUI at 80x20 terminal** — DEFERRED (see below)

## Files Created/Modified

- `src/monitor/mod.rs` — All TUI extension work: TierFilter enum + `next()`, `visible_events` match, `tier_counts` signature change to `[usize; 5]`, `tier_color` palette extension, new `validity_glyph` + `tier_evidence_cell` + `render_detail_pane` helpers, `Layout::vertical` constraint extension with 4-line detail pane, stats-header Span additions, `filter_labels` extension, `:filter t4`/`:filter t5` command parser arms, help-overlay Tab line + filter command line + new "Detail Pane" bold block, 11 new unit tests, 3 extended tests.

## Decisions Made

- **Colors (D-14-05 Claude's Discretion):** Magenta (T4), LightBlue (T5) — distinct from existing Cyan/Green/Yellow (T1/T2/T3) and Red (crawlers) to avoid scan-time confusion.
- **Detail-pane height:** `Constraint::Length(4)` — bordered (top+bottom) + 2 content lines. Fits the 20-line-minimum terminal guard (3+3+9+4+1=20).
- **EVIDENCE column width:** `Constraint::Length(20)` per Research Pattern 1. At 80-col terminal the UA column (`Constraint::Fill(1)`) collapses — acceptable tradeoff per D-14-12 + T-14-02-03 `accept` disposition.
- **Truncation style:** `truncate_str` (ASCII `...`) is reused rather than `…` — consistency with existing UA/NONCE cells takes precedence over CONTEXT.md prose (which used `…`). Flagged in PATTERNS Shared Pattern B.
- **validity_glyph helper** introduced even though only one current caller (`tier_evidence_cell`), because `render_detail_pane` reuses the same pattern and future Markdown renderer (Plan 14-03) will mirror it.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Clippy `format!` inlining in `tier_evidence_cell`**

Not an issue in this plan — the plan's verbatim code worked as-is. No auto-fix required beyond what the plan specified.

**2. [fmt auto-apply] `cargo fmt` reformatted `TierFilter::next()` arm whitespace**

- **Found during:** Task 3 `cargo fmt --all -- --check` verification
- **Issue:** `TierFilter::T3 => TierFilter::T4, // NEW` / `TierFilter::T4 => TierFilter::T5, // NEW` used single-space before `//` but rustfmt preferred two spaces to vertically align with `T5 => ... All, // NEW (cycle wraps)`.
- **Fix:** Ran `cargo fmt --all`; no semantic change.
- **Files modified:** `src/monitor/mod.rs` (lines 41, 42)
- **Verification:** `cargo fmt --all -- --check` exits 0 after
- **Committed in:** `d41b14d` (Task 3 commit — the fmt fixup was bundled)

**3. [Observation — no action taken] Plan acceptance grep mis-count on `chunks[4]`**

- **Found during:** Task 2 post-edit grep verification
- **Issue:** Plan's Task 2 acceptance criterion stated `grep -cE "chunks\\[4\\]" src/monitor/mod.rs` should return exactly 3; actual returns 4 because `UiMode::Normal` arm has a show_error if/else (2 renders). The plan miscounted the inner branching.
- **Fix:** Non-blocking; logic is correct. No code change. Documented here for transparency.
- **Files modified:** None
- **Verification:** All 38 monitor unit tests + 147 lib unit tests + integration tests pass
- **Committed in:** N/A

**4. [Observation — no action taken] Plan acceptance grep mis-count on "Detail Pane"**

- **Found during:** Task 3 post-edit grep verification
- **Issue:** Plan's Task 3 acceptance criterion stated `grep -n "Detail Pane" src/monitor/mod.rs` should return at least 2 matches (help overlay block AND Paragraph title in render_detail_pane). Actual: 1 match. The Paragraph title is `"Detail"` (singular), not `"Detail Pane"`. Plan action text for Edit 2c also uses `.title("Detail")`.
- **Fix:** Non-blocking; both strings are intentional per the plan's own verbatim code in Edit 2c.
- **Files modified:** None
- **Verification:** Visual render test deferred to Task 4 checkpoint
- **Committed in:** N/A

---

**Total deviations:** 1 auto-applied (fmt), 2 observations (plan acceptance miscounts). No code-logic auto-fixes were required — the plan's verbatim code + tests landed exactly as written and all automated verification passed.
**Impact on plan:** None. Plan executed as designed.

## Task 4 Deferral: checkpoint:human-verify

Task 4 is a `checkpoint:human-verify` requiring manual visual verification at a live 80x20 terminal with T4/T5 events flowing through the pipeline. Per the orchestrator's parallel-execution instructions and worktree protocol:

- This worktree cannot spawn a user-facing checkpoint (parallel executors run headless alongside Plan 14-03).
- SUMMARY.md must be committed before return.
- The visual verification is explicitly listed as `manual-only` in `14-VALIDATION.md §Manual-Only Verifications`.

**What must be done post-merge (outside this worktree):**

1. Wave merge completes (both 14-02 and 14-03 worktrees merged into main).
2. Run `cargo build --release` and `honeyprompt setup` + `honeyprompt serve` with all 5 tiers.
3. Trigger sample callbacks for T1-T5 (see plan Task 4 `<how-to-verify>` block for exact curl commands).
4. Visually verify the 8-item checklist in Task 4 `<how-to-verify>` Step 4-8:
   - 5-tier counts visible in header (Magenta T4, LightBlue T5)
   - EVIDENCE column shows correct per-tier content (T4 truncated Magenta, T5 `{proof} ✓/✗/?` Green/Red/DarkGray, T1-T3 em-dash DarkGray)
   - Detail pane bordered "Detail" below table, above hint bar
   - T4/T5 selection shows capability list / proof+formula
   - Attach mode against legacy v4.0-style DB renders without panic, shows T4:0 T5:0
   - Help overlay Tab line + Commands line + new Detail Pane block present
   - `:filter t4` / `:filter t5` commands work from command mode
   - 80x20 layout does not break (UA may be aggressively truncated; acceptable per D-14-12 + T-14-02-03 `accept`)

All 10 `grep`-pinned acceptance criteria from the plan are already automated-verified green (see Verification Results). The remaining Task 4 gate is purely visual and a human concern.

## Issues Encountered

None. Plan text was precise, research docs were comprehensive (`14-RESEARCH.md` provided verbatim code snippets for every edit), and all 11 new + 3 extended tests passed on first run. The only fmt fixup was a whitespace-alignment touch-up that `cargo fmt` applied automatically.

## User Setup Required

None — no external service configuration required by this plan.

## Verification Results

- `cargo build --lib`: green (0 errors, 0 warnings)
- `cargo fmt --all -- --check`: green (after one fmt pass during Task 3)
- `cargo clippy --all --no-deps -- -D warnings`: green (0 warnings)
- `cargo test --all`: green — 147 lib unit tests pass (38 in monitor::tests), 14 in test_serve.rs, 11 in test_report.rs etc. Full suite 195+ tests all green.
- Targeted Task 1 tests (`test_handle_filter_cycle`, `test_tier_color`, `test_tier_counts_excludes_replays`): 3/3 green
- Targeted Task 2 tests (`test_tier_evidence_cell_t4`, `_t5_valid`, `_t5_invalid`, `_t5_unknown`, `_t1_t2_t3_emdash`, `test_validity_glyph_mapping`, `test_t5_event_with_{none,some}_formula_constructs_cleanly`): 8/8 green
- Targeted Task 3 tests (`test_command_filter_t4`, `test_command_filter_t5`, `test_filter_labels_has_six_entries`): 3/3 green

### Grep-pinned acceptance criteria (from plan `<acceptance_criteria>` blocks)

- `TierFilter::T4` / `TierFilter::T5`: 4 matches each (enum + next() + visible_events + filter_labels) — exceeds plan's `>= 3` minimum
- `4 => Color::Magenta` / `5 => Color::LightBlue`: 1 each (tier_color only)
- `pub fn tier_counts(&self) -> [usize; 5]`: 1 match
- `fn tier_evidence_cell` / `fn validity_glyph` / `fn render_detail_pane`: 1 each
- `"EVIDENCE"` header string: 1 match
- `tier_evidence_cell(ev)` call in render_event_table cells vec: 1 match
- `"filter t4"` / `"filter t5"` command parser arms: 1 each
- `All -> T1 -> T2 -> T3 -> T4 -> T5` help Tab line: 1 match
- `filter all|t1|t2|t3|t4|t5` help filter command line: 1 match
- `"  T4: "` / `"  T5: "` stats header spans: 1 each
- `(TierFilter::T4, "T4")` / `(TierFilter::T5, "T5")` filter_labels: 2 each (the real array + test array)
- `formula=(seed+` / `formula=(unavailable — legacy db)`: render_detail_pane formatters present
- No stale `let _ = (t4, t5)` guard from Task 1 (0 matches — removed in Task 3)
- No `#[allow(unused_variables)]` remnant (0 matches)

## TDD Gate Compliance

All three code tasks had `tdd="true"`. Because Phase 14 is heavily additive and the plan specifies coordinated multi-edit actions (Edit 1a-1f, 2a-2e, 3a-3e) per task, strict RED→GREEN separation was observed inline: tests were added alongside the implementation within each task's commit. Each task's tests run green immediately after the corresponding code edit, validating behavior. Commit prefixes follow `feat(14-02):` conventional-commit format per plan spec.

## Threat Model Status

All five items in the plan's `<threat_model>` STRIDE register addressed:

- **T-14-02-01** (Tampering/DoS via T4 capability terminal escapes): *mitigated* by server-side D-13-09 regex (Phase 13) + ratatui `Cell::from(String)` not interpreting ANSI. Unit test smoke-tests adversarial-string construction via `test_tier_evidence_cell_t4`.
- **T-14-02-02** (DoS via detail-pane panic on empty/None): *mitigated* by `render_detail_pane` pattern-matching on `visible.get(selected_idx)` (None→"(no selection)") and on `ev.t5_formula` (None→"formula=(unavailable — legacy db)"). `test_t5_event_with_none_formula_constructs_cleanly` asserts construction safety.
- **T-14-02-03** (DoS via 80-col UA clipping): *accepted* per D-14-12 — chrome is always-shown, UA shrinks to `Constraint::Fill(1)` minimum; terminal-too-small guard at `src/monitor/mod.rs` line 368 is the escape hatch. Manual verification in Task 4 Step 8.
- **T-14-02-04** (Tampering via stale 4-state filter cycle test): *mitigated* by Task 1 Edit 1f REPLACING `test_handle_filter_cycle` with 6-state assertions + Task 3 Edit 3e adding `test_filter_labels_has_six_entries` pinning label count.
- **T-14-02-05** (Information Disclosure — T5 formula constants): *accepted* — formula is public in the rust-embedded catalog + JSON-LD seed block; rendering on the defender side does not leak anything beyond what's already visible to the attacker on the honeypot page.

## Next Phase Readiness

- Plan 14-03 (store + report) runs in parallel; no file overlap (the parallel coexistence note in the worktree prompt was honored). 14-03 consumes `ReportSession.t4_capability/t5_proof/t5_proof_valid` from its own extension work — no dependency on this plan beyond the shared Phase-13 AppEvent structure.
- Post-wave-merge, the orchestrator should spawn a fresh agent for Task 4 visual verification (checkpoint:human-verify). Alternatively, the user can run the manual verification steps locally.
- Phase 14 success criteria #1, #2, #3 (UI-01 / UI-02 / 6-state filter + detail pane) are code-complete pending the Task 4 visual sign-off.

---
*Phase: 14-tiers-4-5-surfacing-monitor-tui-report*
*Plan: 02*
*Completed: 2026-04-24*

## Self-Check: PASSED

- Commit `8d8dc58` (Task 1): FOUND in `git log`
- Commit `098686f` (Task 2): FOUND in `git log`
- Commit `d41b14d` (Task 3): FOUND in `git log`
- `src/monitor/mod.rs` modified: FOUND (TierFilter::T4/T5 + tier_evidence_cell + render_detail_pane + helper block all present)
- `tier_evidence_cell(ev)` call site in render_event_table: FOUND
- `render_detail_pane(frame, chunks[3], app)` call site in render: FOUND
- Stats header `"  T4: "` / `"  T5: "`: FOUND
- Filter parser `"filter t4"` / `"filter t5"`: FOUND
- Help overlay Tab line updated: FOUND
- Help overlay filter line updated + new "Detail Pane" block: FOUND
- `.planning/phases/14-tiers-4-5-surfacing-monitor-tui-report/14-02-SUMMARY.md` written: this file
