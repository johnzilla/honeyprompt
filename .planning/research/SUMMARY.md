# Project Research Summary

**Project:** HoneyPrompt
**Domain:** AI agent detection — canary tokens + honeypot + prompt injection compliance measurement
**Researched:** 2026-03-28
**Confidence:** HIGH (architecture, table-stakes features) / MEDIUM (AI-agent-specific detection, emerging space)

> Note: STACK.md was not produced by the parallel research agent. Stack findings below are derived from ARCHITECTURE.md, which contains a comprehensive technology mapping section with verified crate recommendations.

---

## Executive Summary

HoneyPrompt occupies a genuinely novel intersection: it combines the passive tripwire model of canary tokens (Thinkst-style generate-deploy-forget-alert) with honeypot behavior capture and, critically, graduated proof of AI agent compliance with injected instructions. No existing tool measures whether an AI browsing agent follows prompt-injection payloads embedded in web content — and does so with proportional, independently verifiable evidence tiers. The recommended build approach is a single Rust binary with a layered async architecture: offline subcommands (`init`, `generate`, `report`) for filesystem operations and a Tokio + Axum server mode for the HTTP callback listener, event broker, and Ratatui TUI all co-existing in one process on one port. The single-binary model is both a deployment differentiator and a practical fit for security researchers who want zero dependencies.

The feature set must satisfy three distinct user workflows simultaneously: canary token deployment (generate → serve → get alerted), honeypot research (capture → store → analyze → export), and AI agent security research (graduated proof levels, payload diversity, ethical constraints). The graduation of proof from Tier 1 (arbitrary callback) through Tier 5 (multi-step compliance chain) is the tool's core intellectual contribution — no competitor offers this. The flagship user-facing experience is the Ratatui TUI live monitor; it must ship in v1 and it must be compelling enough to screenshot and share.

The principal risks are: (1) false positive inflation from link-checkers and indexing crawlers triggering low-tier callbacks, (2) nonce replay producing phantom detections in research reports, (3) over-trusting User-Agent and IP for agent attribution, and (4) security vulnerabilities in the callback endpoint itself (SQL injection, terminal injection). All four risks must be addressed in Phase 1 schema and server design — retrofitting them is painful. The ethical model (visible human warnings, metadata-only mode, curated-only payloads, no credential collection) is non-negotiable and must be hard-coded, not configuration options.

---

## Key Findings

### Recommended Stack

STACK.md was not produced. The following is derived from the technology mapping in ARCHITECTURE.md, which was researched at HIGH confidence against official docs and crate documentation.

HoneyPrompt is built in Rust with `tokio` as the async runtime. The HTTP layer uses `axum` 0.8 (released January 2025, current stable), serving static files via `tower_http::ServeDir` and receiving callback beacons on the same port. SQLite is the event store, accessed through `tokio-rusqlite` — the standard pattern for bridging the non-`Sync` `rusqlite::Connection` into an async context via a background thread. The TUI is `ratatui` + `crossterm`, driven by a `tokio::select!` loop over broadcast channel events and keyboard input. Template rendering uses `minijinja` or `tera` with compile-time asset embedding via `rust-embed`.

**Core technologies:**
- `tokio` (multi-thread): async runtime — required for concurrent HTTP server + TUI + DB writer in one process
- `axum` 0.8: HTTP server — current stable, clean ergonomics for handler extraction + `ServeDir`
- `tower_http`: static file serving — built-in companion to axum, no additional crate needed
- `tokio-rusqlite`: SQLite async bridge — only correct pattern for rusqlite in async context; direct rusqlite in tokio fails to compile
- `rusqlite` with `params![]`: event store — parameterized queries mandatory; string interpolation is a known CVE vector (RUSTSEC-2025-0043)
- `ratatui` + `crossterm`: TUI — official async template is documented and stable
- `tokio::sync::broadcast`: event fan-out — DB writer and TUI each need every event independently; mpsc would require duplication
- `clap` (derive): CLI parsing — subcommand enum pattern; each verb independently useful
- `minijinja` or `tera` + `rust-embed`: template rendering + asset embedding — compile-time embedding into binary supports single-binary distribution
- `axum-client-ip`: real IP extraction from proxy headers — community crate, MEDIUM confidence
- `axum-extra::TypedHeader`: typed header extraction — official axum-extra, HIGH confidence

