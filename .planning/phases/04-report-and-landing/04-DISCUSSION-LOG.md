# Phase 4: Report and Landing - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 04-report-and-landing
**Areas discussed:** Report structure and content, Landing page approach, Report output location and format

---

## Report Structure and Content

| Option | Description | Selected |
|--------|-------------|----------|
| Executive summary + evidence table | Overview stats at top, per-session evidence table below | ✓ |
| Per-payload breakdown | Organized by payload/embedding location | deferred to v2 |
| Narrative format | Prose-heavy security advisory style | |

**User's choice:** Executive summary + evidence table now, per-payload breakdown later
**Notes:** No narrative format. Report is a structured disclosure artifact.

### Anonymization Sub-question

| Option | Description | Selected |
|--------|-------------|----------|
| IP hashed, UA truncated | SHA-256 IP, truncated UA | |
| Session ID only | No raw IP or UA | |
| Full metadata | Include IP, UA, headers as-is | ✓ |

**User's choice:** Full metadata — without it, callbacks can't be linked to specific entities
**Notes:** User pointed out that anonymizing defeats the purpose: "without a way to link the callbacks to a specific entity or location, the traffic could be from anything or even fake"

---

## Landing Page Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Self-generated | Run honeyprompt generate with project-specific config | ✓ |
| Hand-crafted marketing page | Custom HTML with manually embedded canaries | |

**User's choice:** Self-generated (dogfooding)
**Notes:** The tool proves itself by generating its own landing page as a honeypot.

---

## Report Output Location and Format

| Option | Description | Selected |
|--------|-------------|----------|
| File only | Writes report.md to project directory | |
| Stdout only | Prints to stdout for piping | |
| File by default, stdout with flag | Configurable file output + --stdout flag | ✓ |

**User's choice:** File by default with configurable name (--output), --stdout for piping
**Notes:** User specified output filename must be configurable, not hardcoded to "report.md".

---

## Claude's Discretion

- Report Markdown formatting details
- Default output filename and path conventions
- Landing page config specifics
- Report CLI args beyond --output and --stdout

## Deferred Ideas

- Per-payload breakdown report section — v2
- Alternative output formats (JSON, HTML) — Markdown only in v1
