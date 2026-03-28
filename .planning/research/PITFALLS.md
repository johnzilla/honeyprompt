# Domain Pitfalls

**Domain:** AI agent honeypot / canary token security tool
**Project:** HoneyPrompt
**Researched:** 2026-03-28

---

## Critical Pitfalls

Mistakes that cause rewrites, security incidents, or invalidate all collected evidence.

---

### Pitfall 1: Payload Embedding Locations Agents Actually Ignore

**What goes wrong:** Payloads embedded only in HTML comments or `display:none` CSS elements produce no callbacks — not because agents are evasion-aware, but because many modern AI browsing agents render-filter their DOM view before the LLM sees the page. Agents process a "clean text" or "accessibility tree" representation, not raw HTML.

**Why it happens:** Research from 2025 (BrowseSafe, promptfoo) confirms that HTML comments and hidden elements are *easier* to detect and filter by defensive agents — which means attackers have also trained their agents to strip these. Meanwhile, semantically embedded payloads woven into visible prose have the highest trigger rates across model families.

**Consequences:** An entire proof-level tier built around hidden-element payloads produces zero callbacks in the wild. The tool looks broken. Researchers lose confidence in the data.

**Prevention:**
- Distribute payloads across ALL embedding locations per honeypot page: HTML comments (catches legacy crawlers), CSS-hidden elements (catches some), visible-but-camouflaged semantic prose (catches modern agents), JSON-LD/structured data (catches structured-content-focused agents), metadata tags.
- Include at least one semantically embedded "authoritative-sounding instruction" payload per page — this class has the highest observed compliance rates.
- Design proof levels to collect evidence from whichever embedding locations work, not assume one location works universally.

**Detection (warning sign):** Zero Proof Level 1 (arbitrary callback) hits after deploying to a known-active AI agent operator. If level 1 gets no hits from any agent in 72 hours of legitimate traffic, the embedding locations are not reaching agents.

**Phase:** Payload catalog design (Phase 1/2). Hard to fix after deployment because changing payloads invalidates historical nonce mapping.

---

### Pitfall 2: Nonce/Callback Integrity — No Replay Prevention

**What goes wrong:** A nonce generated once and embedded statically in an HTML page can be replayed. If an agent (or a human researcher probing the tool) copies the callback URL from the source and fires it manually — or if a benign link-checker fires it — the event store records a false positive with a legitimate-looking nonce. Worse: if the same honeypot page is served to multiple visitors with the same nonce, you cannot distinguish "one agent fired once" from "one human found the URL and replayed it."

**Why it happens:** Statically generated pages embed fixed nonces for simplicity. Replay attacks are invisible in the event store because the nonce validates correctly.

**Consequences:** Research reports cite detections that are actually link-checkers or human inspectors. Security researchers citing HoneyPrompt data will be embarrassed. Erosion of tool credibility.

**Prevention:**
- Per-visitor nonces: generate a fresh nonce for every page load, not per generated static file. This requires the serve mode to inject nonces dynamically or embed them via a redirect/cookie seed.
- Record the first-fire timestamp and flag subsequent fires of the same nonce as replay events, not new detections.
- Implement a short TTL window for nonce validity (configurable, default 24h). Callbacks arriving after TTL are flagged as stale.
- In metadata-only mode (no body accepted), use path-segment encoding of nonce+timestamp so each callback URL is time-scoped.

**Detection (warning sign):** The same nonce fires more than once. Any repeated nonce in the event store is a replay — flag it in the TUI monitor immediately, do not count it as an independent detection.

**Phase:** HTTP callback listener and event store design (Phase 1). This is a foundational schema decision; retrofitting is painful.

---

### Pitfall 3: Over-Trusting User-Agent and IP for Agent Attribution

**What goes wrong:** Attributing "AI agent" status based on User-Agent strings or IP ASN lookup produces both false positives (legitimate OpenAI crawlers doing normal indexing) and false negatives (agents that spoof Chrome UA and route through residential proxies or Cloudflare Workers).

**Why it happens:** The temptation is to say "this request came from 34.x.x.x (Google Cloud) with a GPTBot UA, therefore it's an agent." But: (1) known AI crawlers like GPTBot/ClaudeBot/Googlebot are *indexing* crawlers, not action-taking agents — they will trigger callbacks but that is normal crawler behavior, not agentic compliance. (2) Datadome and HUMAN Security research from 2025 documents AI agents actively spoofing Chrome User-Agents. (3) Cloudflare Workers and Lambda allow any origin IP.

**Consequences:** Reports claim to show "AI agent detections" that are actually Googlebot indexing the honeypot page — a worthless signal. Conversely, genuinely malicious autonomous agents are classified as "unknown human."