### Expected Features

**Must have (table stakes):**
- `init` + `generate` + `serve` CLI subcommands — the core deployment path; users need a honeypot online in under 2 minutes
- HTTP callback listener (same process, same port as static serving) — the only way to receive proof of agent execution
- Prompt-injection payload catalog, Tiers 1–5 — tool is useless without payloads; curated-only for safety
- SQLite event store — persistent capture history; schema must include session/visit grouping and replay flags from day one
- Nonce-per-token generation — cryptographic nonce in callback URL path; validates callback authenticity
- Agent fingerprinting from request metadata (UA, IP, ASN, HTTP headers) — supporting evidence, not primary classification basis
- Ratatui TUI live monitor — the flagship UX; demo-able; non-negotiable for v1
- Markdown report generator (`report` subcommand) — shareable disclosure artifact
- robots.txt + ai.txt generation — establishes explicit opt-out, reduces indexer noise; treated as friction not detection signal
- Visible human warning on every generated page — hard-coded ethical requirement, not a config option
- Metadata-only mode (no request body stored) — default safety stance

**Should have (competitive differentiators):**
- Graduated 5-tier proof levels (Tiers 1–5) — the core intellectual contribution; no competitor does this
- Multiple payload embedding locations per page (HTML comments, meta tags, invisible elements, JSON-LD, semantic prose) — distinguishes surface-skimming from deep-parsing agents
- Conditional-branch callback (Tier 2) — proves agent evaluated logic, not just followed a URL
- Computed callback (Tier 3) — proves agent performed non-sensitive computation
- Capability introspection callback (Tier 4) — proves agent inspected its own tool inventory
- Multi-step compliance chain (Tier 5) — proves agent followed a dependency sequence
- Known-agent lookup enrichment (GPTBot → OpenAI, etc.) — annotates captures with provider names
- Evidence-grade disclosure report (payload description, embedding location, proof level, anonymized metadata)
- Session-based detection counting (visits, not callback rows) — prevents inflated statistics in reports
- "Known good crawler" suppression list — separates indexing traffic from compliance detections

**Defer to v2+:**
- DNS callback listener — significant deployment complexity for marginal v1 benefit
- Custom payload authoring — safety risk; rich curated catalog is sufficient
- Web dashboard — TUI is the flagship; dashboard is a different product
- Email/Slack/webhook alert integrations — JSON export covers v1; researchers can pipe
- TLS fingerprinting — HTTP-level signals adequate for v1
- Multi-page / linked-site deployments — orchestration complexity; v2+
- SIEM integration — enterprise feature; out of scope for research tool
- Windows support — security researchers primarily on Linux/macOS; revisit after core stable

### Architecture Approach

HoneyPrompt uses a layered architecture with two distinct runtime modes: offline mode (synchronous, filesystem + SQLite only, for `init`/`generate`/`report`/`export`) and server mode (async Tokio runtime hosting Axum HTTP server, Event Broker task, DB Writer task, and TUI task simultaneously). The Event Broker is the architectural centerpiece: it receives raw callback events from Axum via mpsc channel, assembles enriched events with fingerprint data, then fans out to both the DB Writer and TUI via a `broadcast` channel — each consumer receives every event independently with no coordination. Pages are generated offline as static files; `serve` mode serves them via `ServeDir` without regeneration. The Fingerprinter is a pure function (request parts → `AgentFingerprint`) with no side effects, making it unit-testable in isolation.

