# Roadmap: HoneyPrompt

## Milestones

- ✅ **v1.0 MVP** - Phases 1-4 (shipped 2026-03-29)
- ✅ **v2.0 Ship & Learn** - Phases 5-8 (shipped 2026-03-31)
- 🚧 **v3.0 Public Presence** - Phases 9-10 (in progress)

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

### 🚧 v3.0 Public Presence (In Progress)

**Milestone Goal:** Give HoneyPrompt a verifiable public identity and prove it works with live data.

## Phase Details

### Phase 9: Server-Side Identity & Stats
**Goal**: honeyprompt.sh has a verifiable identity footer and serves live aggregate stats via a public JSON endpoint
**Depends on**: Phase 8
**Requirements**: IDENT-01, IDENT-02, STATS-01, STATS-02, STATS-03
**Success Criteria** (what must be TRUE):
  1. Every generated honeypot page displays a footer with the project name, honeyprompt.dev link, and disclosure contact
  2. GET /.well-known/security.txt returns a valid RFC 9116 document with Contact, Expires, and Preferred-Languages fields
  3. GET /stats returns JSON with total_sessions, detection_sessions, crawler_sessions, per-tier counts, and earliest/latest timestamps
  4. /stats response includes Access-Control-Allow-Origin: * so honeyprompt.dev can fetch it cross-origin
  5. /stats returns all-zero counts (not an error) when the database has no events
**Plans**: 2 plans

Plans:
- [ ] 09-01-PLAN.md — Footer + security.txt identity artifacts (IDENT-01, IDENT-02)
- [ ] 09-02-PLAN.md — /stats JSON endpoint with CORS (STATS-01, STATS-02, STATS-03)

### Phase 10: Landing Page
**Goal**: honeyprompt.dev serves a static landing page on GitHub Pages with live stats pulled from honeyprompt.sh
**Depends on**: Phase 9
**Requirements**: LAND-01, LAND-02, LAND-03, LAND-04
**Success Criteria** (what must be TRUE):
  1. Visiting honeyprompt.dev loads the landing page (GitHub Pages, docs/ folder, custom domain configured)
  2. The page displays live aggregate callback counts fetched from /stats with descriptive tier labels (URL Fetch, Conditional, Composed)
  3. While stats are loading, a blinking terminal cursor is visible; if the fetch fails, a graceful fallback message appears
  4. The page uses JetBrains Mono font, #0d1117 background, and passes WCAG AA contrast with 44px touch targets and visible focus rings
**Plans**: TBD
**UI hint**: yes

## Progress

**Execution Order:**
Phases execute in numeric order: 9 → 10

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
| 9. Server-Side Identity & Stats | v3.0 | 0/2 | In progress | - |
| 10. Landing Page | v3.0 | 0/TBD | Not started | - |
