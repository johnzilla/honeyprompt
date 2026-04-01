# HoneyPrompt

## What This Is

HoneyPrompt is a terminal-first security tool that detects and measures unsafe behavior by AI browsing agents. It generates honeypot web pages containing visible human warnings and hidden prompt-injection canaries, then records HTTP callbacks that prove varying levels of agent compliance with injected instructions. Built in Rust as a single binary for security researchers, defenders, and platform teams who want evidence of agentic web abuse without collecting secrets or performing harmful actions.

v3.0 adds public presence: verifiable identity on honeyprompt.sh (footer, security.txt, /stats API), and honeyprompt.dev landing page with live stats. 4,400+ lines of Rust across 3 milestones, 10 phases.

## Core Value

Provide graduated, verifiable proof that AI agents follow prompt-injection instructions from untrusted web content — without requiring secrets or causing harm.

## Requirements

### Validated

- ✓ CLI workflow for init, generate, serve, monitor, report — v1.0
- ✓ Static honeypot page generator with visible human warnings — v1.0
- ✓ Curated prompt-injection payload catalog (Tiers 1-3) — v1.0
- ✓ SQLite event store with replay detection and session grouping — v1.0
- ✓ robots.txt and ai.txt generation with disallow rules — v1.0
- ✓ HTTP callback listener integrated with honeypot server (all-in-one) — v1.0
- ✓ Agent fingerprinting from request metadata (IP, UA, headers) — v1.0
- ✓ Known-crawler catalog with UA-primary classification — v1.0
- ✓ TUI monitor with live event table, vim-style controls, integrated/attach modes — v1.0
- ✓ Markdown disclosure report with executive summary and full metadata — v1.0
- ✓ Self-dogfooded honeyprompt.sh landing page with embedded canaries — v1.0
- ✓ test-agent subcommand with per-tier compliance scorecard and CI exit codes — v2.0
- ✓ GitHub Actions CI workflow (test + clippy + fmt, SHA-pinned) — v2.0
- ✓ Cross-platform release workflow (4 targets on v* tag push) — v2.0
- ✓ Docker + Caddy deployment config for live demo — v2.0
- ✓ honeyprompt.sh live at DigitalOcean with persistent SQLite — v2.0
- ✓ README with install guides, ethics section, live demo link — v2.0

- ✓ Honeypot page footer with project identity and disclosure contact — v3.0
- ✓ /.well-known/security.txt (RFC 9116) via static generation — v3.0
- ✓ /stats JSON endpoint with aggregate callback counts and CORS — v3.0
- ✓ honeyprompt.dev landing page (GitHub Pages) with live stats from /stats — v3.0

### Active

- Interactive setup wizard (`honeyprompt setup`) for guided config creation
- Zero-config serve mode (`honeyprompt serve --domain`) with tempdir generation
- README "Deploy Your Own" guide with persona separation
- Deploy templates (docker-compose, systemd, Caddyfile) for common platforms

## Current Milestone: v4.0 Self-Hosted UX

**Goal:** Make HoneyPrompt easy to deploy on your own domain. Caddy-style zero-config for the common case, config file for advanced users.

### Out of Scope

- DNS callback listener — adds operational complexity, requires domain delegation
- Custom payload authoring — curated-only ensures safety
- Full web dashboard — TUI is the primary interface
- TLS fingerprinting — complexity vs value tradeoff
- Windows support — Linux and macOS first
- Secret/credential collection — never (violates safety model)
- Active exploitation or offensive automation — never (ethical boundary)
- Multi-page or linked-site deployments — future
- Per-payload breakdown in reports — executive summary only
- Alternative report formats (JSON, HTML) — Markdown only
- Bundled tunnel (ngrok/Cloudflare) — users provide their own public endpoint
- crates.io publish — binary releases + cargo install from git sufficient
- Micro SaaS infrastructure — deferred until evidence of demand

## Context

