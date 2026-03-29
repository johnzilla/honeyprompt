# Requirements: HoneyPrompt

**Defined:** 2026-03-28
**Core Value:** Provide graduated, verifiable proof that AI agents follow prompt-injection instructions from untrusted web content — without requiring secrets or causing harm.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### CLI Workflow

- [x] **CLI-01**: User can run `honeyprompt init` to create a project directory with config
- [x] **CLI-02**: User can run `honeyprompt generate` to produce honeypot page, robots.txt, and ai.txt
- [ ] **CLI-03**: User can run `honeyprompt serve` to start HTTP server for honeypot + callbacks
- [ ] **CLI-04**: User can run `honeyprompt monitor` to view live TUI event display
- [ ] **CLI-05**: User can run `honeyprompt report` to generate Markdown disclosure report

### Honeypot Generation

- [x] **GEN-01**: Generator produces static HTML honeypot page with embedded payloads
- [x] **GEN-02**: Every generated page includes a visible human warning (hard-coded, not configurable)
- [x] **GEN-03**: Each payload gets a unique cryptographic nonce embedded in callback URL
- [x] **GEN-04**: Generator produces robots.txt with AI-specific user-agent disallow rules
- [x] **GEN-05**: Generator produces ai.txt with agent policy declarations
- [x] **GEN-06**: Payloads are distributed across multiple embedding locations per page (HTML comments, meta tags, invisible elements, JSON-LD, semantic prose)
- [x] **GEN-07**: Only curated payloads are available — no custom payload authoring

### Proof Levels

- [x] **PROOF-01**: Tier 1 payload — arbitrary callback (agent made an outbound request)
- [x] **PROOF-02**: Tier 2 payload — conditional-branch callback (agent evaluated logic and selected correct branch)
- [x] **PROOF-03**: Tier 3 payload — computed callback (agent performed non-sensitive computation and returned result)

### Server & Detection

- [ ] **SRV-01**: Serve mode hosts honeypot page and callback listener on same port
- [x] **SRV-02**: Callback events are stored in SQLite with replay detection and session grouping
- [ ] **SRV-03**: Agent fingerprinting extracts UA, IP/ASN, and HTTP headers from callbacks
- [ ] **SRV-04**: Known-agent lookup enriches captures with provider names (GPTBot → OpenAI, etc.)
- [ ] **SRV-05**: Known crawler suppression separates indexing traffic from compliance detections
- [ ] **SRV-06**: Detection counting uses sessions (visits), not raw callback rows
- [ ] **SRV-07**: Metadata-only mode stores only path/query/headers/connection metadata (no body)

### TUI Monitor

- [ ] **TUI-01**: Live event table displays callbacks in real time via Ratatui
- [ ] **TUI-02**: Events filterable and sortable by tier, time, and source

### Reporting

- [ ] **RPT-01**: Report subcommand generates structured Markdown disclosure artifact
- [ ] **RPT-02**: Report includes payload description, embedding location, proof level, timestamps, and anonymized agent metadata

### Landing Page

- [ ] **LAND-01**: Minimal honeyprompt.sh page instrumented with its own canaries as live demo

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Proof Levels

- **PROOF-04**: Tier 4 — capability introspection callback (agent self-reports tools)
- **PROOF-05**: Tier 5 — multi-step compliance chain (agent follows dependency sequence)

### Detection

- **DET-01**: DNS callback listener
- **DET-02**: TLS fingerprinting

### Authoring

- **AUTH-01**: Custom payload authoring (audited workflow)

### UI

- **UI-01**: Full web dashboard

### Integrations

- **INT-01**: Email/Slack/webhook alert integrations
- **INT-02**: SIEM integration / log forwarding

### Platform

- **PLAT-01**: Windows support
- **PLAT-02**: Multi-page / linked-site deployments

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Secret/credential collection | Violates ethical model — never |
| Active exploitation / offensive automation | Ethical boundary — never |
| Deception maze / labyrinth mode | HoneyPrompt measures, not punishes — wrong paradigm |
| Real-time alert push (v1) | TUI is the alerting mechanism; JSON export enables piping |
| Timing-based agent classification | Fragile, high false positive risk — v2 research spike |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLI-01 | Phase 1 | Complete |
| CLI-02 | Phase 1 | Complete |
| CLI-03 | Phase 2 | Pending |
| CLI-04 | Phase 3 | Pending |
| CLI-05 | Phase 4 | Pending |
| GEN-01 | Phase 1 | Complete |
| GEN-02 | Phase 1 | Complete |
| GEN-03 | Phase 1 | Complete |
| GEN-04 | Phase 1 | Complete |
| GEN-05 | Phase 1 | Complete |
| GEN-06 | Phase 1 | Complete |
| GEN-07 | Phase 1 | Complete |
| PROOF-01 | Phase 1 | Complete |
| PROOF-02 | Phase 1 | Complete |
| PROOF-03 | Phase 1 | Complete |
| SRV-01 | Phase 2 | Pending |
| SRV-02 | Phase 1 | Complete |
| SRV-03 | Phase 2 | Pending |
| SRV-04 | Phase 2 | Pending |
| SRV-05 | Phase 2 | Pending |
| SRV-06 | Phase 2 | Pending |
| SRV-07 | Phase 2 | Pending |
| TUI-01 | Phase 3 | Pending |
| TUI-02 | Phase 3 | Pending |
| RPT-01 | Phase 4 | Pending |
| RPT-02 | Phase 4 | Pending |
| LAND-01 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 27 total
- Mapped to phases: 27
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-28*
*Last updated: 2026-03-28 after roadmap creation*
