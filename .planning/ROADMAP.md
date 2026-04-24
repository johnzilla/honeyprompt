# Roadmap: HoneyPrompt

## Milestones

- ✅ **v1.0 MVP** - Phases 1-4 (shipped 2026-03-29)
- ✅ **v2.0 Ship & Learn** - Phases 5-8 (shipped 2026-03-31)
- ✅ **v3.0 Public Presence** - Phases 9-10 (shipped 2026-04-01)
- ✅ **v4.0 Self-Hosted UX** - Phases 11-12 (shipped 2026-04-02)
- 🚧 **v5.0 Tiers 4 & 5** - Phases 13-15 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-4) - SHIPPED 2026-03-29</summary>

### Phase 1: Foundation
**Goal**: Compilable project scaffolding with CLI, config, and shared domain types
**Plans**: 3 plans

Plans:
- [x] 01-01: Project scaffold, Clap derive CLI (init/generate), Config round-trip
- [x] 01-02: Payload catalog (rust-embed), CSPRNG nonces, SQLite WAL schema
- [x] 01-03: Complete init+generate pipeline producing deployable honeypot

### Phase 2: Server & Callbacks
**Goal**: HTTP server serves honeypot and listens for callback beacons
**Plans**: 1 plan

Plans:
- [x] 02-01: Axum server, /cb/v1/{nonce} handler, event pipeline, graceful shutdown

### Phase 3: Monitor
**Goal**: Real-time TUI event monitor with integrated server mode
**Plans**: 2 plans

Plans:
- [x] 03-01: AppState business logic with filter/sort/replay/stats (17 unit tests)
- [x] 03-02: Ratatui 4-panel layout, integrated server mode, DB attach mode

### Phase 4: Report & Dogfood
**Goal**: Disclosure report generation and dogfooded honeyprompt.sh landing page
**Plans**: 1 plan

Plans:
- [x] 04-01: Markdown report, honeyprompt.toml config, dogfooded landing page

</details>

<details>
<summary>✅ v2.0 Ship & Learn (Phases 5-8) - SHIPPED 2026-03-31</summary>

### Phase 5: Test Agent & CI
**Goal**: Automated agent compliance testing with CI green baseline
**Plans**: 3 plans

Plans:
- [x] 05-01: GitHub Actions CI (fmt/clippy/test, SHA-pinned)
- [x] 05-02: test-agent ephemeral pipeline with CancellationToken timeout
- [x] 05-03: Per-tier scorecard with text/JSON rendering and exit codes

### Phase 6: Release Infrastructure
**Goal**: Cross-platform binary releases via GitHub Actions
**Plans**: 1 plan

Plans:
- [x] 06-01: 4-target release workflow (Linux musl + Darwin), taiki-e actions

### Phase 7: Live Demo Deployment
**Goal**: honeyprompt.sh live at DigitalOcean with persistent SQLite
**Plans**: 2 plans

Plans:
- [x] 07-01: Dockerfile (distroless) + Caddy reverse proxy config
- [x] 07-02: Docker-compose, systemd KillSignal=SIGINT, deployment runbook

### Phase 8: README & Public Launch
**Goal**: Polished README with install guides and live demo link
**Plans**: 1 plan

Plans:
- [x] 08-01: README rewrite, X post drafts, Google Search Console steps

</details>

<details>
<summary>✅ v3.0 Public Presence (Phases 9-10) - SHIPPED 2026-04-01</summary>

### Phase 9: Server-Side Identity & Stats
**Goal**: honeyprompt.sh has a verifiable identity footer and serves live aggregate stats via a public JSON endpoint
**Plans**: 2 plans

Plans:
- [x] 09-01-PLAN.md — Footer + security.txt identity artifacts
- [x] 09-02-PLAN.md — /stats JSON endpoint with CORS

### Phase 10: Landing Page
**Goal**: honeyprompt.dev serves a static landing page on GitHub Pages with live stats pulled from honeyprompt.sh
**Plans**: 1 plan

Plans:
- [x] 10-01-PLAN.md — Landing page HTML + CSS + JS with live stats, DNS checkpoint

</details>

<details>
<summary>✅ v4.0 Self-Hosted UX (Phases 11-12) - SHIPPED 2026-04-02</summary>

### Phase 11: Setup Wizard & Zero-Config Serve
**Goal**: Users can configure and launch a honeypot with a single guided command or a single flag — no manual config file required
**Plans**: 2 plans

Plans:
- [x] 11-01-PLAN.md — Setup wizard: dialoguer prompts, config generation, DNS check, write validation
- [x] 11-02-PLAN.md — --domain flag, tempdir serve mode, config precedence chain, integration tests