**Major components and build order:**
1. `config/` + `catalog/` (Layer 0) — Config struct, curated payload definitions; no dependencies
2. `fingerprint/` + `store/` (Layer 1) — Pure fingerprinting function; SQLite schema and query interface
3. `generator/` + `broker/` (Layer 2) — HTML/robots.txt/ai.txt rendering; event fan-out with broadcast channel
4. `server/` + `tui/` + `report/` (Layer 3) — Axum server + Ratatui TUI + Markdown reporter; depend on broker and store
5. `cli/` (Layer 4) — Clap dispatch wiring all subcommands; orchestrates everything

### Critical Pitfalls

1. **Payload embedding locations agents actually ignore** — Payloads in HTML comments or CSS-hidden elements produce zero callbacks from modern agents that process a clean-text DOM view. Prevention: distribute payloads across ALL embedding locations per page; include at least one semantically embedded "authoritative-sounding instruction" payload — this class has the highest observed compliance rates. Phase 1 design decision; hard to change post-deployment.

2. **Nonce replay producing phantom detections** — Static nonces embedded in pages can be replayed by link-checkers, human inspectors, or anyone who finds a Google-cached copy. Prevention: flag repeated nonce fires as replay events (not new detections) in the event schema from day one; implement per-visitor nonce injection or TTL-scoped nonces; warn on public IP deployment without disallow rule.

3. **Over-trusting UA/IP for agent classification** — GPTBot/ClaudeBot/Googlebot are indexing crawlers, not autonomous agents; they will trigger Tier 1 callbacks but that is normal crawler behavior. Adversarial agents spoof Chrome UA and route through residential proxies. Prevention: behavioral proof level (did the agent fire the computed callback with the correct derived value?) is the primary classification basis — UA and IP are metadata, not classification criteria.

4. **Callback endpoint as injection vector** — The open-internet callback endpoint is a SSRF, SQL injection, and terminal injection surface. Prevention: parameterized queries (`params![]`) for all DB writes — never `format!()` SQL; strip ANSI escape sequences before TUI rendering; strict schema validation on callback path (nonce = alphanumeric, fixed length); return 204 unconditionally without reflecting input. This is a Phase 1 decision with CVE precedent (RUSTSEC-2025-0043).

5. **robots.txt compliance treated as a detection signal** — 72% of UK sites with AI disallow directives were violated in 2026 Cloudflare data; compliance is voluntary. Treating robots.txt visits as a "proof level 0" pollutes the evidence model. Prevention: robots.txt is friction that reduces noise, not a detection criterion; report generator must treat it as context metadata only.

---

## Implications for Roadmap

Based on research, the architecture's explicit component build order maps directly to a 4-phase structure with a clear dependency chain.

### Phase 1: Foundation — Offline Generation Pipeline

**Rationale:** The deployment path (`init` → `generate`) must be complete and trustworthy before async server work begins. This layer has no runtime dependencies — it is pure filesystem + template rendering. Building it first validates the payload catalog design, nonce generation scheme, and output format before any network code exists. The event store schema must also be locked here because retrofitting replay detection or session grouping is painful.

**Delivers:** Working `honeyprompt init` + `honeyprompt generate` that produce a deployable honeypot page, robots.txt, ai.txt, and a `callback-map.json` nonce ledger. SQLite schema with replay detection fields, session grouping, and parameterized write interface.

**Addresses (from FEATURES.md):** Static honeypot page generation, prompt-injection payload catalog (Tiers 1–5), nonce-per-token generation, robots.txt + ai.txt generation, visible human warning (hard-coded), metadata-only mode schema.

**Avoids (from PITFALLS.md):** Pitfall 1 (multi-location payload distribution locked in catalog design), Pitfall 2 (nonce replay detection in schema from day one), Pitfall 4 (parameterized DB writes, never string interpolation), Pitfall 6 (noindex meta tag + disallow rule generated by default).

