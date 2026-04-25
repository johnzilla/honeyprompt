# Roadmap: HoneyPrompt

## Milestones

- ✅ **v1.0 MVP** - Phases 1-4 (shipped 2026-03-29)
- ✅ **v2.0 Ship & Learn** - Phases 5-8 (shipped 2026-03-31)
- ✅ **v3.0 Public Presence** - Phases 9-10 (shipped 2026-04-01)
- ✅ **v4.0 Self-Hosted UX** - Phases 11-12 (shipped 2026-04-02)
- ✅ **v5.0 Tiers 4 & 5** - Phases 13-15 (shipped 2026-04-25)

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

<details>
<summary>✅ v5.0 Tiers 4 & 5 (Phases 13-15) - SHIPPED 2026-04-25</summary>

### Phase 13: Tiers 4 & 5 Backend (Payloads + Routes + Store)
**Goal**: Honeypot emits T4/T5 payloads, receives callbacks at `/cb/v4/` and `/cb/v5/` routes, verifies T5 proofs server-side, and persists results in an additively-migrated SQLite schema — `/cb/v1/` behavior unchanged.
**Plans**: 4 plans

Plans:
- [x] 13-01-PLAN.md — Catalog + Types + Nonce helpers (tier4/5 TOML, Tier::Tier4/5 enum, T5Formula, derive_seed, is_valid_nonce, base64 dep)
- [x] 13-02-PLAN.md — Store migration (PRAGMA user_version gate, ALTER TABLE ADD COLUMN x3, insert_callback_event signature extension, first-write-wins tests)
- [x] 13-03-PLAN.md — Generator (Tier::Tier4/Tier5 match arms, seed JSON-LD emission, placeholder substitution)
- [x] 13-04-PLAN.md — Server handlers + broker wiring (t4/t5 handlers, NonceMeta formula extension, RawCallbackEvent/AppEvent fields, /cb/v1/ byte-identical regression)

### Phase 14: Tiers 4 & 5 Surfacing (Monitor TUI + Report)
**Goal**: Monitor TUI and Markdown disclosure report show T4 capability lists and T5 proofs with server-verified validity alongside T1–T3, with T4/T5 counts in the executive summary.
**Plans**: 3 plans

Plans:
- [x] 14-01-PLAN.md — T5Formula propagation end-to-end (types + server + broker + monitor integrated-mode bug fix + attach-mode None)
- [x] 14-02-PLAN.md — Monitor TUI extensions (EVIDENCE column, detail pane, 5-tier header, filter cycle, help overlay)
- [x] 14-03-PLAN.md — Report extensions (store queries, proof_level, Evidence column, exec summary T4/T5 rows, integration tests)

### Phase 15: Tiers 4 & 5 Validation & Docs (test-agent + README)
**Goal**: `honeyprompt test-agent` scores T4/T5 alongside T1–T3 with 0/1/2 exit-code semantics preserved; README documents the full 5-tier proof model.
**Plans**: 3 plans

Plans:
- [x] 15-01-PLAN.md — test-agent scorecard 5-tier extension (Scorecard struct + render_text/render_json + store::detections_by_tier → [u32; 5])
- [x] 15-02-PLAN.md — README 5-tier documentation (Proof Levels italic examples + Ethics T4/T5 no-secrets bullets + Project Status Phase 15 row)
- [x] 15-03-PLAN.md — TODOS.md ## Shipped section (T4 + T5 entries above existing security-email TODO)

**Full details:** see `.planning/milestones/v5.0-ROADMAP.md` for success criteria, cross-cutting constraints, key decisions, deferred items, and frozen protocol surface.

</details>

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
| 14. Tiers 4 & 5 Surfacing | v5.0 | 3/3 | Complete    | 2026-04-24 |
| 15. Tiers 4 & 5 Validation & Docs | v5.0 | 3/3 | Complete    | 2026-04-25 |
