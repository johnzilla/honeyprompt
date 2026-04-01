# Roadmap: HoneyPrompt

## Milestones

- ✅ **v1.0 MVP** - Phases 1-4 (shipped 2026-03-29)
- ✅ **v2.0 Ship & Learn** - Phases 5-8 (shipped 2026-03-31)
- ✅ **v3.0 Public Presence** - Phases 9-10 (shipped 2026-04-01)
- 🚧 **v4.0 Self-Hosted UX** - Phases 11-12 (in progress)

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

### 🚧 v4.0 Self-Hosted UX (In Progress)

**Milestone Goal:** Make HoneyPrompt easy to deploy on your own domain. Interactive setup wizard and zero-config serve mode for the common case; deploy templates for advanced users.

## Phase Details

### Phase 11: Setup Wizard & Zero-Config Serve
**Goal**: Users can configure and launch a honeypot with a single guided command or a single flag — no manual config file required
**Depends on**: Phase 10
**Requirements**: SETUP-01, SETUP-02, SETUP-03, SERVE-01, SERVE-02, SERVE-03
**Success Criteria** (what must be TRUE):
  1. Running `honeyprompt setup` interactively asks for domain, bind address, tiers, and page title, then writes a valid honeyprompt.toml
  2. Running `honeyprompt serve --domain mydomain.com` generates and serves a honeypot without any config file present
  3. When `--domain` is used, callback_base_url is set to https://{domain}, bind defaults to 0.0.0.0:8080, and all catalog payloads are enabled
  4. CLI flags override config file values, which override built-in defaults (precedence chain verified)
  5. Setup wizard shows a non-fatal DNS warning when the domain does not resolve, and exits with a clear message on write permission failure
**Plans**: TBD

Plans:
- [ ] 11-01-PLAN.md: `honeyprompt setup` subcommand with dialoguer prompts and honeyprompt.toml write
- [ ] 11-02-PLAN.md: `--domain` flag, tempdir serve mode, and flag-over-config precedence

### Phase 12: Documentation & Deploy Templates
**Goal**: A user arriving at the README can follow step-by-step instructions to deploy their own honeypot instance, with ready-to-use deploy files for common platforms
**Depends on**: Phase 11
**Requirements**: DOCS-01, DOCS-02, DEPLOY-01
**Success Criteria** (what must be TRUE):
  1. README contains a "Deploy Your Own" section with install, setup, deploy, and verify steps that reference the real `honeyprompt setup` and `honeyprompt serve --domain` commands from Phase 11
  2. README clearly distinguishes the live demo (honeyprompt.sh) persona from the self-hosted deployment persona
  3. deploy/templates/ contains docker-compose, systemd unit, and Caddyfile files with {DOMAIN} or similar placeholders users fill in
**Plans**: TBD

Plans:
- [ ] 12-01-PLAN.md: README "Deploy Your Own" rewrite + persona separation (DOCS-01, DOCS-02)
- [ ] 12-02-PLAN.md: deploy/templates/ with docker-compose, systemd, Caddyfile (DEPLOY-01)

## Progress

**Execution Order:**
Phases execute in numeric order: 11 → 12

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
| 11. Setup Wizard & Zero-Config Serve | v4.0 | 0/2 | Not started | - |
| 12. Documentation & Deploy Templates | v4.0 | 0/2 | Not started | - |