**Research flag:** Standard patterns — Rust template rendering, SQLite schema design, and nonce generation are well-documented. No additional research phase needed.

---

### Phase 2: Server Mode — HTTP Callback Listener and Event Pipeline

**Rationale:** With static pages deployable, the async server loop is the next critical path. This validates the core detection loop: agent fires a URL, server receives it, event hits the store. The TUI is explicitly deferred to Phase 3 because the detection loop is valuable (and testable) without it — and building broker + DB writer first means the TUI has a working event source to consume.

**Delivers:** Working `honeyprompt serve` that serves static files and receives callback beacons. Full event pipeline: Axum handler → mpsc → Event Broker → broadcast → DB Writer → SQLite. Agent fingerprinting (UA, IP, ASN, headers). Known-crawler suppression list (GPTBot, ClaudeBot, Googlebot flagged as "indexed," not "agent complied"). Basic stdout event logging before TUI exists.

**Addresses (from FEATURES.md):** HTTP callback listener, agent fingerprinting, known-agent lookup enrichment, SQLite event store population.

**Avoids (from PITFALLS.md):** Pitfall 3 (classification based on proof level, not UA/IP — enforced in event schema), Pitfall 4 (callback endpoint security: strict schema validation, 204-only response, no body reflection), Pitfall 5 (robots.txt not stored as detection event), Pitfall 9 (Tier 1 labeled as weak signal; Tier 2+ required for "potential agent" status), Pitfall 10 (WAL mode with fallback detection at startup).

**Research flag:** The async architecture pattern (tokio-rusqlite + broadcast channel + mpsc handoff) is HIGH confidence against official docs. The `axum-client-ip` crate is MEDIUM confidence (community crate). Fingerprinting catalog of known AI provider ASN ranges will need ongoing maintenance — treat as a data file, not hardcoded logic.

---

### Phase 3: TUI Monitor and Flagship Experience

**Rationale:** The Ratatui TUI is the product's face — the screenshot-able demo that makes HoneyPrompt recognizable. It requires the broadcast channel from Phase 2 to have a live event source. Building it third means the live experience can be built on a working, tested event pipeline. Session-based counting (visits not callback rows) must be enforced here to prevent inflated statistics.

**Delivers:** Working `honeyprompt monitor` (or TUI launched automatically from `serve`). Real-time event table filterable by tier, time, source. Session-based "unique agent visits" count prominent in UI. Replay events flagged visually, not counted as detections. ANSI escape sequence stripping for all agent-supplied strings.

**Addresses (from FEATURES.md):** Live event monitor TUI (flagship UX), filter/sort by tier/time/source.

**Avoids (from PITFALLS.md):** Pitfall 8 (session-based counting, not row counting), Pitfall 11 (async DB reads via spawn_blocking, never synchronous on TUI event loop), Pitfall 3 (TUI clearly labels UA/IP as metadata, not classification basis).

**Research flag:** The Ratatui async pattern (`tokio::select!` over broadcast + crossterm EventStream) is HIGH confidence from official Ratatui async template docs. The UX design for "unique agent sessions vs. callback counts" is product judgment — no external research needed.

---

### Phase 4: Report Generator and Offline Analysis

**Rationale:** The report subcommand only reads from the already-built store. It is the last piece because its quality depends on the event schema being stable (Phases 1–2) and the session model being defined (Phase 3). Shipping last allows the report format to reflect the final evidence model.

**Delivers:** Working `honeyprompt report` that produces a structured Markdown disclosure artifact. Covers: payload description, embedding location, proof level breakdown by session, callback timestamps, anonymized agent metadata. All agent-supplied strings escaped before Markdown interpolation.

**Addresses (from FEATURES.md):** Markdown report generator, evidence-grade disclosure report suitable for sharing with platform teams or including in research papers.

