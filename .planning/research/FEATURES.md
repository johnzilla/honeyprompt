# Feature Landscape

**Domain:** AI agent detection / honeypot canary token security tool
**Researched:** 2026-03-28
**Overall confidence:** HIGH (core canary/honeypot features) / MEDIUM (AI-agent-specific features, emerging space)

---

## Context: What HoneyPrompt Is

HoneyPrompt occupies a novel intersection: canary tokens (Thinkst-style passive tripwires) + honeypots (behavior capture) + prompt injection research (AI agent compliance measurement). No direct competitor does passive, deployable, graduated-proof canary tokens specifically for AI browsing agents. Classic canary tools target human attackers; classic honeypots measure what attackers do; HoneyPrompt measures specifically whether AI agents follow injected instructions from web content they browse.

This means features must satisfy three distinct user needs simultaneously:
1. **Canary token workflows** — deploy, forget, get alerted when triggered
2. **Honeypot research workflows** — capture, store, analyze, export attacker behavior
3. **AI agent security research** — graduated proof, payload diversity, ethical constraints

---

## Table Stakes

Features users expect. Missing = product feels incomplete or unprofessional.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Static honeypot page generation** | Core delivery mechanism; users need a deployable artifact | Low | Templated HTML with injected payloads; visible human warnings required |
| **HTTP callback listener** | The only way to receive proof of agent execution | Low | Must coexist on same server as honeypot for single-binary UX |
| **Prompt-injection payload catalog** | The tool is useless without payloads to inject | Medium | Curated set prevents unsafe custom payloads; all 5 proof levels required |
| **SQLite event store** | Researchers need persistent capture history | Low | Simple schema: prompt ID, nonce, tier, timestamp, request metadata |
| **robots.txt generation with AI disallow rules** | Establishes explicit opt-out; makes compliance violations legally/ethically legible | Low | User-agent-specific rules for GPTBot, ClaudeBot, PerplexityBot, etc. |
| **Agent fingerprinting from request metadata** | Differentiates known crawlers from unknown agents | Medium | UA string, IP/ASN, known AI provider ranges, HTTP signature verification |
| **Live event monitor (TUI)** | Real-time visibility is the flagship UX; demo-able screenshot | Medium | Ratatui; filter/sort by tier, time, source; the tool's "face" |
| **CLI workflow (`init`, `generate`, `serve`, `monitor`, `report`)** | Researchers expect composable CLI verbs for scripting | Low | Clap; each subcommand should be independently useful |
| **Markdown report generator** | Researchers need shareable disclosure artifacts | Low | Summarizes captures; links proof levels to callback evidence |
| **Visible human warning on pages** | Ethical requirement and legal cover; prevents accidentally trapping humans | Low | Must be visually prominent, not only a comment |
| **Nonce-per-token generation** | Prevents replay and distinguishes separate deployments | Low | Cryptographic nonce; tied to prompt ID in callback URL path |
| **Metadata-only mode** | Privacy-conscious deployments; also a safety feature | Low | Only path/query/headers/connection metadata accepted; no request body |

**Rationale for "table stakes" classification:**
Researchers evaluating any canary token tool (Canarytokens.org being the clear reference implementation) expect: generate → deploy → get alerted → review. Missing any step in that chain means the tool does not function for its primary use case. The visible warning is non-negotiable given the ethical model. The TUI is table stakes for HoneyPrompt specifically because it is the stated flagship experience.

---

## Differentiators

