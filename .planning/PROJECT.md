# HoneyPrompt

## What This Is

HoneyPrompt is a terminal-first security tool that detects and measures unsafe behavior by AI browsing agents. It generates honeypot web pages containing visible human warnings and hidden prompt-injection canaries, then records HTTP callbacks that prove varying levels of agent compliance with injected instructions. Built in Rust for security researchers, defenders, and platform teams who want evidence of agentic web abuse without collecting secrets or performing harmful actions.

## Core Value

Provide graduated, verifiable proof that AI agents follow prompt-injection instructions from untrusted web content — without requiring secrets or causing harm.

## Requirements

### Validated

- ✓ CLI workflow for init and generate — Phase 1
- ✓ Static honeypot page generator with visible human warnings — Phase 1
- ✓ Curated prompt-injection payload catalog (Tiers 1-3) — Phase 1
- ✓ SQLite event store schema with replay detection — Phase 1
- ✓ robots.txt and ai.txt generation with disallow rules — Phase 1
- ✓ HTTP callback listener integrated with honeypot server (all-in-one) — Phase 2
- ✓ Agent fingerprinting from request metadata (IP, UA, headers, ASN) — Phase 2
- ✓ TUI monitor for live event viewing with filters — Phase 3
- ✓ CLI workflow for report — Phase 4
- ✓ Markdown disclosure report generator — Phase 4
- ✓ Minimal honeyprompt.sh landing page (instrumented with its own canaries) — Phase 4

### Active

- None — all v1 requirements validated

### Out of Scope

- DNS callback listener — v2 (adds operational complexity, requires domain delegation)
- Custom payload authoring — v2 (curated-only ensures safety in v1)
- Full web dashboard — v2+ (TUI is the primary interface)
- TLS fingerprinting — v2 (complexity vs value tradeoff for v1)
- Windows support — v2+ (Linux and macOS first)
- Secret/credential collection — never (violates safety model)
- Active exploitation or offensive automation — never (ethical boundary)
- Multi-page or linked-site deployments — v2+

## Context

- Security research tool in the emerging AI agent detection space
- No direct competitor does passive, deployable canary tokens specifically for AI browsing agents
- Existing tools either test known models via API or target classic attackers, not unknown browsing agents in the wild
- The TUI monitor is the flagship experience — the demo screenshot, the sell
- All payloads must be designed so success signals never require transmitting secrets
- Callback data limited to: prompt ID, nonce, tier/proof level, derived non-sensitive values, optional agent self-reported capability flags
- "Metadata only" mode should be supported where no callback body is accepted

### Proof Levels

1. **Arbitrary callback** — Agent executed an injected outbound request
2. **Conditional callback** — Agent evaluated a condition and selected the correct branch
3. **Computed callback** — Agent performed a non-sensitive computation and returned the result
4. **Capability introspection** — Agent inspected its own tools/permissions and encoded non-sensitive metadata
5. **Multi-step compliance chain** — Agent completed a sequence of dependent harmless actions

### Payload Embedding Locations

HTML comments, metadata tags, invisible elements, alternate representations, structured content (JSON-LD, microdata), linked resources

### Safety Model

- No API keys, session cookies, prompt contents, user data, file contents, or env vars
- Prefer HTTP beacons carrying only prompt ID, nonce, tier, and derived values
- Metadata-only mode: only path, query, headers, and connection metadata accepted
- Safe payloads only — unsafe custom payloads impossible in v1

## Constraints

- **Language**: Rust — single-binary distribution, performance, security community credibility
- **CLI**: Clap for argument parsing
- **TUI**: Ratatui for terminal UI
- **HTTP**: Axum or equivalent lightweight async stack
- **Storage**: SQLite via rusqlite or similar
- **Templates**: Built-in for site generation and reports
- **Platform**: Linux and macOS first
- **Performance**: Fast startup, low memory footprint
- **Ethics**: All generated content must include visible warnings for humans; payloads must be auditable

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| HTTP-only callbacks in v1 | DNS requires domain delegation setup, adds user complexity | — Pending |
| Curated payloads only in v1 | Safety guarantee — prevents users from creating harmful payloads | — Pending |
| All-in-one server model | Simpler UX — one process serves honeypot + listens for callbacks | — Pending |
| All 5 proof levels in v1 | Graduated evidence model is core to the value proposition | — Pending |
| TUI as flagship experience | The live monitoring view is the demo, the screenshot, the sell | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-29 after Phase 4 completion — all v1 phases complete*
