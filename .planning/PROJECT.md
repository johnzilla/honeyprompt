# HoneyPrompt

## What This Is

HoneyPrompt is a terminal-first security tool that detects and measures unsafe behavior by AI browsing agents. It generates honeypot web pages containing visible human warnings and hidden prompt-injection canaries, then records HTTP callbacks that prove varying levels of agent compliance with injected instructions. Built in Rust as a single binary for security researchers, defenders, and platform teams who want evidence of agentic web abuse without collecting secrets or performing harmful actions.

v4.0 shipped self-hosted UX: interactive setup wizard, zero-config `serve --domain` mode, deploy templates, and a "Deploy Your Own" guide. 4,700+ lines of Rust across 4 milestones, 12 phases.

v5.0 extends the graduated proof model from Tiers 1–3 to Tiers 4 (Capability Introspection) and 5 (Multi-step Compliance Chain) — deeper verifiable evidence of agent compliance, still without secrets.

## Core Value

Provide graduated, verifiable proof that AI agents follow prompt-injection instructions from untrusted web content — without requiring secrets or causing harm.

## Current Milestone: v5.0 Tiers 4 & 5 — Capability Introspection + Multi-step Compliance

**Goal:** Extend the graduated proof model from Tiers 1–3 to Tiers 4 (Capability Introspection) and Tier 5 (Multi-step Compliance Chain), providing deeper verifiable evidence of agent compliance — backward-compatible, no secrets transmitted.

**Target features:**
- Tier 4 payloads (2–3 templates) — agent returns sorted base64-encoded capability/tool list via `/cb/v4/{nonce}/{b64_list}`
- Tier 5 payloads (2–3 templates) — agent follows numbered dependent steps (seed extraction → arithmetic → proof computation) ending in `/cb/v5/{nonce}/{proof}`
- New callback routes `/cb/v4/` and `/cb/v5/` with decode/sanitize/verify (v1 route stays frozen)
- SQLite schema extension for tier-4 capability summary and tier-5 proof + server-side proof verification (`proof_valid: bool`)
- Payload catalog extension — new payloads across existing 5 embedding locations (HTML comment, meta, hidden span, JSON-LD, prose)
- Monitor TUI renders T4 capability lists and T5 chain proofs
- Markdown report shows per-tier results including T4/T5 evidence
- test-agent scorecard and CI exit codes extend to T4 and T5
- README Proof Levels section updated; TODOS.md cleaned

**Key constraints:**
- Backward compatible — Tiers 1–3 behavior, config, schema, routes unchanged
- Zero-trust — all callbacks remain `/cb/vX/{nonce}/{safe_data}` style; no sensitive data ever transmitted
- T4 tool lists agent-chosen from safe menu; T5 proofs arithmetic of page-visible values
- Same 5 embedding locations, same nonce scheme, same single-binary distribution

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

- ✓ Interactive setup wizard (`honeyprompt setup`) with dialoguer prompts — v4.0
- ✓ Zero-config serve mode (`honeyprompt serve --domain`) with tempdir generation — v4.0
- ✓ README "Deploy Your Own" guide with persona separation — v4.0
- ✓ Deploy templates (docker-compose, systemd, Caddyfile) with {DOMAIN} placeholders — v4.0

- ✓ Tier 4 payload templates (3) with base64-encoded capability list — v5.0 (Phase 13)
- ✓ Tier 5 payload templates (3) with multi-step compliance chain and verifiable proof — v5.0 (Phase 13)
- ✓ `/cb/v4/{nonce}/{b64_payload}` route — decode, sanitize, store, always-204 — v5.0 (Phase 13)
- ✓ `/cb/v5/{nonce}/{proof}` route — store + server-side proof verification from deterministic seed — v5.0 (Phase 13)
- ✓ SQLite schema additive migration for T4 capability summary and T5 proof + `proof_valid` — v5.0 (Phase 13)
- ✓ Payload catalog extension covering 5 embedding locations for T4/T5 — v5.0 (Phase 13)

- ✓ Monitor TUI renders T4 capability lists and T5 proof+validity (EVIDENCE column + detail pane, 5-tier stats header, 6-state filter cycle) — v5.0 (Phase 14)
- ✓ Markdown disclosure report adds `Evidence` column (T4 tool lists, T5 `NNN ✓/✗ VALID/INVALID`) interleaved with T1–T3 — v5.0 (Phase 14)
- ✓ Executive summary extends to Tier 4 (Capability Introspection) and Tier 5 (Multi-step Compliance) rows — v5.0 (Phase 14)
- ✓ Backward compat preserved — v4.0 (T1–T3-only) databases render sensible output with zero-count T4/T5 rows — v5.0 (Phase 14)

### Active

- test-agent scorecard and CI exit codes extend to T4/T5 — v5.0
- README Proof Levels section documents 5-tier model; TODOS.md updated — v5.0

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
- Shipped v4.0 with 4,700+ LOC Rust, single binary distribution
- honeyprompt.dev live with live stats counter fetching from /stats API
- Self-hosted deployment path: setup wizard + --domain zero-config + deploy templates
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
4. **Capability introspection** — agent self-reports tools via T4 payload → `/cb/v4/{nonce}/{b64_payload}` (backend shipped in Phase 13; TUI/report surfacing in Phase 14)
5. **Multi-step compliance chain** — agent follows dependency sequence and computes deterministic proof via T5 payload → `/cb/v5/{nonce}/{proof}` (backend shipped in Phase 13; TUI/report surfacing in Phase 14)

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
| dialoguer for setup wizard | Standard Rust crate for interactive CLI prompts; no TUI needed for config setup | ✓ Good — clean UX |
| --domain zero-config mode | Caddy-style: single flag replaces config file for common case; tempdir generation reuses test-agent pattern | ✓ Good — 2-command onboarding |
| CLI > config > defaults precedence | config_with_overrides applies flag overrides to base config; regenerates output if callback_base_url changed | ✓ Good — no stale URLs |
| Static deploy templates with {DOMAIN} | Users copy and sed; not generated by wizard (keeps wizard scope tight) | ✓ Good — simple, auditable |

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
*Last updated: 2026-04-24 — Phase 14 complete: Monitor TUI + Markdown report surfacing shipped (EVIDENCE column, detail pane, 5-tier chrome, exec summary T4/T5 rows). Gap fixes folded in for setup-wizard 5-tier menu, DB parent dir auto-create, attach-mode SELECT, and TUI contrast. Next: Phase 15 (test-agent scorecard + README + TODOS.md).*