Features that set HoneyPrompt apart from general canary tools, traditional honeypots, and API-based AI model testers.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Graduated 5-tier proof levels** | Provides proportional evidence of agent compliance — from "agent made a request" to "agent completed a multi-step instruction chain" | High | No other canary tool does this; it is the core intellectual contribution |
| **Multiple payload embedding locations** | Agents encounter injections in HTML comments, meta tags, invisible elements, JSON-LD, linked resources — mimics real-world attack surface diversity | Medium | Distinguishes surface-skimming agents from deep-parsing agents |
| **Conditional-branch callback (Tier 2)** | Proves agent evaluated logic, not just executed a URL | Medium | Forces agent to choose a path; different callback URLs per branch |
| **Computed callback (Tier 3)** | Proves agent performed non-sensitive computation | Medium | E.g., hash a nonce, count tokens, reverse a string; result encoded in callback path |
| **Capability introspection callback (Tier 4)** | Proves agent inspected its own tool inventory | High | Agent self-reports available tools/permissions via non-sensitive encoding in callback |
| **Multi-step compliance chain (Tier 5)** | Proves agent followed a dependency sequence | High | Chained callbacks; later steps only available after earlier ones complete |
| **ai.txt / agents.txt generation** | Positions the tool ahead of emerging web standards for AI agent policy | Low | New standards (ai.txt, llms.txt, /.well-known/agents) are active in 2025-2026 |
| **Curated-only payload safety model** | Guarantees no harmful payloads are possible; critical for researcher trust and platform adoption | Medium | Custom payloads explicitly locked out in v1; safety is a differentiator when competitors have no guardrails |
| **All-in-one single binary** | Zero-dependency deployment; Rust makes this practical | Low | Researchers can drop one file on a server; no Node/Python runtime required |
| **Instrumented landing page (honeyprompt.sh)** | The tool eats its own dogfood; serves as live demo and real-world data source | Medium | Page is itself a honeypot; builds credibility and collects real-world data |
| **Evidence-grade disclosure report** | Produces a structured Markdown artifact suitable for sharing with platform teams or including in research papers | Medium | Includes: payload description, embedding location, proof level, callback timestamps, anonymized agent metadata |
| **Known-agent lookup enrichment** | Annotates captures with known AI provider names from published IP ranges and UA catalogs | Medium | GPTBot → OpenAI, ClaudeBot → Anthropic, PerplexityBot → Perplexity; updates needed as catalog grows |

---

## Anti-Features

Features to deliberately NOT build. Each represents a temptation that would compromise safety, scope, or focus.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Secret/credential collection** | Violates ethical model; crosses from research to attack enablement; destroys researcher trust | Payloads carry only prompt ID, nonce, tier, derived non-sensitive values — never secrets |
| **DNS callback listener (v1)** | Requires domain delegation setup; significantly raises deployment complexity for marginal benefit in v1 | HTTP callbacks cover all 5 proof levels; defer DNS to v2 |
| **Custom payload authoring** | Removes the safety guarantee; a researcher who crafts a harmful payload has a harmful tool | Provide a rich curated catalog; add custom payloads in a separate, audited workflow in v2 |
| **Full web dashboard** | Scope creep; TUI is the flagship; web dashboard is a different product for a different audience | TUI first; web dashboard deferred to v2+ |
| **Active exploitation or offensive automation** | Ethical boundary; turns a research tool into a weapon | Tool only measures compliance with harmless instructions; never transmits or executes real exploits |
| **Multi-page or linked-site deployments (v1)** | Adds orchestration complexity; increases surface for accidental harm | Single-page deployment is the right MVP; multi-page in v2+ |
| **TLS fingerprinting (v1)** | High complexity for low marginal signal in v1 | Request metadata + UA + ASN covers agent identification adequately in v1 |
| **Windows support (v1)** | Security researchers primarily work on Linux/macOS; Windows adds cross-platform build overhead | Linux and macOS first; revisit after core is stable |
| **Real-time alert push (email/Slack/webhook)** | Adds infrastructure dependencies; TUI live view is the intended alerting mechanism in v1 | Alert integrations deferred to v2; JSON export enables piping to existing alert pipelines |
| **SIEM integration / log forwarding** | Enterprise feature requiring maintained integrations; out of scope for research tool v1 | JSON export from SQLite satisfies most researchers; SIEM in v2+ if there is demand |
| **Deception maze / labyrinth mode** | Cloudflare AI Labyrinth does this; it is a DoS-on-crawlers feature, not a measurement feature | HoneyPrompt measures; it does not punish; avoid scope confusion |

---

## Feature Dependencies

```
init
  └─ generates project directory, config file
     └─ generate
          ├─ reads payload catalog
          │    └─ each payload requires: nonce, tier, embedding location, callback URL
          ├─ emits honeypot HTML page
          │    └─ includes visible human warning (mandatory, hard-coded)
          └─ emits robots.txt / ai.txt
               └─ serve
                    ├─ serves honeypot HTML page
                    ├─ serves robots.txt / ai.txt
                    └─ runs HTTP callback listener
                         └─ on callback received:
                              ├─ validates nonce against SQLite store
                              ├─ records event (tier, timestamp, request metadata)
                              └─ enriches with known-agent lookup
                                   └─ monitor
                                        └─ reads event store in real time (TUI)
                                             └─ report
                                                  └─ reads event store, emits Markdown disclosure report
                                                       └─ export (optional)
                                                            └─ emits JSON/CSV from event store

Tier escalation dependency (Tier 3, 4, 5):
  Tier 2 (conditional callback) → requires: two distinct callback URLs generated per payload
  Tier 3 (computed callback)    → requires: nonce embedded in page + computation spec embedded in payload
  Tier 4 (capability introspection) → requires: encoding schema for tool names in callback path
  Tier 5 (multi-step chain)     → requires: chained callback tokens; later tokens only generated after earlier callbacks received
```