**Prevention:**
- Never classify a callback as "AI agent detected" based on UA or IP alone. The callback is only interesting when an agent *followed the injected instruction*, which is demonstrated by proof level, not origin metadata.
- Maintain a separate "known good crawler" blocklist/allowlist to suppress noise from GPTBot, ClaudeBot, Bingbot, Googlebot. These are flagged as "crawler indexed" not "agent complied."
- Store all metadata (UA, IP, ASN, headers) but mark it as supporting evidence, not classification basis.
- Use behavioral signals: did the agent fire the *specific* computed callback with the correct derived value? That cannot be spoofed by a passive crawler.

**Detection (warning sign):** TUI monitor showing many Proof Level 1 hits from ASNs belonging to OpenAI/Google/Anthropic infrastructure — these are indexing crawlers, not autonomous agents.

**Phase:** Fingerprinting module and event schema (Phase 1/2). Classification logic must be explicit in the schema so the TUI does not present misleading counts.

---

### Pitfall 4: Callback Listener Accepts Arbitrary Payloads — SSRF and Log Injection Risk

**What goes wrong:** The honeypot callback endpoint accepts inbound HTTP requests from the open internet. If that endpoint reflects any request data (body, query, headers) into the database or terminal output without sanitization, it becomes a vector for: (1) SSRF via crafted redirect responses if the server follows any redirects, (2) SQL injection into the event store via unsanitized query parameters, (3) terminal injection (ANSI escape codes) into the TUI display.

**Why it happens:** "It's just a logging endpoint" thinking. The 2025 matrix-sdk-sqlite SQL injection (CVE-2025-53549, RUSTSEC-2025-0043) was caused by the same pattern: using `format!()` to build SQL from external input.

**Consequences:** An attacker (or a curious AI agent) can probe the callback endpoint with crafted payloads and corrupt the event store, achieve SSRF to internal services, or inject escape sequences that crash the TUI or hide events.

**Prevention:**
- Use parameterized queries (SQLx `query!` macro or rusqlite `params![]`) for all event store writes. Never use string interpolation for SQL.
- Strip or reject ANSI escape sequences from any string destined for TUI display before rendering.
- The callback listener must only accept: nonce (alphanumeric, fixed length), proof level (enum), tier-specific derived values (bounded schema). Reject anything outside schema. Return 204 unconditionally — do not reflect input.
- Never follow redirects on the callback listener side. The listener receives; it does not fetch.
- Bind the callback listener port to localhost or a defined interface in default config. Users who expose it to the internet do so explicitly.

**Detection (warning sign):** Any query parameter in a callback request that is longer than the expected nonce length, or contains SQL metacharacters, or contains escape sequences.

**Phase:** HTTP server and event store (Phase 1). Must be enforced at the schema level from day one.

---

### Pitfall 5: robots.txt / ai.txt as a Detection Signal — Compliance Is Voluntary and Declining

**What goes wrong:** Generating robots.txt with `Disallow: /` and ai.txt as a "detection layer" creates the false impression that any agent crawling the honeypot has violated the disallow directive. In reality, 72% of UK sites with AI crawler disallow directives were violated in 2026 Cloudflare data. Compliance is voluntary. Treating robots.txt compliance as a meaningful signal pollutes the report.

**Why it happens:** robots.txt is an obvious "signal of interest" — if you disallow and they come anyway, that seems meaningful. But legitimate research crawlers and search indexers also violate robots.txt (unintentionally or by policy), so the signal is noisy.

**Consequences:** Reports claim "agent violated robots.txt" as a separate proof when it is not distinguishable from normal crawler non-compliance. Devalues the proof-level evidence model.

**Prevention:**
- Generate robots.txt and ai.txt as friction for legitimate traffic reduction (reduces noise), not as a detection criterion in its own right.
- Do NOT create a "proof level 0: visited despite disallow" — it is not meaningful evidence of agentic behavior.
- In the report generator, note robots.txt status as context metadata, not as a detection event.
- Document in the tool that robots.txt is friction, not enforcement.

**Detection (warning sign):** If the report generator assigns a positive detection status to requests that only have the "visited despite Disallow" property with no proof-level callback.

**Phase:** robots.txt generation and report generator (Phase 2/3). Design the report schema to treat robots.txt as metadata, not a detection signal, from the start.

---

## Moderate Pitfalls

---

### Pitfall 6: Static Honeypot Pages Are Indexed and the Payloads Become Public

**What goes wrong:** A researcher deploys HoneyPrompt, Google indexes the honeypot page, and the prompt-injection payload text appears in Google's index. Security researchers or AI safety teams find it. Now every agent developer knows the exact payload text and can filter it. Worse: the nonces embedded in the static page are now public, enabling replay attacks from anyone who finds the Googlebot-cached copy.

