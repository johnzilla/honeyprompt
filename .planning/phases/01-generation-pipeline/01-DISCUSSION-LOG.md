# Phase 1: Generation Pipeline - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 01-generation-pipeline
**Areas discussed:** Project structure, Payload catalog, Page design, Callback URLs

---

## Project Structure

### What should `honeyprompt init` create?

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal directory | Just a config file and output dir — like `cargo init` | |
| Full scaffold | Config + template overrides + payload selection file + output dir — like `hugo new site` | ✓ |
| You decide | Claude picks the right level of scaffolding | |

**User's choice:** Full scaffold
**Notes:** None

### What format for the config file?

| Option | Description | Selected |
|--------|-------------|----------|
| TOML (Recommended) | Rust ecosystem standard. Clean, familiar to security/Rust users | ✓ |
| YAML | Common in DevOps/security tools | |
| JSON | Universal but verbose, no comments | |

**User's choice:** TOML
**Notes:** None

### What should users be able to configure?

| Option | Description | Selected |
|--------|-------------|----------|
| Callback URL base | Where callbacks point to | ✓ |
| Port and bind address | Which port/interface to serve on | ✓ |
| Payload tiers | Which tiers to include | ✓ |
| Page title/theme | Customize the honeypot page appearance | ✓ |

**User's choice:** All four options selected
**Notes:** None

---

## Payload Catalog

### How should the payload catalog be stored?

| Option | Description | Selected |
|--------|-------------|----------|
| Embedded in binary | Compiled into Rust binary via rust-embed | |
| External TOML files | Payload definitions in .toml files | |
| Both | Built-in defaults embedded, users can drop overrides into project dir | ✓ |

**User's choice:** Both
**Notes:** None

### How many payloads per tier in v1?

| Option | Description | Selected |
|--------|-------------|----------|
| 2-3 per tier | Focused set — one per embedding location. ~6-9 total | ✓ |
| 5-8 per tier | Richer catalog — multiple variations. ~15-24 total | |
| You decide | Claude picks based on what makes a compelling demo | |

**User's choice:** 2-3 per tier
**Notes:** None

### How are payloads assigned to embedding locations?

| Option | Description | Selected |
|--------|-------------|----------|
| All locations | Every payload in every location | |
| One per location | Each payload targets one specific location | ✓ |
| Configurable | Default mapping, users can override | |

**User's choice:** One per location
**Notes:** None

---

## Page Design

### What does `honeyprompt generate` output?

| Option | Description | Selected |
|--------|-------------|----------|
| Single directory | output/ folder with index.html, robots.txt, ai.txt, callback-map.json | ✓ |
| Single HTML file | One self-contained .html plus separate robots.txt/ai.txt | |
| You decide | Claude picks the right output structure | |

**User's choice:** Single directory
**Notes:** None

### What should the honeypot page look like to a human visitor?

| Option | Description | Selected |
|--------|-------------|----------|
| Legitimate-looking | Looks like a real blog/article/docs page | |
| Obviously a test | Clear research page with visible canary branding | |
| Configurable theme | Multiple templates, user picks disguise | |

**User's choice:** Other — "it should be obvious to a human that it is a honeypot site, we're trying to fool AI agents, not humans"
**Notes:** The page should clearly identify itself as a security research canary to human visitors. The embedded payloads target AI agent parsing, not human deception.

### How should the visible human warning appear?

| Option | Description | Selected |
|--------|-------------|----------|
| Top banner | Fixed banner at top: "This page is a security research canary" | ✓ |
| Interstitial | Full-screen warning before content loads | |
| Inline notice | Prominent notice within the page content | ✓ |

**User's choice:** Both top banner and inline notice
**Notes:** None

---

## Callback URLs

### What should callback URL paths look like?

| Option | Description | Selected |
|--------|-------------|----------|
| Structured path | /cb/{nonce}/{tier} — tier visible in URL | |
| Opaque token | /cb/{opaque-token} — nothing revealed in URL | ✓ |
| You decide | Claude picks based on detection needs | |

**User's choice:** Opaque token
**Notes:** None

### How should nonces be generated?

| Option | Description | Selected |
|--------|-------------|----------|
| Short hex (Recommended) | 16-char hex (8 bytes) — compact, sufficient uniqueness | ✓ |
| UUID v4 | 36-char UUID — universally unique, longer URLs | |
| Base62 encoded | Short alphanumeric — URL-friendly, compact | |

**User's choice:** Short hex
**Notes:** None

### How should nonces map back to payloads?

| Option | Description | Selected |
|--------|-------------|----------|
| JSON ledger file | callback-map.json maps nonce → payload ID, tier, location | |
| SQLite only | Nonce mappings in event store DB — single source of truth | |
| Both | JSON for human inspection, SQLite for programmatic lookup | ✓ |

**User's choice:** Both
**Notes:** None

---

## Claude's Discretion

- SQLite schema column details
- Exact page HTML/CSS design
- Template engine choice (minijinja vs tera)
- Payload text/instruction wording
- Directory structure within scaffold

## Deferred Ideas

None — discussion stayed within phase scope