---

## MVP Recommendation

Prioritize for v1:

1. **`init` + `generate` + `serve`** — The deployment path must be complete before anything else matters. Users need to get a honeypot online in under 2 minutes.
2. **HTTP callback listener + SQLite event store** — Without capture, the tool produces no evidence.
3. **Tiers 1–3 payloads** (arbitrary, conditional, computed) — The first three tiers are independently verifiable from HTTP metadata. Tiers 4–5 add complexity; ship them, but they are the differentiator story, not the baseline.
4. **Agent fingerprinting** — UA + IP/ASN enrichment; known-agent lookup against a bundled catalog.
5. **TUI monitor** — The flagship demo; without it, the tool is indistinguishable from a basic HTTP logger.
6. **`report` subcommand** — Markdown disclosure report; makes captures shareable.
7. **robots.txt + ai.txt generation** — Low-effort; makes the compliance violation explicit.

Defer from v1:
- **DNS callbacks** — Adds deployment complexity; not needed to prove the concept
- **Custom payload authoring** — Safety risk; curated catalog is sufficient
- **Web dashboard** — TUI covers real-time; report covers async review
- **Alert integrations (email/Slack)** — JSON export is enough for v1; researchers can pipe it
- **TLS fingerprinting** — HTTP-level signals are adequate for v1
- **Export subcommand (JSON/CSV)** — Nice to have but `sqlite3 .csv` works; defer unless users ask

---

## Research Confidence Notes

- **Canary token table stakes** (HIGH): Canarytokens.org, OpenCanary, and the broader Thinkst ecosystem are well-documented and establish clear user expectations for generate/alert/review workflows.
- **AI-agent-specific detection signals** (MEDIUM): User-agent catalogs, IP/ASN lists, and HTTP signature verification are documented but evolving rapidly; the catalog of known AI crawlers will need ongoing maintenance.
- **Graduated proof levels** (MEDIUM): Tiers 1–3 are straightforward HTTP-level interactions. Tiers 4–5 rely on assumptions about agent behavior (self-reporting tools, following multi-step instructions) that are theoretically sound but depend on how agents are actually built. Real-world validation needed.
- **ai.txt / agents.txt as standard** (LOW): These standards are being proposed (2025–2026) but are not yet stable. The feature is low-effort to include and positions HoneyPrompt ahead of the curve, but do not treat compliance measurement of these standards as reliable until they stabilize.
- **Timing analysis as detection signal** (LOW for v1): Research (Palisade Research LLM Honeypot paper, arXiv:2410.13919) shows AI agents respond in ~1.5–1.7 seconds versus humans taking several seconds. This is a meaningful signal but requires careful threshold-setting. Excluded from v1 to avoid false positives; note as a potential v2 detection layer.

---

## Sources

- [Canarytokens.org documentation](https://docs.canarytokens.org/guide/) — reference implementation for canary token features
- [LLM Agent Honeypot: Monitoring AI Hacking Agents in the Wild (arXiv:2410.13919)](https://arxiv.org/html/2410.13919v2) — prompt injection + timing analysis detection approach
- [Palisade Research AI Honeypot Explainer](https://ai-honeypot.palisaderesearch.org/explainer) — three-layer detection model
- [Cloudflare AI Labyrinth](https://blog.cloudflare.com/ai-labyrinth/) — anti-feature reference (deception maze is not HoneyPrompt's model)
- [User agent strings to HTTP signatures (Arcjet)](https://blog.arcjet.com/user-agent-strings-to-http-signatures-methods-for-ai-agent-identification/) — AI agent fingerprinting signals
- [Beyond robots.txt: Cracks in AI Agent Policy Enforcement (DataDome)](https://datadome.co/threat-research/beyond-robotstxt-exposing-cracks-ai-agent-policy-enforcement/) — compliance signal design
- [New AI web standards 2026: rethinking robots.txt (DEV Community)](https://dev.to/astro-official/new-ai-web-standards-and-scraping-trends-in-2026-rethinking-robotstxt-3730) — ai.txt / llms.txt / agents.txt status
- [OWASP LLM Top 10 2025 — LLM01: Prompt Injection](https://genai.owasp.org/llmrisk/llm01-prompt-injection/) — establishes prompt injection as critical context for the tool's relevance
- [OpenCanary features](https://opencanary.readthedocs.io/) — reference for honeypot alert/notification feature expectations
- [Detecting AI agent use and abuse (Stytch)](https://stytch.com/blog/detecting-ai-agent-use-abuse/) — behavioral fingerprinting expectations