**Why it happens:** Static HTML served on the open web is public. The page is designed to look like a legitimate page to humans — which also means it looks indexable to crawlers.

**Prevention:**
- Generate a robots.txt `Disallow` for the honeypot page paths and a `noindex` meta tag by default.
- Rotate payload text variants in the catalog so a known payload leaking doesn't invalidate all future deployments. Payloads should be semantically equivalent but textually varied.
- Per-visitor nonce injection (see Pitfall 2) means Googlebot's cached copy has a dead nonce, not a replayable one.
- Warn users in the CLI output when serving on a public IP without a disallow rule.

**Detection (warning sign):** Googlebot/Bingbot User-Agent appears in the event store before any "real" agent callbacks. Check whether the page was indexed.

**Phase:** Site generator and serve command (Phase 1/2).

---

### Pitfall 7: Timing-Based Agent Classification Is Fragile

**What goes wrong:** The Palisade Research honeypot and similar projects use "responded in < 1.5 seconds" as a signal of LLM agency vs. human interaction. HoneyPrompt's HTTP callback model does not involve interactive responses — agents fire a callback URL, not engage in a conversation. But if HoneyPrompt ever adds interactive classification (future phases), the timing approach has known weaknesses: slow networks, throttled agents, and human copy-paste all create overlap.

**Why it happens:** Timing is the only passive behavioral signal available when content-based classification fails.

**Consequences:** False negatives (agents classified as humans because they throttled) and false positives (fast-typing humans or automated testing frameworks classified as agents).

**Prevention:**
- For v1, timing is irrelevant — the callback model does not require timing. Do not introduce it.
- If timing is added later (e.g., for an interactive challenge page), treat it as a supporting signal only, never the sole classification basis.
- Document the fragility in the TUI: "Response timing is supplementary evidence only."

**Phase:** Future interactive detection phases only. Not applicable to v1 HTTP callback model.

---

### Pitfall 8: Proof Level Tier Inflation — Miscounting Levels as Independent Confirmations

**What goes wrong:** An agent fires both Proof Level 1 (arbitrary callback) and Proof Level 3 (computed callback) from the same page visit. The event store records two events. The report counts this as "two detections." In reality, it is one agent visit with multi-level compliance, which is stronger evidence — but the framing matters. Treating each fired proof level as a separate "detection" inflates apparent hit counts and misleads readers.

**Why it happens:** It is natural to count database rows as detections. If each proof level is a separate row, naive counting inflates the number.

**Consequences:** Research reports citing HoneyPrompt data are accused of inflated statistics. Trust damage.

**Prevention:**
- The event schema must link proof levels to a parent "visit" or "session" identified by shared nonce seed or IP+timestamp window.
- Reports show "X agent visits detected, with Y demonstrating computed-level compliance" — visits, not callback count.
- TUI monitor should display "unique agent sessions" prominently, with per-session proof level breakdown.

**Detection (warning sign):** Report output showing detection counts significantly higher than unique IP+nonce combos.

**Phase:** Event store schema and report generator (Phase 1 and Phase 3).

---

### Pitfall 9: Legitimate Security Scanners Trigger Proof Level 1

**What goes wrong:** Link-checking tools, security scanners (Shodan, ZMap), and website monitoring services fire all discovered URLs including callback URLs embedded in HTML. This produces Proof Level 1 (arbitrary callback) hits from benign automated tools, not AI agents.

**Why it happens:** Proof Level 1 only requires "something made an outbound HTTP request to a URL found in the page." That is exactly what link checkers do.

**Consequences:** False positives inflate detection counts. Proof Level 1 loses credibility as an evidence tier.

**Prevention:**
- Proof Level 1 should be clearly labeled in the TUI and reports as "weakest signal — consistent with link-checker behavior."
- Cross-reference Level 1 callbacks against the known scanner/crawler blocklist.
- Require at least Level 2 (conditional branch selection) before the TUI promotes a session to "potential agent" status.
- Document the limitation in the tool: Level 1 alone is not actionable evidence of AI agency.

**Detection (warning sign):** Level 1 callbacks firing within seconds of a page load, from ASNs associated with scanning infrastructure (Shodan, Censys, VirusTotal).

**Phase:** Event classification logic and TUI design (Phase 2).

---

## Minor Pitfalls

---

### Pitfall 10: Single-Binary Distribution Broken by SQLite WAL Mode on Some Filesystems

**What goes wrong:** rusqlite in WAL (Write-Ahead Logging) mode requires the filesystem to support shared memory (`-shm` files). On NFS, some Docker overlay filesystems, and certain embedded Linux environments, WAL mode fails silently or corrupts the database.

