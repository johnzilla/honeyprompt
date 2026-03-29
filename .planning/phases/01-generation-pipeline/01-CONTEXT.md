# Phase 1: Generation Pipeline - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Offline `init` + `generate` workflow producing deployable honeypot pages with a curated prompt-injection payload catalog (Tiers 1–3), robots.txt, ai.txt, and a locked SQLite event store schema with replay detection and session grouping. No network code — pure filesystem + template rendering + schema definition.

</domain>

<decisions>
## Implementation Decisions

### Project structure
- **D-01:** `honeyprompt init` creates a full scaffold: config file, template overrides directory, payload selection file, and output directory
- **D-02:** Config format is TOML (`honeyprompt.toml`) — Rust ecosystem standard
- **D-03:** User-configurable settings: callback URL base, port/bind address, which payload tiers to include, page title/theme

### Payload catalog
- **D-04:** Built-in defaults embedded in binary via rust-embed, but users can drop override TOML files into the project directory
- **D-05:** 2–3 payloads per tier in v1 (~6–9 total), one payload per embedding location — clean signal for research
- **D-06:** Each payload targets one specific embedding location (HTML comment, meta tag, invisible element, JSON-LD, semantic prose, etc.) — not duplicated across locations

### Page design
- **D-07:** `honeyprompt generate` outputs a single directory: `output/` containing index.html, robots.txt, ai.txt, callback-map.json
- **D-08:** Page is obviously a honeypot to humans — clear "this is a security research canary" branding. The goal is to fool AI agents, not humans.
- **D-09:** Visible human warning appears as both a fixed top banner AND an inline notice within the page content

### Callback URLs
- **D-10:** Callback URL paths use opaque tokens: `/cb/{opaque-token}` — tier and payload info encoded internally, nothing revealed in URL structure
- **D-11:** Nonces are 16-char hex strings (8 bytes) — compact URLs, sufficient uniqueness for research
- **D-12:** Nonce-to-payload mappings stored in both JSON ledger file (callback-map.json for human inspection) and SQLite (for programmatic lookup)

### Claude's Discretion
- SQLite schema column details (beyond: replay detection fields, session grouping, parameterized writes)
- Exact page HTML/CSS design within the "obviously a honeypot" constraint
- Template engine choice (minijinja vs tera)
- Payload text/instruction wording within each tier
- Directory structure within the generated scaffold

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project context
- `.planning/PROJECT.md` — Core value, safety model, proof levels, constraints
- `.planning/REQUIREMENTS.md` — Full v1 requirement list with REQ-IDs

### Research findings
- `.planning/research/SUMMARY.md` — Stack recommendations, phase implications, research flags
- `.planning/research/ARCHITECTURE.md` — Component boundaries, build layers, technology mapping with verified crate versions
- `.planning/research/FEATURES.md` — Table stakes vs differentiators, feature dependency tree, MVP recommendation
- `.planning/research/PITFALLS.md` — Critical pitfalls (especially Pitfall 1: embedding locations, Pitfall 2: nonce replay, Pitfall 4: SQL injection)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield Rust project

### Established Patterns
- None yet — this phase establishes the foundational patterns

### Integration Points
- SQLite schema defined here will be consumed by Phase 2 server code
- Payload catalog format defined here will be read by Phase 2 for nonce validation
- Output directory structure defined here will be served by Phase 2's `ServeDir`

</code_context>

<specifics>
## Specific Ideas

- Page should look like a security research tool page, not a deception — humans see "this is a canary" clearly
- Payloads in locations agents parse (HTML comments, meta tags, JSON-LD) while the visible page is obviously labeled
- callback-map.json serves as a human-readable audit trail alongside the SQLite store
- Project scaffold should feel like `cargo init` or `hugo new site` — familiar to CLI users

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-generation-pipeline*
*Context gathered: 2026-03-28*