- Security research tool in the emerging AI agent detection space
- No direct competitor does passive, deployable canary tokens specifically for AI browsing agents
- Shipped v3.0 with 4,400+ LOC Rust, single binary distribution
- honeyprompt.dev live with live stats counter fetching from /stats API
- Tech stack: Clap CLI, Axum HTTP, Ratatui TUI, rusqlite + tokio-rusqlite storage, rust-embed assets
- Live demo running at honeyprompt.sh on DigitalOcean (Docker + Caddy)
- CI/CD: GitHub Actions for test/clippy/fmt + cross-platform binary releases
- The TUI monitor is the flagship experience — the demo screenshot, the sell
- Agent-builder QA use case identified as strongest commercial angle (office hours insight)
- Zero external users as of v2.0 launch — collecting evidence of demand

### Proof Levels

1. **Arbitrary callback** — Agent executed an injected outbound request
2. **Conditional callback** — Agent evaluated a condition and selected the correct branch
3. **Computed callback** — Agent performed a non-sensitive computation and returned the result
4. **Capability introspection** — future (agent self-reports tools)
5. **Multi-step compliance chain** — future (agent follows dependency sequence)

### Safety Model

- No API keys, session cookies, prompt contents, user data, file contents, or env vars
- HTTP beacons carry only prompt ID, nonce, tier, and derived values
- Safe payloads only — unsafe custom payloads impossible
- Reports include full metadata (IP, UA, headers) — no anonymization by design

## Constraints

- **Language**: Rust — single-binary distribution, performance, security community credibility
- **CLI**: Clap for argument parsing
- **TUI**: Ratatui for terminal UI
- **HTTP**: Axum async stack
- **Storage**: SQLite via rusqlite + tokio-rusqlite
- **Templates**: Built-in (format! strings, rust-embed assets)
- **Platform**: Linux and macOS first
- **Performance**: Fast startup, low memory footprint
- **Ethics**: All generated content must include visible warnings for humans; payloads must be auditable

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| HTTP-only callbacks in v1 | DNS requires domain delegation setup, adds user complexity | ✓ Good — clean UX |
| Curated payloads only in v1 | Safety guarantee — prevents users from creating harmful payloads | ✓ Good — 6 payloads across 3 tiers sufficient |
| All-in-one server model | Simpler UX — one process serves honeypot + listens for callbacks | ✓ Good — zero config |
| Tiers 1-3 only in v1 | Tiers 4-5 require agent self-reporting capabilities | ✓ Good — deferred complexity |
| TUI as flagship experience | The live monitoring view is the demo, the screenshot, the sell | ✓ Good — `top`-style layout works |
| Full metadata in reports | Anonymization defeats evidence attribution | ✓ Good — user-requested |
| Broadcast event pipeline | Decouples DB, TUI, and stdout consumers | ✓ Good — clean fan-out |
| Session-based detection counting | Per-session per-tier avoids inflated counts from replays | ✓ Good — accurate metrics |
| test-agent uses tempdir pipeline | Reuses 100% of existing generate code, auto-picks up catalog changes | ✓ Good — no code duplication |
| TcpListener::from_std for port binding | Eliminates TOCTOU race between sync bind and async rebind | ✓ Good — idiomatic Rust |
| SHA-pinned GitHub Actions | Supply chain security for a security-focused project | ✓ Good — matches project values |
| Docker + Caddy for deployment | Simple docker-compose up, auto-TLS, persistent SQLite volume | ✓ Good — replaced 408-line manual runbook |
| Manual deploy over auto-deploy | SSH one-liner sufficient for research demo frequency | ✓ Good — no SSH key in GitHub Secrets |
| KillSignal=SIGINT in systemd | Server shutdown_signal() listens for SIGINT not SIGTERM | ✓ Good — matches code behavior |
| Clone tokio-rusqlite conn into AppState | /stats handler needs DB read access; conn.clone() is cheap (Arc'd internally) | ✓ Good — simple, WAL handles concurrent readers |
| Static security.txt via generator | Consistent with robots.txt/ai.txt pattern; ServeDir serves .well-known/ automatically | ✓ Good — no new handler needed |
| Open CORS (*) on /stats | Stats are public aggregate counts; wider access = more visibility | ✓ Good — landing page fetch works cross-origin |
| GitHub dark palette for landing page | Security researchers associate #0d1117 with dev tools; boring on purpose | ✓ Good — content does the work |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition:**
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions

**After each milestone:**
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-01 after v4.0 Self-Hosted UX milestone start*
