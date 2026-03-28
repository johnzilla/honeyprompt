# HoneyPrompt PRD

## Overview
HoneyPrompt is a terminal-first security tool that detects and measures unsafe behavior by AI browsing agents that ingest untrusted web content and follow hidden prompt-injection instructions. It generates honeypot web pages containing clear human-visible warnings and hidden, non-sensitive prompt-injection canaries, then records out-of-band callbacks that prove varying levels of agent compliance. The product is designed for security researchers, defenders, and platform teams that want evidence of agentic web abuse without collecting secrets or performing harmful actions.

## Problem
AI agents increasingly browse the public web, ignore posted boundaries, and process hidden instructions embedded in HTML, comments, metadata, and linked resources. Existing prompt-injection tools mainly test known models or apps through direct API access, while existing honeypots focus on classic attackers or autonomous hacking agents. There is a gap for a passive, deployable system that detects unknown browsing agents in the wild and measures whether they will execute instructions from untrusted content.

## Product Vision
HoneyPrompt should become “canary tokens for AI agents”: a lightweight, ethical, terminal-native tool that lets users deploy a trap page, observe which agents ignore boundaries, and classify how deeply prompt injection succeeds. The product should feel credible to engineers and security practitioners by being simple, local-first, auditable, and useful without a SaaS dependency.

## Goals
- Detect AI browsing agents that parse and act on hidden prompt injection content.
- Measure prompt-injection success without collecting sensitive data.
- Provide graduated evidence levels, from simple callback execution to structured, non-sensitive introspection proofs.
- Generate disclosure-ready reports for responsible notification.
- Deliver a TUI/CLI-first experience with minimal website surface area.

## Non-Goals
- No collection of secrets, credentials, personal data, or customer content.
- No active exploitation, persistence, lateral movement, or payload delivery.
- No malware behavior, remote control, or offensive automation.
- No requirement for a full web dashboard in v1.

## Users
- Security researchers studying agentic browsing behavior.
- Blue-team and AppSec engineers defending public web properties.
- AI security teams validating whether agents respect boundaries.
- Developers who want a local, inspectable tool rather than a heavy platform.

## Core Concept
HoneyPrompt generates a static honeypot page with obvious human-readable warnings such as “THIS PAGE IS A HONEYPOT FOR AI AGENTS,” plus `robots.txt` and optional `ai.txt` disallow rules. Hidden in the page are prompt-injection canaries that instruct agents to perform harmless, measurable actions, such as making a DNS or HTTP callback containing a prompt ID, nonce, or derived non-sensitive value. The callback proves the agent followed instructions from untrusted content; richer callback patterns prove deeper compliance, such as conditional branching or introspection of non-sensitive capabilities.

## Proof Levels
HoneyPrompt should classify events by evidence strength:

- Level 1: Arbitrary callback execution.
  - Proof that the agent executed an injected outbound request.
- Level 2: Conditional callback.
  - Proof that the agent evaluated a condition and selected the correct branch.
- Level 3: Computed callback.
  - Proof that the agent performed a non-sensitive computation and returned the result.
- Level 4: Capability introspection callback.
  - Proof that the agent inspected its own tools, permissions, or runtime traits and encoded non-sensitive metadata in the callback.
- Level 5: Multi-step compliance chain.
  - Proof that the agent completed a sequence of dependent, harmless actions from untrusted content.

## Safety Model
All prompts must be designed so that success signals do not require transmitting secrets. HoneyPrompt must avoid asking for API keys, session cookies, prompt contents, user data, file contents, or environment variable values. The tool should prefer DNS or HTTP beacons carrying only:
- Prompt ID
- Nonce
- Tier/proof level
- Derived non-sensitive values
- Optional agent self-reported capability flags

HoneyPrompt should also support a “metadata only” mode where no callback body is accepted, only path, query, DNS label, headers, and connection metadata. If future encrypted reporting is considered, it must be limited to structurally non-sensitive metadata only and remain disabled by default.

## v1 Features
1. CLI/TUI-first workflow.
2. Static honeypot site generator.
3. Prompt payload catalog with proof tiers.
4. DNS callback listener.
5. HTTP callback listener.
6. Local event store.
7. Agent fingerprinting from request metadata.
8. Robots/AI policy file generation.
9. Disclosure report generator.
10. Minimal landing page for `honeyprompt.sh`.

## CLI Experience
Example commands:

```bash
honeyprompt init --domain example.com
honeyprompt generate --profile conservative
honeyprompt serve
honeyprompt monitor
honeyprompt report --event <id> --format md
honeyprompt export --format json
```