**Prevention:** Default to WAL mode but detect failure at startup and fall back to DELETE journal mode with a logged warning. Document the limitation.

**Phase:** Event store initialization (Phase 1).

---

### Pitfall 11: Ratatui TUI Blocks on Synchronous DB Reads During Live Monitor

**What goes wrong:** The TUI live monitor polls the SQLite event store for new events. If reads are synchronous on the main event loop thread, slow disk I/O (especially on NFS or encrypted FS) will cause the terminal to stutter or drop input events.

**Prevention:** Run DB reads on a Tokio background task via `tokio::spawn` or `spawn_blocking`. Send results to the TUI via a channel. The TUI event loop only reads from the channel, never the database directly.

**Phase:** TUI monitor implementation (Phase 2).

---

### Pitfall 12: Report Markdown Contains Unescaped Agent-Supplied Data

**What goes wrong:** The report generator includes "agent self-reported capability flags" or UA strings in Markdown output. If an agent sends a UA string containing Markdown metacharacters (backticks, brackets, asterisks), the report renders unexpectedly or, in rendered HTML output, introduces XSS.

**Prevention:** Escape all agent-supplied strings before interpolating into Markdown. For HTML export, apply HTML escaping. Never trust any string from the callback request body.

**Phase:** Report generator (Phase 3).

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|---|---|---|
| Payload catalog design | Pitfall 1: Hidden-only payloads miss modern agents | Multi-location distribution, include semantic embedding |
| HTTP callback listener | Pitfall 4: SSRF + SQL injection via callback body | Parameterized queries, strict schema validation, no body reflection |
| Event store schema | Pitfall 2: Nonce replay uncounted | Per-visitor nonces, replay detection flag in schema from day one |
| Fingerprinting module | Pitfall 3: UA/IP over-trusted for classification | Behavioral signal (proof level) is primary; UA/IP is metadata |
| robots.txt generator | Pitfall 5: Compliance treated as detection signal | robots.txt = friction only, not a proof level |
| TUI monitor | Pitfall 8 + 11: Inflated counts, UI stutter | Session-based counting, async DB reads |
| Site generator | Pitfall 6: Page indexed, payloads public | noindex meta tag, disallow rule generated by default |
| Report generator | Pitfall 8 + 12: Inflated stats, Markdown injection | Visit-based counts, escape all agent-supplied strings |
| Proof Level 1 design | Pitfall 9: Link-checkers trigger Level 1 | Label Level 1 as weak signal, require Level 2+ for "potential agent" status |

---

## Sources

- LLM Agent Honeypot (Palisade Research): https://ai-honeypot.palisaderesearch.org/explainer
- LLM Agent Honeypot — arXiv paper: https://arxiv.org/html/2410.13919v2
- BrowseSafe: Understanding and Preventing Prompt Injection Within AI Browser Agents: https://arxiv.org/html/2511.20597v1
- Indirect Prompt Injection in Web-Browsing Agents (promptfoo): https://www.promptfoo.dev/blog/indirect-prompt-injection-web-agents/
- Fooling AI Agents: Web-Based Indirect Prompt Injection in the Wild (Unit 42): https://unit42.paloaltonetworks.com/ai-agent-prompt-injection/
- AI Agent Detection Guide (HUMAN Security): https://www.humansecurity.com/learn/blog/ai-agent-signals-traffic-detection/
- AI Agents Are Spoofing Their Way In (Datadome): https://datadome.co/agent-trust-management/ai-agent-spoofing/
- AI Crawlers Violate robots.txt on 72% of UK Sites (Cloudflare / 365i): https://www.365i.co.uk/blog/2026/01/07/ai-crawler-compliance-tracking-cloudflare/
- Canary Token Detection and Bypass (Lupovis): https://www.lupovis.io/detecting-canary-tokens-and-seeds-without-raising-an-alert/
- RUSTSEC-2025-0043 — SQLite SQL Injection via format!(): https://rustsec.org/advisories/RUSTSEC-2025-0043.html
- Privacy and Legal Implications of Honeypot Data (Springer): https://link.springer.com/chapter/10.1007/978-3-032-09660-9_2
- Detecting and Analyzing Prompt Abuse in AI Tools (Microsoft Security, March 2026): https://www.microsoft.com/en-us/security/blog/2026/03/12/detecting-analyzing-prompt-abuse-in-ai-tools/
- Canary Tokens at Grafana Labs: https://grafana.com/blog/2025/08/25/canary-tokens-learn-all-about-the-unsung-heroes-of-security-at-grafana-labs/
- Ratatui Async Event Handling: https://ratatui.rs/tutorials/counter-async-app/async-event-stream/
