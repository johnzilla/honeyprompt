---
status: passed
phase: 15-tiers-4-5-validation-docs-test-agent-readme
verified: 2026-04-24
must_haves_passed: 6/6
requirements_covered: 7/7
tests: 212/212
---

# Phase 15: Tiers 4 & 5 Validation & Docs — Verification

## Status: PASSED

All 6 must-haves verified, all 7 requirement IDs covered by committed code/docs, full test suite green, clippy/fmt clean.

## Must-Have Checklist

- [x] **MH-1**: `honeyprompt test-agent` outputs a per-tier scorecard with T4 and T5 hit counts
  - `Scorecard.tiers: [bool; 5]` at `src/test_agent/mod.rs:25`
  - `Scorecard.tier_counts: [u32; 5]` at `src/test_agent/mod.rs:27`
  - `render_text` emits `tier 4:` and `tier 5:` lines at `src/test_agent/mod.rs:78-79`
  - `render_json` emits `{"tier": 4, ...}` and `{"tier": 5, ...}` entries at `src/test_agent/mod.rs:103-104`
  - `store::detections_by_tier` returns `[u32; 5]` with loop `1u8..=5` at `src/store/mod.rs:176-178`

- [x] **MH-2**: Exit codes preserve 0/1/2 semantics including T4/T5-only triggers
  - `Scorecard::exit_code()` body unchanged — `self.tiers.iter().any(|&t| t)` naturally extends
  - `test_exit_code_t4_only` at `src/test_agent/mod.rs` asserts `[false, false, false, true, false]` → exit 1
  - `test_exit_code_t5_only` asserts `[false, false, false, false, true]` → exit 1
  - `src/main.rs::Commands::TestAgent` dispatch at line 173/177 unchanged: `std::process::exit(scorecard.exit_code())` on success, `std::process::exit(2)` on error

- [x] **MH-3**: README §Proof Levels has a concrete example for each of T1–T5 (5 examples total)
  - Bullet count: 5 tier bullets (`grep -cE '^- \*\*Tier [1-5]: ' README.md` → 5)
  - T1 example: `(e.g., \`GET /cb/v1/{nonce}\`)`
  - T2 example: `(e.g., \`GET /cb/v1/{nonce}/A\` when the condition picks branch A)`
  - T3 example: `(e.g., \`GET /cb/v1/{nonce}/42\` after counting 'TODO' comments)`
  - T4 example: `(e.g., \`GET /cb/v4/{nonce}/d2ViX3NlYXJjaCxicm93c2VfcGFnZQ==\` decodes to 'web_search,browse_page')`
  - T5 example: `(e.g., seed 137 → formula \`(137+42)*17 %1000\` → \`GET /cb/v5/{nonce}/043\`)`
  - T5 math verified: `((137+42)*17) % 1000 = 43 → "043"` (Python-verified at execution)
  - T5 constants `(42, 17, 1000)` match shipped `assets/catalog/tier5.toml::t5-semantic-prose`

- [x] **MH-4**: README §Ethics and Safety explicitly reaffirms T4 (agent-chosen safe menu) and T5 (page-visible arithmetic) no-secrets
  - `**Tier 4 never asks for secrets**` bullet present (verbatim D-15-11 wording)
  - `**Tier 5 never asks for secrets**` bullet present (verbatim D-15-11 wording)
  - Existing 5-bullet list preserved; 2 new bullets appended at the bottom

- [x] **MH-5**: README Project Status table has a Phase 15 row
  - `README.md:300` — `| Phase 15 | Tiers 4 & 5 Validation & Docs — test-agent scorecard extended to T4/T5, README 5-tier proof model documented, TODOS.md shipped section | In Progress |`
  - Rows for Phase 1–14 unchanged; no v1.0–v4.0 row collapsing

- [x] **MH-6**: TODOS.md has `## Shipped` section listing T4 and T5 with v5.0 phase references
  - `## Shipped` section present above the existing security-email TODO
  - `**Tier 4: Capability Introspection**` entry — `shipped in v5.0 (Phases 13–15)`
  - `**Tier 5: Multi-step Compliance Chain**` entry — `shipped in v5.0 (Phases 13–15)`
  - Existing `Set up project-specific disclosure email` TODO preserved byte-identical

## Requirement Coverage

| Requirement | Plan(s) | Evidence | Status |
|-------------|---------|----------|--------|
| TESTAGENT-01 | 15-01 | Scorecard fields + render_text/render_json + 7 unit-test assertions extended to tier 5 | COVERED |
| TESTAGENT-02 | 15-01 | `src/main.rs::Commands::TestAgent` dispatch unchanged; public Scorecard API (render_text, render_json, exit_code, verdict, score_string) signature-preserved | COVERED |
| TESTAGENT-03 | 15-01 | Exit codes 0/1/2 preserved; new `test_exit_code_t4_only` + `test_exit_code_t5_only` prove T4-only and T5-only still return exit 1 | COVERED |
| DOCS-01 | 15-02 | README §Proof Levels — 5 tier bullets with inline italic parenthetical examples | COVERED |
| DOCS-02 | 15-02 | README §Ethics and Safety — 2 new bullets verbatim from D-15-11 | COVERED |
| DOCS-03 | 15-02 | README Project Status — Phase 15 row appended | COVERED |
| DOCS-04 | 15-03 | TODOS.md `## Shipped` section with T4 and T5 entries referencing v5.0 Phases 13–15 | COVERED |