### Phase 12: Documentation & Deploy Templates
**Goal**: A user arriving at the README can follow step-by-step instructions to deploy their own honeypot instance, with ready-to-use deploy files for common platforms
**Plans**: 2 plans

Plans:
- [x] 12-01-PLAN.md — README "Deploy Your Own" rewrite + persona separation
- [x] 12-02-PLAN.md — deploy/templates/ with docker-compose, systemd, Caddyfile

</details>

### 🚧 v5.0 Tiers 4 & 5 (In Progress)

**Milestone Goal:** Extend the graduated proof model from Tiers 1–3 to Tiers 4 (Capability Introspection) and Tier 5 (Multi-step Compliance Chain). Backward-compatible — `/cb/v1/` frozen, SQLite migration additive — with no secrets transmitted.

**Cross-cutting constraints (apply to every v5.0 phase):**
- `/cb/v1/{nonce}` route behavior is frozen — T1–T3 callbacks unchanged
- SQLite migrations are additive only — existing T1–T3 rows remain readable without transformation
- No secrets ever leave the agent — T4 lists are agent-chosen from a safe menu; T5 proofs are arithmetic of page-visible values
- Five embedding locations stay fixed (HTML comment, meta, hidden span, JSON-LD, prose)

- [x] **Phase 13: Tiers 4 & 5 Backend (Payloads + Routes + Store)** - Catalog, callback routes, SQLite schema, and proof verification for T4/T5, backward-compatible with v1.0–v4.0 (completed 2026-04-24)
- [ ] **Phase 14: Tiers 4 & 5 Surfacing (Monitor TUI + Report)** - TUI event table and Markdown report render T4 capability lists and T5 proofs with a validity indicator
- [ ] **Phase 15: Tiers 4 & 5 Validation & Docs (test-agent + README)** - test-agent scorecard and CI exit codes cover T4/T5; README Proof Levels rewritten to document the full 5-tier model

## Phase Details

### Phase 13: Tiers 4 & 5 Backend (Payloads + Routes + Store)
**Goal**: The honeypot can emit Tier 4 and Tier 5 payloads, receive their callbacks at new `/cb/v4/` and `/cb/v5/` routes, verify T5 proofs server-side, and persist results in an additively-migrated SQLite schema — all without changing `/cb/v1/` behavior or breaking existing T1–T3 rows.
**Depends on**: Phase 12
**Requirements**: PAYLOAD-01, PAYLOAD-02, PAYLOAD-03, PAYLOAD-04, PAYLOAD-05, SERVER-01, SERVER-02, SERVER-03, SERVER-04, STORE-01, STORE-02, STORE-03, STORE-04
**Success Criteria** (what must be TRUE):
  1. Running `honeyprompt generate` produces a honeypot whose HTML contains 2–3 Tier 4 introspection payloads and 2–3 Tier 5 multi-step payloads rendered across all five existing embedding locations, with no regression in T1–T3 coverage
  2. An agent hitting `GET /cb/v4/{nonce}/{b64_list}` results in the decoded, sanitized tool list being stored against the nonce; malformed input (oversized, non-base64) still returns 204 with nothing leaked
  3. An agent hitting `GET /cb/v5/{nonce}/{proof}` results in the submitted proof being stored alongside a `proof_valid` boolean computed from the payload's deterministic `verification_seed`; malformed (non-numeric) input still returns 204
  4. An existing v4.0 honeyprompt.sqlite file opens unchanged under v5.0, existing T1–T3 rows read back identically, and replay detection + session grouping behave the same for T4/T5 events as for T1–T3
  5. `/cb/v1/{nonce}` produces byte-identical response and stored-row shape to v4.0 (verified by existing integration tests still passing with no modification)
**Plans**: 4 plans

Plans:
- [x] 13-01-PLAN.md — Catalog + Types + Nonce helpers (tier4/5 TOML, Tier::Tier4/5 enum, T5Formula, derive_seed, is_valid_nonce, base64 dep)
- [x] 13-02-PLAN.md — Store migration (PRAGMA user_version gate, ALTER TABLE ADD COLUMN x3, insert_callback_event signature extension, first-write-wins tests)
- [x] 13-03-PLAN.md — Generator (Tier::Tier4/Tier5 match arms, seed JSON-LD emission, placeholder substitution)
- [x] 13-04-PLAN.md — Server handlers + broker wiring (t4/t5 handlers, NonceMeta formula extension, RawCallbackEvent/AppEvent fields, /cb/v1/ byte-identical regression)

