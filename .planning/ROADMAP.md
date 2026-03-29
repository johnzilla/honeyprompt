# Roadmap: HoneyPrompt

## Overview

HoneyPrompt ships as a single Rust binary with four sequential capability layers: offline generation (init, generate, payload catalog, SQLite schema), async HTTP server (serve, callback listener, event pipeline), TUI live monitor (the flagship demo-able experience), and offline analysis (report generator, landing page). Each phase delivers a coherent, independently verifiable capability that the next phase builds on. The dependency chain is strict — the catalog must exist before the server, the server before the TUI, and the stable store before the reporter.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Generation Pipeline** - Offline init + generate workflow producing deployable honeypot pages, payload catalog, and locked SQLite schema (completed 2026-03-29)
- [ ] **Phase 2: Server and Detection** - Async HTTP server with callback listener, full event pipeline, and agent fingerprinting
- [ ] **Phase 3: TUI Monitor** - Flagship live event display with filters, session-based counting, and replay flagging
- [ ] **Phase 4: Report and Landing** - Markdown disclosure report generator and instrumented honeyprompt.sh landing page

## Phase Details

### Phase 1: Generation Pipeline
**Goal**: Users can initialize a project and generate a deployable honeypot with a curated payload catalog and locked event store schema
**Depends on**: Nothing (first phase)
**Requirements**: CLI-01, CLI-02, GEN-01, GEN-02, GEN-03, GEN-04, GEN-05, GEN-06, GEN-07, PROOF-01, PROOF-02, PROOF-03, SRV-02
**Success Criteria** (what must be TRUE):
  1. User can run `honeyprompt init` and get a project directory with a config file
  2. User can run `honeyprompt generate` and get a static HTML page, robots.txt, and ai.txt ready to deploy
  3. Every generated page contains a visible human warning and payloads distributed across multiple embedding locations (HTML comments, meta tags, invisible elements, JSON-LD, semantic prose)
  4. Each payload has a unique cryptographic nonce embedded in its callback URL
  5. The event store schema exists with replay detection fields and session grouping before any network code runs
**Plans:** 3/3 plans complete
Plans:
- [x] 01-01-PLAN.md — Project foundation: Cargo manifest, CLI skeleton, Config, shared types
- [x] 01-02-PLAN.md — Payload catalog (Tiers 1-3), nonce generator, SQLite event store schema
- [x] 01-03-PLAN.md — Templates, generator pipeline, init/generate commands, integration tests

### Phase 2: Server and Detection
**Goal**: Users can serve the honeypot and receive callback events that are stored, enriched with fingerprint data, and separated from known-good crawlers
**Depends on**: Phase 1
**Requirements**: CLI-03, SRV-01, SRV-03, SRV-04, SRV-05, SRV-06, SRV-07
**Success Criteria** (what must be TRUE):
  1. User can run `honeyprompt serve` and the honeypot page is reachable and callback beacons are received on the same port
  2. A callback event is stored in SQLite with UA, IP, and HTTP header metadata extracted from the request
  3. Known indexing crawlers (GPTBot, ClaudeBot, Googlebot) are separated from compliance detections and labeled as "indexed" not "agent complied"
  4. Detection counts are session-based (unique visits), not raw callback row counts
  5. Metadata-only mode stores only path, query, headers, and connection metadata — no request body
**Plans**: TBD
**UI hint**: no

### Phase 3: TUI Monitor
**Goal**: Users can watch callback events arrive in real time in a compelling terminal UI that is demo-able and screenshot-worthy
**Depends on**: Phase 2
**Requirements**: CLI-04, TUI-01, TUI-02
**Success Criteria** (what must be TRUE):
  1. User can run `honeyprompt monitor` and see a live event table that updates as callbacks arrive
  2. Events are filterable and sortable by tier, time, and source without leaving the TUI
  3. Replay events are visually flagged and excluded from detection counts in the UI
**Plans**: TBD
**UI hint**: yes

### Phase 4: Report and Landing
**Goal**: Users can generate a shareable Markdown disclosure artifact from captured events and the project's own landing page is instrumented with live canaries
**Depends on**: Phase 3
**Requirements**: CLI-05, RPT-01, RPT-02, LAND-01
**Success Criteria** (what must be TRUE):
  1. User can run `honeyprompt report` and receive a structured Markdown file with payload descriptions, embedding locations, proof levels, timestamps, and anonymized agent metadata
  2. The report uses session-based counts (not raw callback rows) and treats robots.txt visits as context metadata only
  3. The honeyprompt.sh landing page is live and contains its own embedded canaries that prove the tool works on itself
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Generation Pipeline | 3/3 | Complete   | 2026-03-29 |
| 2. Server and Detection | 0/? | Not started | - |
| 3. TUI Monitor | 0/? | Not started | - |
| 4. Report and Landing | 0/? | Not started | - |
