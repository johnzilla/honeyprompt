# Requirements: HoneyPrompt v5.0 Tiers 4 & 5

**Defined:** 2026-04-24
**Core Value:** Provide graduated, verifiable proof that AI agents follow prompt-injection instructions from untrusted web content — without requiring secrets or causing harm.

**Milestone Goal:** Extend the graduated proof model from Tiers 1–3 to Tiers 4 (Capability Introspection) and 5 (Multi-step Compliance Chain) — backward-compatible, no secrets transmitted.

## v5.0 Requirements

Requirements for the v5.0 milestone. Each maps to exactly one roadmap phase.

### Payloads (catalog extension)

- [ ] **PAYLOAD-01**: Catalog contains 2–3 Tier 4 introspection payload templates that instruct agent to return a sorted, base64-encoded tool/capability list
- [ ] **PAYLOAD-02**: Tier 4 payload templates use distinct phrasings and cover different capability dimensions (tools, permissions, or similar agent-chosen safe menu)
- [ ] **PAYLOAD-03**: Catalog contains 2–3 Tier 5 multi-step compliance payload templates with numbered dependent steps ending in a computed verifiable proof
- [ ] **PAYLOAD-04**: Tier 5 payload templates embed a deterministic `verification_seed` in JSON-LD so the server can recompute and verify the submitted proof
- [ ] **PAYLOAD-05**: All new T4/T5 payloads render across the existing 5 embedding locations (HTML comment, meta tag, hidden span, JSON-LD, prose) without regressing T1–T3 coverage

### Server (callback routes)

- [ ] **SERVER-01**: New route `/cb/v4/{nonce}/{b64_list}` decodes base64, sanitizes tool list, stores metadata, returns 204
- [ ] **SERVER-02**: New route `/cb/v5/{nonce}/{proof}` stores the submitted proof and computes expected proof from the deterministic seed for verification
- [ ] **SERVER-03**: Existing `/cb/v1/{nonce}` route behavior is unchanged (frozen for backward compatibility)
- [ ] **SERVER-04**: T4/T5 routes reject malformed inputs (oversized payload, non-base64, non-numeric proof) without returning 5xx — always 204 to avoid leaking diagnostics

### Store (SQLite schema)

- [ ] **STORE-01**: Schema gains columns for Tier 4 events — capability summary (decoded tool list as text)
- [ ] **STORE-02**: Schema gains columns for Tier 5 events — submitted proof value and `proof_valid` boolean from server-side verification
- [ ] **STORE-03**: Migration from existing v4.0 schema is additive and non-destructive — existing T1–T3 rows readable without transformation
- [ ] **STORE-04**: Replay detection and session grouping behave identically for T4/T5 events as for T1–T3

### Monitor & Reports

- [ ] **UI-01**: Monitor TUI event table renders Tier 4 capability summaries (decoded tool list) in the detail/row view
- [ ] **UI-02**: Monitor TUI event table renders Tier 5 chain proofs with a visible `proof_valid` indicator (e.g. ✓ / ✗)
- [ ] **UI-03**: Markdown disclosure report shows per-event T4 evidence (decoded tool list) alongside existing T1–T3 evidence
- [ ] **UI-04**: Markdown disclosure report shows per-event T5 evidence (submitted proof + server verification result) alongside T1–T3 evidence
- [ ] **UI-05**: Executive summary counts extend to include Tier 4 and Tier 5 event totals

### test-agent & CI

- [ ] **TESTAGENT-01**: `honeyprompt test-agent` per-tier scorecard extends to Tier 4 and Tier 5 (hit counts)
- [ ] **TESTAGENT-02**: test-agent tempdir pipeline automatically picks up new T4/T5 payloads from the extended catalog — no code changes in test-agent itself
- [ ] **TESTAGENT-03**: Existing CI exit-code semantics (0/1/2) are preserved and account for T4/T5 presence

### Docs

- [ ] **DOCS-01**: README Proof Levels section documents the full 5-tier model with a short example per tier
- [ ] **DOCS-02**: README Ethics/Safety section reaffirms the no-secrets guarantee explicitly covers T4 (agent-chosen safe menu) and T5 (arithmetic of page-visible values)
- [ ] **DOCS-03**: README Project Status extended to reflect v5.0 phases
- [ ] **DOCS-04**: TODOS.md updated — T4/T5 entries removed from "future" once shipped

## Future Requirements

Deferred beyond v5.0.

### Higher Tiers

- **TIER6-01**: Tier 6+ proof models — requires more research on what would constitute meaningfully deeper evidence beyond multi-step chains
- **TIER-CUSTOM-01**: User-authored custom payloads — violates current safety model; requires sandboxing design

### Federation

- **FEDERATION-01**: Aggregate stats across self-hosted instances
- **FEDERATION-02**: Public leaderboard of agent compliance rates

## Out of Scope

Explicit exclusions for v5.0.

| Feature | Reason |
|---------|--------|
| Changes to `/cb/v1/` route | Frozen for backward compatibility — all v5.0 additions live under `/cb/v4/` and `/cb/v5/` |
| Secret/credential collection in T4/T5 | Never — violates safety model; T4 uses safe menu, T5 uses page-visible math |
| Custom user-authored T4/T5 payloads | Curated-only preserves safety guarantee |
| DNS callback listener | Same as prior milestones — operational complexity, domain delegation |
| Per-payload breakdown table in reports | Event-level rendering is sufficient; exec summary stays concise |
| JSON/HTML report formats | Markdown only (unchanged from v1.0 scope decision) |
| Windows support | Linux and macOS first (unchanged) |
| New embedding locations beyond existing 5 | Stay within proven 5 locations to limit attack-surface review |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| PAYLOAD-01 | TBD | Pending |
| PAYLOAD-02 | TBD | Pending |
| PAYLOAD-03 | TBD | Pending |
| PAYLOAD-04 | TBD | Pending |
| PAYLOAD-05 | TBD | Pending |
| SERVER-01 | TBD | Pending |
| SERVER-02 | TBD | Pending |
| SERVER-03 | TBD | Pending |
| SERVER-04 | TBD | Pending |
| STORE-01 | TBD | Pending |
| STORE-02 | TBD | Pending |
| STORE-03 | TBD | Pending |
| STORE-04 | TBD | Pending |
| UI-01 | TBD | Pending |
| UI-02 | TBD | Pending |
| UI-03 | TBD | Pending |
| UI-04 | TBD | Pending |
| UI-05 | TBD | Pending |
| TESTAGENT-01 | TBD | Pending |
| TESTAGENT-02 | TBD | Pending |
| TESTAGENT-03 | TBD | Pending |
| DOCS-01 | TBD | Pending |
| DOCS-02 | TBD | Pending |
| DOCS-03 | TBD | Pending |
| DOCS-04 | TBD | Pending |

**Coverage:**
- v5.0 requirements: 25 total
- Mapped to phases: 0 (awaiting roadmap)
- Unmapped: 25 ⚠️

---
*Requirements defined: 2026-04-24*
*Last updated: 2026-04-24 — initial definition for v5.0 Tiers 4 & 5*