**Avoids (from PITFALLS.md):** Pitfall 8 (visit-based counts, not callback rows), Pitfall 12 (agent-supplied strings escaped before Markdown rendering), Pitfall 5 (robots.txt appears as context metadata, not a detection event in the report).

**Research flag:** Standard patterns — Markdown generation is trivial. No research phase needed.

---

### Phase Ordering Rationale

- **Dependency chain is strict:** The catalog must exist before the generator, the generator before the server (serves static files), the server before the TUI (needs event source), and all three before the reporter (reads from stable store).
- **ARCHITECTURE.md explicitly encodes this order** in its component build layers (Layer 0 → Layer 4). The phase structure above maps directly to those layers.
- **Pitfall front-loading:** Five of the twelve identified pitfalls must be addressed in Phase 1 schema decisions. The architecture research explicitly warns that retrofitting nonce replay detection, session grouping, or parameterized DB writes is painful — these are not Phase 2 problems.
- **TUI last among server-mode components:** The TUI is the flagship experience but it is also the most UI-polish-intensive component. Building it on a working event pipeline prevents rework.

### Research Flags

Phases needing deeper research during planning:
- **Phase 2 (server mode):** The fingerprinting catalog of known AI provider IP/ASN ranges is data that evolves continuously. Plan for this as a versioned data file (not hardcoded) with an update mechanism. Evaluate whether `axum-client-ip` is maintained enough for production use before committing to it.
- **Phase 3 (TUI design):** The UX for distinguishing "indexed by crawler" from "complied by agent" is novel and has no prior art to copy. The session model and UI hierarchy deserve a planning-time design pass before implementation.

Phases with standard patterns (no additional research needed):
- **Phase 1 (generation pipeline):** Rust template rendering, SQLite schema, nonce generation — all well-documented. HIGH confidence.
- **Phase 4 (report generator):** Markdown generation from SQLite queries — trivially standard.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Derived from ARCHITECTURE.md technology mapping, which was verified against official Tokio, Axum, Ratatui, and docs.rs sources. STACK.md was not produced by research agent. |
| Features | HIGH (table stakes) / MEDIUM (AI-agent-specific) | Table stakes canary token features are well-documented via Canarytokens.org and OpenCanary. AI-agent-specific detection and graduated proof levels are novel; real-world validation of Tiers 4–5 is still needed. |
| Architecture | HIGH | Verified against official Tokio channels tutorial, Axum 0.8 announcement, tokio-rusqlite docs, and Ratatui async template. Async patterns are idiomatic and stable. |
| Pitfalls | HIGH | Pitfalls sourced from real CVEs (RUSTSEC-2025-0043), published research (arXiv:2410.13919, BrowseSafe), and live observational data (Cloudflare 72% violation rate). High confidence in applicability. |

**Overall confidence:** HIGH for the build approach; MEDIUM for AI-agent detection signal quality (the space is evolving rapidly).

### Gaps to Address

- **STACK.md not produced:** Stack recommendations are derived from ARCHITECTURE.md's technology mapping. If the stack researcher was unable to complete their file, the technology choices should be reviewed during Phase 1 planning to confirm crate versions and alternatives. The ARCHITECTURE.md technology table is comprehensive enough that this is not blocking.
- **Tier 4–5 proof level validation:** Capability introspection (Tier 4) and multi-step compliance chain (Tier 5) rely on assumptions about how agents self-report tool inventories and follow dependency sequences. These are theoretically sound but require real-world validation against actual deployed agents. Plan a research spike during Phase 1 to test Tier 4–5 payload designs against available agent platforms before finalizing the catalog.
- **Per-visitor nonce injection:** The architecture currently describes static nonce generation at `generate` time, but Pitfall 2 identifies per-visitor nonce injection as the correct approach. This requires the serve mode to inject nonces dynamically (e.g., via a redirect or cookie seed) rather than serving a fully static HTML file. This is a design decision that needs resolution in Phase 1/2 planning — it affects whether `ServeDir` static serving is sufficient or whether a dynamic handler is needed for the main page.
- **ai.txt / agents.txt standards stability:** These emerging standards (2025–2026) are not yet stable. Include them as low-effort features but do not build compliance measurement of these standards into proof level logic until they stabilize.
- **Known AI provider ASN catalog maintenance:** The catalog of known AI crawler IP ranges needs a clear ownership and update story. Define this as a versioned data file during Phase 2 planning.