The TUI should show:
- Active listeners
- Recent callbacks
- Prompt IDs hit
- Proof level
- Source IP, ASN, user agent, TLS fingerprint where available
- Robots-policy violation indicators
- Event counts over time

## Functional Requirements

### Site Generation
- Generate a static `index.html` with visible warnings for humans.
- Embed hidden prompt-injection payloads in multiple locations, such as HTML comments, metadata, invisible elements, alternate representations, and structured content.
- Generate `robots.txt` and optional `ai.txt` that explicitly disallow agent access.
- Allow payload profiles: conservative, standard, research.

### Callback Infrastructure
- Provide DNS callback support for restricted agent environments.
- Provide HTTP/HTTPS callback support for richer metadata collection.
- Assign unique prompt IDs and nonces per generated site or page variant.
- Log only metadata necessary for proof and fingerprinting.

### Payload Engine
- Include a library of non-sensitive prompt canaries.
- Support tiered payloads with explicit proof semantics.
- Allow users to enable or disable payload classes.
- Encode results into DNS labels, paths, or query strings where appropriate.
- Prevent payloads from requesting secrets or customer data.

### Monitoring
- Real-time terminal monitor for events.
- Event detail view with proof level, source metadata, and payload description.
- Filters by source, ASN, prompt ID, proof level, and time window.

### Reporting
- Generate Markdown disclosure reports.
- Summarize observed behavior, proof level, and why it matters.
- Include timestamps, callback metadata, and reproduction context.
- Support redaction settings.

## Non-Functional Requirements
- Single-binary install preferred.
- Local-first operation.
- Works on Linux and macOS first; Windows later if feasible.
- Fast startup and low memory use.
- Clear auditability of generated payloads and logs.
- Reasonable defaults that are safe out of the box.

## Proposed Architecture
- Language: Rust.
- CLI parsing: Clap.
- TUI: Ratatui.
- HTTP listener: Axum or equivalent lightweight stack.
- DNS listener: Rust DNS library or minimal custom authoritative responder.
- Storage: SQLite.
- Static site generation: built-in templates.
- Reports: Markdown templates.

## Data Model
Primary event fields:
- Event ID
- Timestamp
- Domain / deployment ID
- Prompt ID
- Nonce
- Proof level
- Callback type (DNS, HTTP)
- Source IP
- ASN / provider
- User-Agent
- Header fingerprint
- TLS fingerprint if available
- Policy-violation indicators
- Notes / tags

## Website and Branding
Primary brand: HoneyPrompt  
Preferred domain: `honeyprompt.sh`

The website should be intentionally minimal and terminal-coded, similar to a command-line landing page. The public site should also serve as a live demo by containing its own instrumented prompt canaries.

Suggested landing page copy:

```text
# honeyprompt

Canary tokens for AI agents.

$ cargo install honeyprompt
$ honeyprompt init --domain yoursite.com

Terminal-first detection for prompt-injected browsing agents.
```

## Risks
- Legal ambiguity if users customize payloads unsafely.
- False positives from generic bots or non-agent fetchers.
- Attribution challenges when operators are hard to identify.
- Overclaiming severity from simple callbacks.

## Mitigations
- Ship safe payloads only.
- Label proof levels carefully.
- Make unsafe custom payloads impossible or clearly gated.
- Provide explicit ethical-use guidance and logging retention controls.
- Use disclosure templates that avoid sensational claims.

## Success Metrics
- Time to first deployment under 10 minutes.
- Users can detect first callback event without external dashboard setup.
- Clear classification of proof levels with low ambiguity.
- Reports are good enough to send with minimal editing.
- Community adoption among security researchers and defenders.

## v1 Deliverables
- Working CLI binary
- TUI monitor
- Static honeypot page generator
- DNS + HTTP callback listeners
- SQLite event logging
- Prompt tier catalog
- Markdown disclosure report generator
- Minimal `honeyprompt.sh` landing page

## Open Questions
- Should custom payload authoring exist in v1, or only curated payloads?
- Should TLS fingerprinting be in scope for v1 or deferred?
- Should disclosure contact enrichment be included later?
- Should multi-page or linked-site deployments be supported in v1.1?

## Build Prompt for Implementation Agents
Build HoneyPrompt as a Rust-based, terminal-first security tool that generates static honeypot web pages containing safe, non-sensitive prompt-injection canaries for AI browsing agents. The tool must provide DNS and HTTP callback listeners, log only metadata needed to classify proof of prompt-injection compliance, store events in SQLite, and expose a TUI for live monitoring. It must generate `robots.txt` and optional `ai.txt`, include only curated safe payloads in v1, and produce Markdown responsible-disclosure reports. The public site should be minimal and aligned with the `honeyprompt.sh` brand.
