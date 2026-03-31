# Phase 2: Server and Detection - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Async HTTP server (`honeyprompt serve`) hosting the honeypot page and callback listener on the same port. Full event pipeline from Axum handler through event broker to DB writer and stdout logger. Agent fingerprinting (UA, IP/ASN, headers, timing), known-crawler suppression, session-based detection counting, and metadata-only mode. No TUI — stdout logging only.

</domain>

<decisions>
## Implementation Decisions

### Event pipeline
- **D-01:** Broadcast architecture: Axum callback handler → mpsc → Event Broker → broadcast → DB Writer + stdout logger. Each consumer receives every event independently.
- **D-02:** Fingerprint extraction: UA string, source IP (proxy-aware), all HTTP headers, ASN/provider lookup from bundled catalog, request timestamp + response latency.
- **D-03:** Callback endpoint returns 204 unconditionally — no body reflection, strict schema validation on path (nonce = alphanumeric, fixed length).

### Crawler catalog
- **D-04:** Known-crawler catalog stored as embedded TOML via rust-embed, same pattern as payload catalog. Users can override with local file.
- **D-05:** UA-primary identification: User-Agent match triggers "known_crawler" label. IP/ASN match adds confidence annotation but doesn't override.
- **D-06:** Three-tier classification: "known_crawler" (GPTBot, ClaudeBot, Googlebot, etc.), "known_agent" (identifiable autonomous agent), "unknown" (unclassified).

### Session model
- **D-07:** Session ID = hash(source IP + User-Agent) with time window expiry (e.g., 30 min gap = new session).
- **D-08:** Detection counting is per-session per-tier. Same agent firing Tier 1 and Tier 2 = 2 detections. Same agent firing Tier 1 ten times = 1 detection.

### Serve UX
- **D-09:** Detailed startup output: bind address, loaded payloads count, nonce count, DB path, then "ready".
- **D-10:** Structured log lines by default (one line per event: timestamp, tier, classification, source IP, UA snippet). `--json` flag for JSON lines output.
- **D-11:** Graceful shutdown on Ctrl+C: finish in-flight requests, flush DB writes, print summary stats, then exit.

### Claude's Discretion
- Event broker task implementation details (tokio::spawn patterns)
- Exact ASN catalog contents and IP range data structure
- Session timeout duration (30 min suggested but flexible)
- Stdout log line format details
- Error handling for malformed callback paths

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project context
- `.planning/PROJECT.md` — Core value, safety model, proof levels, constraints
- `.planning/REQUIREMENTS.md` — v1 requirements with REQ-IDs (CLI-03, SRV-01, SRV-03–07)

### Research findings
- `.planning/research/SUMMARY.md` — Stack recommendations (Axum 0.8, tokio-rusqlite, broadcast channel), phase implications
- `.planning/research/ARCHITECTURE.md` — Event Broker architecture, component build layers, async patterns
- `.planning/research/PITFALLS.md` — Pitfall 3 (UA/IP over-trust), Pitfall 4 (callback injection), Pitfall 5 (robots.txt as signal)

### Phase 1 code
- `src/store/mod.rs` — SQLite schema with replay detection, `insert_nonce` interface
- `src/config/mod.rs` — Config struct with `bind_address`, `port`, `callback_url` fields
- `src/types.rs` — Tier, EmbeddingLocation, Payload, NonceMapping types
- `src/nonce.rs` — Nonce generation and validation

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/store/mod.rs`: SQLite schema with `nonce_map` table, `fire_count`/`is_replay`/`session_id` columns already defined. Phase 2 writes callback events to this schema.
- `src/config/mod.rs`: Config struct already has `bind_address`, `port`, `callback_url` fields. Phase 2 reads these for server startup.
- `src/types.rs`: Tier enum, NonceMapping struct usable for callback validation.
- `src/nonce.rs`: Nonce validation logic reusable for callback path parsing.

### Established Patterns
- rust-embed for binary-embedded assets (payload catalog) — same pattern for crawler catalog
- rusqlite with `params![]` for parameterized queries — mandatory, never `format!()`
- TOML for configuration and data files

### Integration Points
- `src/main.rs` needs `serve` subcommand dispatch (already stubbed in CLI)
- `src/store/mod.rs` needs callback event insertion (new function alongside existing `insert_nonce`)
- `src/lib.rs` needs new modules: `server`, `broker`, `fingerprint`

</code_context>

<specifics>
## Specific Ideas

- Callback endpoint must return 204 always — never reflect input (Pitfall 4 from research)
- Proof level (tier) is the primary classification basis, not UA/IP (Pitfall 3)
- robots.txt visits are friction/noise reduction, not detection events (Pitfall 5)
- Known crawlers should be labeled as "indexed" not "agent complied" in the event store

</specifics>

<deferred>
## Deferred Ideas

- TLS fingerprinting — user selected during scoping but explicitly v2 per PROJECT.md Out of Scope. Noted for future consideration.

</deferred>

---

*Phase: 02-server-and-detection*
*Context gathered: 2026-03-29*
