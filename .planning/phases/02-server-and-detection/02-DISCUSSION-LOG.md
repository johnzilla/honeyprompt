# Phase 2: Server and Detection - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 02-server-and-detection
**Areas discussed:** Event pipeline, Crawler catalog, Session model, Serve UX

---

## Event Pipeline

### How should events flow from callback handler to consumers?

| Option | Description | Selected |
|--------|-------------|----------|
| Broadcast (Recommended) | Handler → mpsc → Event Broker → broadcast → DB Writer + stdout logger | ✓ |
| Direct write | Handler writes directly to SQLite, stdout logger reads from DB | |
| You decide | Claude picks the architecture | |

**User's choice:** Broadcast
**Notes:** None

### What metadata should be extracted from each callback request?

| Option | Description | Selected |
|--------|-------------|----------|
| UA + IP + headers | User-Agent, source IP (proxy-aware), all HTTP headers | ✓ |
| ASN lookup | Map IP to ASN/provider name from bundled catalog | ✓ |
| Request timing | Record request timestamp + response latency | ✓ |
| TLS info | TLS version, cipher suite | ✓ |

**User's choice:** All four selected
**Notes:** TLS info is v2 per PROJECT.md scope — deferred, not included in Phase 2.

---

## Crawler Catalog

### How should the known-crawler catalog be stored?

| Option | Description | Selected |
|--------|-------------|----------|
| Embedded TOML | Bundled via rust-embed, same pattern as payload catalog | ✓ |
| JSON data file | External crawlers.json loaded at startup | |
| You decide | Claude picks format | |

**User's choice:** Embedded TOML
**Notes:** None

### How should crawlers be identified?

| Option | Description | Selected |
|--------|-------------|----------|
| UA string only | Match on User-Agent containing known bot names | |
| UA + IP ranges | UA matching plus known provider IP/ASN ranges | |
| UA primary, IP secondary | UA triggers label; IP/ASN adds confidence annotation | ✓ |

**User's choice:** UA primary, IP secondary
**Notes:** None

### What labels should identified crawlers get?

| Option | Description | Selected |
|--------|-------------|----------|
| Binary: crawler/agent | Simple: known_crawler or unknown | |
| Three-tier | known_crawler, known_agent, unknown | ✓ |
| You decide | Claude picks classification | |

**User's choice:** Three-tier
**Notes:** None

---

## Session Model

### How should sessions be assigned to group callbacks?

| Option | Description | Selected |
|--------|-------------|----------|
| IP + UA hash | Hash of source IP + UA → session ID | |
| IP + UA + time window | Same but sessions expire after timeout | ✓ |
| You decide | Claude picks strategy | |

**User's choice:** IP + UA + time window
**Notes:** None

### How should detection counts work?

| Option | Description | Selected |
|--------|-------------|----------|
| Unique sessions | Count distinct session IDs | |
| Sessions per tier | Count distinct sessions per proof tier | ✓ |
| You decide | Claude picks counting | |

**User's choice:** Sessions per tier
**Notes:** None

---

## Serve UX

### What should `honeyprompt serve` show on startup?

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal | Just bind address and Ctrl+C hint | |
| Detailed | Bind address, payloads, nonces, DB path, "ready" | ✓ |
| You decide | Claude picks | |

**User's choice:** Detailed
**Notes:** None

### How should events display in stdout before TUI exists?

| Option | Description | Selected |
|--------|-------------|----------|
| Structured log lines | One line per event | |
| JSON lines | One JSON object per event | |
| Both | Structured default, --json flag for JSON | ✓ |

**User's choice:** Both
**Notes:** None

### How should serve handle Ctrl+C?

| Option | Description | Selected |
|--------|-------------|----------|
| Graceful | Finish in-flight, flush DB, print stats, exit | ✓ |
| Immediate | Stop accepting, drop in-flight, exit fast | |
| You decide | Claude picks | |

**User's choice:** Graceful
**Notes:** None

---

## Claude's Discretion

- Event broker task implementation details
- ASN catalog contents and IP range data structure
- Session timeout duration
- Stdout log line format details
- Error handling for malformed callback paths

## Deferred Ideas

- TLS fingerprinting — v2 per PROJECT.md
