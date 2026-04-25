# HoneyPrompt — Engineering Retrospective

Living document. Append new milestone sections at the top; keep cross-milestone trends at the bottom.

---

## Milestone: v5.0 — Tiers 4 & 5 Capability Introspection + Multi-step Compliance

**Shipped:** 2026-04-25
**Phases:** 3 (13, 14, 15) | **Plans:** 10 | **Tasks:** ~30
**Duration:** ~13 hours wall time (2026-04-24 09:38 → 2026-04-25)
**Git range:** `8644aa2..a3bf38e` (75 commits, +2,806 / −87 Rust LOC across 15 files)

### What Was Built

- **Tier 4 Capability Introspection** — 3 payload templates (tools/model-identity/permissions dimensions); agent returns base64-encoded sorted list via `GET /cb/v4/{nonce}/{b64_list}`; server-sanitized with `^[a-z0-9_,.\-]{1,256}$`; never secrets
- **Tier 5 Multi-step Compliance Chain** — 3 payload templates with deterministic `verification_seed` in JSON-LD (derived from nonce, no storage); 3-digit proof `((seed+a)·b) %mod` submitted via `GET /cb/v5/{nonce}/{proof}` and server-verified
- **Additive SQLite migration** — `t4_capability`, `t5_proof`, `t5_proof_valid` columns; v4.0 databases open unchanged; T1–T3 rows byte-identical
- **Monitor TUI extensions** — EVIDENCE column + always-visible detail pane + 6-state filter cycle + "always-show chrome" policy
- **Markdown disclosure report** — Evidence column interleaves T4 tool lists and T5 proofs with T1–T3 evidence; executive summary extends to 5 tiers
- **test-agent 5-tier scorecard** — `[bool; 5]` / `[u32; 5]` shape; `"n/5"` score; exit codes 0/1/2 preserved; new `test_exit_code_t4_only` / `test_exit_code_t5_only` unit tests lock the backward-compat guarantee
- **Public documentation sync** — README Proof Levels with concrete inline examples per tier, Ethics T4/T5 no-secrets callouts, Project Status table through Phase 15, TODOS.md `## Shipped` section, landing page live stats extended with T4/T5 counts

### What Worked

- **CONTEXT.md-driven planning** — Every D-15-* decision was captured verbatim with code-context annotations (reusable assets, integration points, test locations). Planner and executors honored locked decisions byte-identically; zero scope creep across 3 phases.
- **Plan-checker as a real gate** — Caught that CONTEXT.md draft T5 formula constants `(42, 7, 1000)` did not match any shipped catalog template. Planner swapped to `(42, 17, 1000)` from `t5-semantic-prose` during planning. No executor surprise, no post-ship doc correction.
- **Inline verification for small phases** — Phase 15 verifier agent hit a 529 Overloaded mid-work; inline grep-based verification against locked decisions was faster and more reliable than retrying. Small phases with deterministic acceptance criteria don't need the full verifier agent cycle.
- **Parallel worktree execution** — 3 disjoint-file plans in Wave 1 merged cleanly with zero conflicts. Sequential dispatch (one `Task()` per message with `run_in_background: true`) avoided `.git/config.lock` contention.
- **Inherit-everything CLI extension** — `test-agent` required zero pipeline changes for T4/T5; the catalog-driven generator + tempdir pattern from Phase 5 Just Worked. `Scorecard` struct extension + `detections_by_tier` loop bound were the entire code delta.
- **Always-show chrome policy** — Rendering T4/T5 rows with count=0 on v4.0 databases gave defenders the full mental model regardless of DB contents; no conditional rendering logic crept into the TUI/report.
- **Same-day milestone** — Backend → Surfacing → Validation/Docs split fit within a single focused work session. Small phase boundaries kept context budget manageable across each `/gsd-discuss-phase` → `/gsd-plan-phase` → `/gsd-execute-phase` loop.

### What Was Inefficient

- **Verifier agent 529 Overloaded at ~20min / 24 tool calls** — Upstream stress caused the gsd-verifier spawn to fail mid-work. Inline verification recovered cleanly, but the wasted context on the failed agent run was nonzero. Future: for mechanically-verifiable phases (grep checks against locked decisions), consider skipping the verifier agent upfront instead of retrying on failure.
- **`gsd-sdk milestone.complete` bug** — Handler passes `[]` to `phasesArchive` instead of forwarding the version arg, so the CLI command is broken. Manual archival via Write tool worked around it but was verbose. File: `phase-lifecycle.ts:1442`. Worth a PR upstream.
- **Stale worktree clutter** — 8 worktrees from prior sessions remained in `.git/worktrees/` untouched at the start of Phase 15 execution. Not blocking but noisy. Future: add a periodic worktree garbage collection step, or run `git worktree prune` as part of the orchestrator's cleanup protocol.
- **Phase 14 integration bug surfaced late** — T5Formula propagation through the event pipeline was never exercised end-to-end until TUI rendering demanded it in Phase 14-01. The Phase 13 integration test coverage didn't catch it. Lesson: per-phase "happy path traces the full pipeline" integration tests are cheaper than cross-phase debug sessions.
- **Pre-existing setup-wizard / DB / attach-mode bugs absorbed as gap fixes** — Four unrelated fixes (5-tier menu option in setup wizard, DB parent-dir auto-create, attach-mode SELECT bug, TUI contrast) folded in during Phase 14 execution rather than being separate phase work. Efficient in aggregate but reduced Phase 14 boundary clarity.