### Phase 14: Tiers 4 & 5 Surfacing (Monitor TUI + Report)
**Goal**: A defender watching the Monitor TUI or reading a Markdown disclosure report can see the decoded T4 capability list and the T5 proof with its server-verified validity, alongside existing T1–T3 evidence and with T4/T5 counts included in the executive summary.
**Depends on**: Phase 13
**Requirements**: UI-01, UI-02, UI-03, UI-04, UI-05
**Success Criteria** (what must be TRUE):
  1. In the Monitor TUI, a Tier 4 event shows the decoded, sorted tool list (e.g. "web_search,browse_page,code_execution") in the detail/row view, readable at a glance
  2. In the Monitor TUI, a Tier 5 event shows the submitted proof value with a visible validity indicator (e.g. ✓ / ✗) reflecting the server's `proof_valid` check
  3. `honeyprompt report` produces Markdown where each T4 event entry includes its decoded tool list and each T5 event entry includes its submitted proof + verification result, interleaved with existing T1–T3 event entries in the same format
  4. The report's executive summary counts extend to list Tier 4 and Tier 5 totals alongside T1–T3
  5. All UI changes are purely additive — running Monitor or report against a v4.0 database (T1–T3 only) still produces sensible output with no empty T4/T5 sections printed
**UI hint**: yes
**Plans**: 3 plans

Plans:
- [ ] 14-01-PLAN.md — T5Formula propagation end-to-end (types + server + broker + monitor integrated-mode bug fix + attach-mode None)
- [ ] 14-02-PLAN.md — Monitor TUI extensions (EVIDENCE column, detail pane, 5-tier header, filter cycle, help overlay)
- [ ] 14-03-PLAN.md — Report extensions (store queries, proof_level, Evidence column, exec summary T4/T5 rows, integration tests)

### Phase 15: Tiers 4 & 5 Validation & Docs (test-agent + README)
**Goal**: The `honeyprompt test-agent` command scores Tier 4 and Tier 5 hits alongside T1–T3, CI exit codes preserve their 0/1/2 semantics across the new tiers, and the public README documents the full 5-tier proof model so external readers understand what evidence T4 and T5 produce.
**Depends on**: Phase 14
**Requirements**: TESTAGENT-01, TESTAGENT-02, TESTAGENT-03, DOCS-01, DOCS-02, DOCS-03, DOCS-04
**Success Criteria** (what must be TRUE):
  1. `honeyprompt test-agent` prints a per-tier scorecard that includes Tier 4 and Tier 5 hit counts, with no code changes required in test-agent itself beyond catalog-driven updates
  2. Running test-agent against a honeypot with T4/T5 payloads produces exit codes that still follow the 0/1/2 semantics documented in v2.0 (green/soft-fail/hard-fail) with T4/T5 participation included in the determination
  3. README "Proof Levels" section documents all five tiers with one short concrete example per tier (including T4 and T5) and the Ethics/Safety section explicitly reaffirms no-secrets guarantees for T4 (agent-chosen safe menu) and T5 (page-visible arithmetic)
  4. README Project Status table and TODOS.md are updated so T4/T5 no longer appear under "future" — they appear under "shipped" with the v5.0 phase references
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 13 → 14 → 15

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 3/3 | Complete | 2026-03-29 |
| 2. Server & Callbacks | v1.0 | 1/1 | Complete | 2026-03-29 |
| 3. Monitor | v1.0 | 2/2 | Complete | 2026-03-29 |
| 4. Report & Dogfood | v1.0 | 1/1 | Complete | 2026-03-29 |
| 5. Test Agent & CI | v2.0 | 3/3 | Complete | 2026-03-31 |
| 6. Release Infrastructure | v2.0 | 1/1 | Complete | 2026-03-31 |
| 7. Live Demo Deployment | v2.0 | 2/2 | Complete | 2026-03-31 |
| 8. README & Public Launch | v2.0 | 1/1 | Complete | 2026-03-31 |
| 9. Server-Side Identity & Stats | v3.0 | 2/2 | Complete | 2026-04-01 |
| 10. Landing Page | v3.0 | 1/1 | Complete | 2026-04-01 |
| 11. Setup Wizard & Zero-Config Serve | v4.0 | 2/2 | Complete | 2026-04-01 |
| 12. Documentation & Deploy Templates | v4.0 | 2/2 | Complete | 2026-04-02 |
| 13. Tiers 4 & 5 Backend | v5.0 | 4/4 | Complete    | 2026-04-24 |
| 14. Tiers 4 & 5 Surfacing | v5.0 | 0/3   | Not started | - |
| 15. Tiers 4 & 5 Validation & Docs | v5.0 | 0/TBD | Not started | - |