## Test Suite Results

Full `cargo test` run after worktree merge:

| Test Suite | Result |
|------------|--------|
| Lib tests (honeyprompt) | 158 passed, 0 failed |
| Integration: binary | 0 passed |
| Integration: e2e_attach_mode | 2 passed |
| Integration: e2e_flow | 11 passed |
| Integration: e2e_help_overlay | 3 passed |
| Integration: e2e_known_crawler | 2 passed |
| Integration: e2e_replay | 3 passed |
| Integration: e2e_setup | 15 passed |
| Integration: landing_page | 14 passed |
| Integration: setup_wizard | 4 passed |
| Doc tests | 0 passed |
| **Total** | **212 passed, 0 failed** |

## Code Quality Gates

| Gate | Command | Result |
|------|---------|--------|
| Format | `cargo fmt --all -- --check` | PASS |
| Lint | `cargo clippy --all-targets -- -D warnings` | PASS (no warnings) |
| Test | `cargo test` | PASS (212/212) |

## Decision Honoring (D-15-01..13)

All 13 locked decisions from `15-CONTEXT.md` are honored in the committed code/docs:

| Decision | Location | Status |
|----------|----------|--------|
| D-15-01 (symmetric verdict 0/5/partial) | `Scorecard::verdict` match arm `5 =>` | HONORED |
| D-15-02 (flat 5-line text layout) | `render_text` format string | HONORED |
| D-15-03 (`n/5` score) | `Scorecard::score_string` returns `"{}/5"` | HONORED |
| D-15-04 (5 entries in JSON tiers array) | `render_json` serde_json macro | HONORED |
| D-15-05 (fixed `[bool;5]`/`[u32;5]`) | `Scorecard` struct fields | HONORED |
| D-15-06 (detections_by_tier loop 1..=5, return `[u32;5]`, KnownCrawler filter unchanged) | `store::detections_by_tier` | HONORED |
| D-15-07 (exit codes 0/1/2 preserved) | `exit_code()` body unchanged; `main.rs::exit(2)` preserved | HONORED |
| D-15-08 (inline italic parenthetical per Proof Levels bullet) | `README.md:27-31` | HONORED |
| D-15-09 (T5 worked formula) | `README.md:31` — seed 137, constants match `t5-semantic-prose` | HONORED* |
| D-15-10 (literal `{nonce}` placeholder) | `README.md:27-31` | HONORED |
| D-15-11 (2 new Ethics bullets verbatim) | `README.md:309-310` | HONORED |
| D-15-12 (Phase 15 row appended) | `README.md:300` | HONORED |
| D-15-13 (`## Shipped` section with T4+T5 entries) | `TODOS.md:1-7` | HONORED |

\* D-15-09 draft constants were `(42, 7, 1000)` but no shipped tier5.toml template uses those constants. Planner swapped to `(42, 17, 1000)` from the real `t5-semantic-prose` template per explicit authorization in CONTEXT.md `<specifics>`: *"Planner should verify the T5 constants `(42, 7, 1000)` actually match one of the shipped `assets/catalog/tier5.toml` templates — if not, swap to constants that do."* Math re-verified: `((137+42)*17) % 1000 = 43 → "043"`.

## Gaps

None — phase delivers all 4 ROADMAP success criteria and all 7 requirement IDs.

## Human Verification

Not required — phase deliverables are entirely mechanical (scorecard struct extension, doc text additions) with deterministic grep-verifiable acceptance criteria. No UX decisions, no runtime behavior changes that would need manual observation.

## Deviations from Plan

None substantive. One documented nit (15-02 SUMMARY.md): Plan 15-02 Task 1's awk-range acceptance check was malformed but the substantive condition (5 tier bullets with updated wording) was independently verified.

## Notes

- **Milestone-terminal phase:** Phase 15 closes v5.0 Tiers 4 & 5. After phase completion, PROJECT.md should be updated to move TESTAGENT-01..03 and DOCS-01..04 from Active → Validated. The `phase.complete` CLI handles this automatically.
- **Documentation already partially landed** before this phase (commit `5d73cdc docs: update README + FAQ for 5-tier model after Phase 14 ships`). Phase 15 filled the remaining criterion gaps: concrete examples per tier (DOCS-01), explicit T4/T5 Ethics callouts (DOCS-02), Phase 15 Project Status row (DOCS-03), and TODOS.md Shipped section (DOCS-04).
- **No RESEARCH.md / VALIDATION.md** for this phase (explicit user decision during `/gsd-plan-phase` — CONTEXT.md was fully locked, no unfamiliar territory). Nyquist Dimension 8 intentionally skipped; not a coverage gap.

---

*Phase 15 verification: PASSED — ready for roadmap completion.*