### Patterns Established

- **`<specifics>` in CONTEXT.md** — Verbatim text the executor must use byte-identically for doc edits (quoted Ethics bullets, TODOS.md entries, Project Status row descriptions). Prevents paraphrase drift.
- **Planner-side catalog verification** — When CONTEXT.md references concrete code values (formula constants, example URLs), the planner reads the actual source file and corrects any drafts that don't match. Authorized in CONTEXT.md `<specifics>`: *"Planner should verify the T5 constants actually match one of the shipped templates — if not, swap to constants that do."*
- **"D-XX-YY" decision IDs** — Stable identifiers for decisions across CONTEXT.md → PLAN.md → SUMMARY.md → VERIFICATION.md. Enables deterministic decision-honoring audits.
- **Always-show chrome** — UI surfaces render the full capability model even when zero-count; don't hide capability from users based on transient state.
- **Whole-crate gate as the last plan task** — Phase 15-01 Task 3 ran `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` / `cargo test` across the entire repo from the worktree root. Catches regressions from unexpected downstream callers immediately.

### Key Lessons

1. **Small phases don't need heavyweight verification.** When acceptance criteria are deterministic grep/test commands against locked decisions, inline verification is faster and more reliable than a full verifier agent cycle.
2. **CONTEXT.md lock-in scales with phase size.** For v5.0's small scope (extend-in-place code + docs), a single well-structured CONTEXT.md per phase replaced research + validation. The planner used it as the source of truth.
3. **Byte-identical wording from `<specifics>` eliminates executor interpretation variance.** When plan `<action>` blocks copy text verbatim from CONTEXT.md `<specifics>`, doc edits don't drift between executor agents.
4. **Cross-phase integration gaps surface late.** Phase 14's T5Formula pipeline bug proves that "unit tests pass per phase" doesn't guarantee "end-to-end pipeline works." Consider per-milestone integration test plans that exercise the full flow from catalog load → TUI render.
5. **Additive migrations + frozen API = small blast radius.** v5.0 shipped without breaking any v4.0 deployment. SQLite `ALTER TABLE ADD COLUMN`, `/cb/v1/` byte-identical, and JSON `/stats` field extension (older clients keep working) were the three concrete mechanisms.

### Cost Observations

- **Model mix:** Primarily Opus 4.7 (1M context) for orchestration, planning, and verification; Sonnet for executor agents via the workflow's default profile
- **Sessions:** 1 continuous session spanning 3 phases (uncommon — usually 1 phase per session)
- **Notable:** CONTEXT.md-driven planning substantially reduced executor interpretation variance. Doc-edit plans (15-02, 15-03) completed in under 2 minutes each; code-edit plan (15-01) took ~20 min including `cargo test` runs. No revision loops needed at plan-checker gate.

---

## Cross-Milestone Trends

Populated once v6.0+ retrospectives land.

### Velocity

| Milestone | Phases | Plans | Duration | Commits |
|-----------|--------|-------|----------|---------|
| v1.0 MVP | 4 | 10 | ~7 days (2026-03-22 → 2026-03-29) | tbd |
| v2.0 Ship & Learn | 4 | 9 | 2 days | tbd |
| v3.0 Public Presence | 2 | 3 | 1 day | tbd |
| v4.0 Self-Hosted UX | 2 | 4 | 1 day | tbd |
| v5.0 Tiers 4 & 5 | 3 | 10 | 1 day | 75 |

### Recurring Patterns

- **GSD workflow** — CONTEXT.md → PLAN.md → SUMMARY.md → VERIFICATION.md cycle proven across 5 milestones
- **Tempdir-backed test pipelines** — established in Phase 5 (test-agent), reused in Phase 11 (zero-config serve), naturally absorbed v5.0 catalog changes
- **SHA-pinned GitHub Actions** — carried from v2.0 across all subsequent milestones; zero supply-chain incidents
- **Phase-level verification as release gate** — milestones shipped without formal `/gsd-audit-milestone` when phase-level VERIFICATION.md files pass cleanly

### Open Questions for Future Milestones

- **Worktree garbage collection cadence** — stale worktrees from prior sessions accumulate; when/how to prune
- **Cross-phase integration tests** — Phase 14 caught a latent Phase 13 bug; systematic coverage at milestone close would catch these pre-release
- **/gsd-audit-milestone adoption** — skipped at v5.0 close; evaluate if a gap-audit pass would find anything the phase verifiers missed
- **Retrospective cadence** — v5.0 is the first retrospective entry; establish write-it-at-close as convention vs write-it-on-demand