---

## Sources

### Primary (HIGH confidence)
- [Ratatui async template — official docs](https://ratatui.github.io/async-template/02-structure.html) — TUI architecture patterns
- [Ratatui async event stream tutorial — official docs](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) — tokio::select! event loop pattern
- [Axum 0.8.0 announcement — official Tokio blog](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) — current stable version confirmation
- [tokio-rusqlite docs — docs.rs](https://docs.rs/tokio-rusqlite/latest/tokio_rusqlite/) — async SQLite bridge pattern
- [Tokio channels tutorial — official Tokio docs](https://tokio.rs/tokio/tutorial/channels) — broadcast vs. mpsc selection
- [axum-extra TypedHeader — docs.rs](https://docs.rs/axum-extra/latest/axum_extra/struct.TypedHeader.html) — typed header extraction
- [Canarytokens.org documentation](https://docs.canarytokens.org/guide/) — table stakes reference for canary token features
- [OWASP LLM Top 10 2025 — LLM01: Prompt Injection](https://genai.owasp.org/llmrisk/llm01-prompt-injection/) — establishes tool relevance
- [RUSTSEC-2025-0043 — SQLite SQL injection via format!()](https://rustsec.org/advisories/RUSTSEC-2025-0043.html) — parameterized query requirement
- [AI Crawlers violate robots.txt on 72% of UK sites (Cloudflare / 365i)](https://www.365i.co.uk/blog/2026/01/07/ai-crawler-compliance-tracking-cloudflare/) — robots.txt as friction, not signal

### Secondary (MEDIUM confidence)
- [LLM Agent Honeypot arXiv:2410.13919 (Palisade Research)](https://arxiv.org/html/2410.13919v2) — prompt injection + timing analysis detection
- [BrowseSafe — arXiv:2511.20597](https://arxiv.org/html/2511.20597v1) — DOM view filtering by modern agents
- [Indirect prompt injection in web-browsing agents (promptfoo)](https://www.promptfoo.dev/blog/indirect-prompt-injection-web-agents/) — embedding location effectiveness
- [AI Agents Spoofing Their Way In (Datadome)](https://datadome.co/agent-trust-management/ai-agent-spoofing/) — UA spoofing prevalence
- [HUMAN Security AI agent detection guide](https://www.humansecurity.com/learn/blog/ai-agent-signals-traffic-detection/) — behavioral fingerprinting signals
- [User agent strings to HTTP signatures (Arcjet)](https://blog.arcjet.com/user-agent-strings-to-http-signatures-methods-for-ai-agent-identification/) — fingerprinting signal quality
- [Multi-interface Rust app with Ratatui + Axum (community)](https://dev.to/sebyx07/building-a-multi-interface-todo-app-with-rust-ratatui-and-axum-1cke) — co-located server + TUI architecture pattern
- [axum-client-ip crate](https://crates.io/crates/axum-client-ip) — proxy-aware IP extraction

### Tertiary (LOW confidence)
- [New AI web standards 2026: rethinking robots.txt (DEV Community)](https://dev.to/astro-official/new-ai-web-standards-and-scraping-trends-in-2026-rethinking-robotstxt-3730) — ai.txt / llms.txt / agents.txt status; standards not yet stable
- Timing-based agent classification (Palisade Research) — response timing as supplementary signal; fragile, excluded from v1

---

*Research completed: 2026-03-28*
*Ready for roadmap: yes*
