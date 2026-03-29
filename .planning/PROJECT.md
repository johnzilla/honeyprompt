# HoneyPrompt

## What This Is

HoneyPrompt is a terminal-first security tool that detects and measures unsafe behavior by AI browsing agents. It generates honeypot web pages containing visible human warnings and hidden prompt-injection canaries, then records HTTP callbacks that prove varying levels of agent compliance with injected instructions. Built in Rust as a single binary for security researchers, defenders, and platform teams who want evidence of agentic web abuse without collecting secrets or performing harmful actions.

v1.0 ships a complete workflow: `init` → `generate` → `serve` → `monitor` → `report`. 3,650 lines of Rust across 4 phases, 10 plans.

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

### Active

- None — planning next milestone

### Out of Scope

- DNS callback listener — v2 (adds operational complexity, requires domain delegation)
- Custom payload authoring — v2 (curated-only ensures safety in v1)
- Full web dashboard — v2+ (TUI is the primary interface)
- TLS fingerprinting — v2 (complexity vs value tradeoff for v1)
- Windows support — v2+ (Linux and macOS first)
- Secret/credential collection — never (violates safety model)
- Active exploitation or offensive automation — never (ethical boundary)
- Multi-page or linked-site deployments — v2+
- Per-payload breakdown in reports — v2 (executive summary only in v1)
- Alternative report formats (JSON, HTML) — v2 (Markdown only in v1)

## Context

- Security research tool in the emerging AI agent detection space
- No direct competitor does passive, deployable canary tokens specifically for AI browsing agents
- Shipped v1.0 with 3,650 LOC Rust, single binary distribution
- Tech stack: Clap CLI, Axum HTTP, Ratatui TUI, rusqlite storage, rust-embed assets
- The TUI monitor is the flagship experience — the demo screenshot, the sell
- All payloads designed so success signals never require transmitting secrets
- Metadata-only mode enforced: no callback body accepted (SRV-07)

### Proof Levels

1. **Arbitrary callback** — Agent executed an injected outbound request
2. **Conditional callback** — Agent evaluated a condition and selected the correct branch
3. **Computed callback** — Agent performed a non-sensitive computation and returned the result
4. **Capability introspection** — v2 (agent self-reports tools)
5. **Multi-step compliance chain** — v2 (agent follows dependency sequence)

### Safety Model

- No API keys, session cookies, prompt contents, user data, file contents, or env vars
- HTTP beacons carry only prompt ID, nonce, tier, and derived values
- Safe payloads only — unsafe custom payloads impossible in v1
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
*Last updated: 2026-03-29 after v1.0 milestone completion*
