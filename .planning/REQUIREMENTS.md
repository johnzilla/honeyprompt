# Requirements: HoneyPrompt

**Defined:** 2026-03-29
**Core Value:** Provide graduated, verifiable proof that AI agents follow prompt-injection instructions from untrusted web content — without requiring secrets or causing harm.

## v2.0 Requirements

Requirements for Ship & Learn milestone. Each maps to roadmap phases.

### test-agent

- [ ] **TEST-01**: User can run `honeyprompt test-agent` to spin up a temporary honeypot server that auto-shuts-down after a configurable timeout
- [ ] **TEST-02**: User can specify listen address, timeout duration, and output format via `--listen`, `--timeout`, `--format` flags
- [ ] **TEST-03**: User sees a per-tier (1/2/3) pass/fail compliance scorecard after the test completes
- [ ] **TEST-04**: Process exits with code 0 (no canaries triggered), 1 (one or more triggered), or 2 (error/no data)
- [ ] **TEST-05**: User can get JSON-formatted output via `--format json` for CI pipeline integration

### Release Infrastructure

- [x] **REL-01**: Every push and PR triggers CI that runs `cargo test`, `cargo clippy`, and `cargo fmt --check`
- [ ] **REL-02**: Pushing a `v*` tag triggers a release workflow that builds cross-platform binaries (x86_64-linux, aarch64-linux, x86_64-darwin, aarch64-darwin) and uploads them to GitHub Releases
- [ ] **REL-03**: README includes `cargo install honeyprompt` and prebuilt binary download instructions

### Deployment

- [ ] **DEPLOY-01**: Repository includes deployment configuration (Dockerfile or systemd unit) for running `honeyprompt serve` as a persistent process
- [ ] **DEPLOY-02**: honeyprompt.sh domain serves a live honeypot with canary payloads over HTTPS
- [ ] **DEPLOY-03**: Live demo has uptime monitoring and process auto-restart

### Launch

- [ ] **LAUNCH-01**: README rewritten with clear value proposition, quick-start for test-agent, live demo link, and installation instructions
- [ ] **LAUNCH-02**: honeyprompt.sh submitted to Google Search Console and linked from README for crawler discoverability
- [ ] **LAUNCH-03**: Ethics/scope section in README explaining what HoneyPrompt does and does not do

## Future Requirements

Deferred to future milestones. Tracked but not in current roadmap.

### Agent Compliance Platform (v3.0+)

- **PLAT-01**: User can deploy a hosted honeypot via `honeyprompt cloud` (one-click Fly.io deployment)
- **PLAT-02**: Public dashboard showing aggregate agent compliance scores across all deployments
- **PLAT-03**: CI-friendly compliance runner with structured JSON reports mapping to OWASP LLM Top 10

### Payload Expansion (v3.0+)

- **PAY-01**: Tier 4 capability introspection payloads (agent self-reports tools)
- **PAY-02**: Tier 5 multi-step compliance chain payloads (agent follows dependency sequence)
- **PAY-03**: Diversified payload instruction text across embedding locations

## Out of Scope

| Feature | Reason |
|---------|--------|
| DNS callback listener | Adds operational complexity, requires domain delegation |
| Custom payload authoring | Curated-only ensures safety |
| Full web dashboard | TUI is the primary interface for v2 |
| TLS fingerprinting | Complexity vs value tradeoff |
| Windows support | Linux and macOS first |
| Secret/credential collection | Violates safety model — never |
| Active exploitation | Ethical boundary — never |
| Bundled tunnel (ngrok/Cloudflare) | Users provide their own public endpoint for remote agents |
| crates.io publish | Deferred — binary releases + cargo install from git sufficient for v2 |
| Micro SaaS infrastructure | Deferred until evidence of demand (10+ self-deploy users or 50+ demo engagement) |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| TEST-01 | Phase 5 | Pending |
| TEST-02 | Phase 5 | Pending |
| TEST-03 | Phase 5 | Pending |
| TEST-04 | Phase 5 | Pending |
| TEST-05 | Phase 5 | Pending |
| REL-01 | Phase 5 | Complete |
| REL-02 | Phase 6 | Pending |
| REL-03 | Phase 6 | Pending |
| DEPLOY-01 | Phase 7 | Pending |
| DEPLOY-02 | Phase 7 | Pending |
| DEPLOY-03 | Phase 7 | Pending |
| LAUNCH-01 | Phase 8 | Pending |
| LAUNCH-02 | Phase 8 | Pending |
| LAUNCH-03 | Phase 8 | Pending |

**Coverage:**
- v2.0 requirements: 14 total
- Mapped to phases: 14
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-29*
*Last updated: 2026-03-29 after roadmap creation*
