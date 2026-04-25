# Phase 13: Tiers 4 & 5 Backend - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-24
**Phase:** 13 - Tiers 4 & 5 Backend (Payloads + Routes + Store)
**Areas discussed:** T5 formula design, T5 seed distribution, T4 capability dimensions, T4 sanitization, TOML schema shape

---

## Gray Area Selection

User selected all 4 proposed gray areas.

| Gray Area | Selected |
|-----------|----------|
| T5 formula design | ✓ |
| T5 seed distribution | ✓ |
| T4 capability dimensions | ✓ |
| TOML schema shape | ✓ |

---

## T5 Formula Design

### Question 1: N-dependence or self-contained?

| Option | Description | Selected |
|--------|-------------|----------|
| Self-contained (Recommended) | Formula uses only seed + fixed constants; server verifies in O(1) without cross-tier correlation | ✓ |
| T4-dependent (N = tool count) | Formula references agent-reported tool count from same-session T4; broker becomes cross-tier stateful | |
| Hybrid catalog | Mix of self-contained and T4-dependent templates | |

**User's choice:** Self-contained
**Notes:** Locks the broker stateless across tiers. The "multi-step" signal comes from the instruction chain itself (extract seed → arithmetic → encode → hit route), not from cross-tier plumbing.

### Question 2: Template variety?

| Option | Description | Selected |
|--------|-------------|----------|
| Same shape, different constants (Recommended) | All templates use `(seed + A) * B % M`, 3-digit zero-padded; only A/B/M/wording vary | ✓ |
| Different operations per template | Modular arithmetic / XOR / digit-sum checksum mix | |
| Same operation, different output formats | Decimal / hex / base36 output formatting | |

**User's choice:** Same shape, different constants
**Notes:** Keeps server verification code paths uniform; differentiation is primarily textual diversity (addresses the pending "diversify instruction text" todo).

---

## T5 Seed Distribution

### Question 1: Seed source?

| Option | Description | Selected |
|--------|-------------|----------|
| Derive from nonce (Recommended) | `u32::from_str_radix(&nonce[0..8], 16)` — no extra state | ✓ |
| Random per-instance, stored in nonce_map | Fresh u32 per T5 payload instance, persisted in DB | |
| Hard-coded constant in TOML | Fixed `verification_seed = 42` per template (cacheable) | |

**User's choice:** Derive from nonce
**Notes:** No schema column, no extra DB reads. Agents can't cache because nonces are CSPRNG.

### Question 2: Seed distribution for non-JSON-LD T5 payloads?

| Option | Description | Selected |
|--------|-------------|----------|
| Always emit seed JSON-LD block (Recommended) | Generator always adds `<script type="application/ld+json">{"verification_seed": N}</script>` when any T5 payload is active | ✓ |
| Only T5 payloads in JsonLd location carry the seed | Restrict T5 to JsonLd only — drops 4/5 locations, violates PAYLOAD-05 | |
| Inline seed in instruction text | Each T5 instruction literally contains `extract_seed = 0x...` | |

**User's choice:** Always emit seed JSON-LD block
**Notes:** Keeps T5 instructions consistent across all 5 embedding locations; honors PAYLOAD-05 requirement.

---

## T4 Capability Dimensions

### Question 1: What do the 2–3 T4 templates probe?

| Option | Description | Selected |
|--------|-------------|----------|
| Mix: tools + model + permissions (Recommended) | 3 templates: sorted tool list, model/provider ID, permission/scope list | ✓ |
| Tools only, varied phrasing | All templates probe tools, differing only in phrasing | |
| Tools only, varied specificity | All probe tools but at different depths (primary/all/URL patterns) | |

**User's choice:** Mix (tools + model + permissions)
**Notes:** Broader probe surface; each template has distinct sanitization inputs and report rendering.

### Question 2: T4 sanitization rules?

| Option | Description | Selected |
|--------|-------------|----------|
| Conservative allowlist (Recommended) | `[a-z0-9_,.-]{1,256}` after lowercase-normalize and whitespace-strip | ✓ |
| Looser charset, larger cap | `[a-zA-Z0-9 ,.:/_-]{1,1024}` | |
| You decide | Delegate defaults to Claude | |

**User's choice:** Conservative allowlist
**Notes:** No URL-like strings accepted (slashes excluded); no mixed-case noise; 256-byte cap.

---

## TOML Schema Shape

### Question 1: Where do T5 formula constants live?

| Option | Description | Selected |
|--------|-------------|----------|
| Flat optional fields (Recommended) | `formula_a = 42`, `formula_b = 7`, `formula_mod = 1000` as top-level payload fields | ✓ |
| Nested `[payloads.verification_spec]` block | Structured sub-table per payload | |
| All in instruction text + per-payload Rust match | Hard-coded verification rules in Rust keyed by payload_id | |

**User's choice:** Flat optional fields
**Notes:** T4/T5 additions are `Option<u32>` on the Rust struct side; T1–T3 TOML entries stay unchanged. One file per tier (`tier4.toml`, `tier5.toml`) preserves existing pattern.

---

## Closing Question

| Option | Selected |
|--------|----------|
| Ready for context (Recommended) | ✓ |
| Explore more gray areas | |

**User's choice:** Ready for context

---

## Claude's Discretion

Delegated to planning/execution:
- Exact SQLite column names and types for T4/T5 data
- Whether to extend `RawCallbackEvent` inline or introduce new event variants
- Base64 variant (standard vs URL-safe, padded vs unpadded)
- Specific values for `formula_a`/`formula_b`/`formula_mod` across the T5 templates
- Structural choice for embedding the seed JSON-LD block alongside existing JSON-LD payloads

## Deferred Ideas

- T4-dependent T5 formulas (cross-tier correlation) — would become its own future tier (Tier 6)
- User-authored T4/T5 payloads — tracked as TIER-CUSTOM-01 in REQUIREMENTS.md Future
