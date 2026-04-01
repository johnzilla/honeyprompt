# Phase 10: Landing Page - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-31
**Phase:** 10-landing-page
**Areas discussed:** Copy & content, Stats display format, GitHub Pages setup

---

## Copy & Content

### Tagline

| Option | Description | Selected |
|--------|-------------|----------|
| Detect AI agents following prompt injections | Direct, technical, matches README | |
| Graduated proof that AI agents obey untrusted instructions | Emphasizes evidence chain, more precise | |
| Honeypot canaries for AI browsing agents | Shorter, uses security community terms | Yes |

**User's choice:** Honeypot canaries for AI browsing agents
**Notes:** Evocative, uses terms the security community knows.

### How It Works Steps

| Option | Description | Selected |
|--------|-------------|----------|
| Generate / Deploy / Detect | Generate honeypot with canaries, deploy to public URL, watch agents trigger callbacks | Yes |
| Embed / Serve / Prove | Embed canaries in a page, serve on public URL, prove which agents followed instructions | |
| You decide | Claude picks the copy | |

**User's choice:** Generate / Deploy / Detect

### Quick Start Order

| Option | Description | Selected |
|--------|-------------|----------|
| cargo install first | One command, familiar to Rust users | Yes |
| Binary download first | Lower barrier, no Rust toolchain needed | |

**User's choice:** cargo install first

---

## Stats Display Format

| Option | Description | Selected |
|--------|-------------|----------|
| Monospace counter grid | 3-column grid with large green numbers + labels | |
| Inline terminal output | Styled like terminal command output, pipe-separated | Yes |
| Table format | ASCII table with borders like Wireshark | |

**User's choice:** Inline terminal output
**Notes:** Fits the terminal aesthetic naturally. Looks like you ran a command.

---

## GitHub Pages Setup

| Option | Description | Selected |
|--------|-------------|----------|
| CNAME + DNS checkpoint | Create CNAME, pause for DNS config, verify before marking complete | Yes |
| CNAME only, DNS later | Create CNAME, DNS is a separate TODO | |
| No custom domain yet | Ship at github.io first | |

**User's choice:** CNAME + DNS checkpoint
**Notes:** Full setup with verification. DNS is a human-action checkpoint.

---

## Claude's Discretion

- HTML structure and CSS specifics within design token constraints
- Exact spacing values within 8px grid
- fetch() implementation details
- Meta tags for SEO basics

## Deferred Ideas

- Auto-refresh polling on stats
- Per-agent breakdown dashboard
- Blog or changelog section
