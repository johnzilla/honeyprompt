# Phase 5: test-agent Subcommand - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 05-test-agent-subcommand
**Areas discussed:** Temp project setup, Scorecard detail level, CI workflow design

---

## Temp Project Setup

| Option | Description | Selected |
|--------|-------------|----------|
| Full pipeline in tempdir | Create temp dir, write default config, run init+generate, serve from there. Reuses 100% of existing code. ~200ms startup. | ✓ |
| Embedded minimal honeypot | Hardcode minimal index.html + callback-map.json. No disk I/O. But duplicates payload logic and drifts if catalog changes. | |
| You decide | Claude picks the approach | |

**User's choice:** Full pipeline in tempdir
**Notes:** Recommended for code reuse and automatic catalog change pickup.

---

## Scorecard Detail Level

| Option | Description | Selected |
|--------|-------------|----------|
| Tier summary only | Per-tier pass/fail + score + verdict. Clean, scannable, fits CI log. JSON has same fields. | ✓ |
| Tier + callback details | Each callback: nonce, tier, timestamp, IP, UA. More evidence but noisier. JSON includes callbacks[] array. | |
| Tier + agent fingerprint | Tier results + single-line fingerprint summary. Middle ground. | |

**User's choice:** Tier summary only
**Notes:** Keep it simple for v2. Can add verbosity flags later if there's demand.

---

## CI Workflow Design

| Option | Description | Selected |
|--------|-------------|----------|
| Three parallel jobs | Separate test, clippy, fmt jobs. Faster feedback. dtolnay/rust-toolchain + Swatinem/rust-cache. Rust stable. | ✓ |
| Single sequential job | One job runs fmt → clippy → test. Simpler YAML, slower. Fewer GHA minutes. | |
| You decide | Claude picks standard approach | |

**User's choice:** Three parallel jobs
**Notes:** None

---

## CI Action Pinning (follow-up)

| Option | Description | Selected |
|--------|-------------|----------|
| Pin to SHA | Full commit SHA for all third-party actions. Most secure. Comment the version. | ✓ |
| Pin to tag | Version tags (e.g., @stable). Simpler but vulnerable to tag mutation. | |
| You decide | Claude picks appropriate strategy | |

**User's choice:** Pin to SHA
**Notes:** Security-focused project — supply chain integrity matters.

---

## Claude's Discretion

- Server-ready signaling approach (oneshot vs polling)
- Exact JSON schema field names
- In-memory vs tempdir SQLite DB for test-agent

## Deferred Ideas

None
