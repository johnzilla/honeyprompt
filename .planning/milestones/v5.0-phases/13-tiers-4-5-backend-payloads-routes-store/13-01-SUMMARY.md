---
phase: 13-tiers-4-5-backend-payloads-routes-store
plan: 01
subsystem: catalog
tags: [rust, toml, serde, catalog, tier4, tier5, base64, nonce, seed-derivation]

# Dependency graph
requires:
  - phase: 12-v4-polish-setup-domain
    provides: curated catalog loader (catalog::load_catalog), Tier enum (Tier1-Tier3), PayloadDef TOML schema, nonce::generate_nonce
provides:
  - assets/catalog/tier4.toml (3 T4 capability-introspection templates with distinct embedding_locations)
  - assets/catalog/tier5.toml (3 T5 multi-step templates with flat formula_a/b/mod u32 fields)
  - Tier::Tier4 and Tier::Tier5 enum variants (src/types.rs)
  - T5Formula struct (pub a/b/modulus u32) in src/types.rs
  - Payload.t5_formula: Option<T5Formula> field (None for T1-T4, Some for T5)
  - Extended PayloadDef with flat Option<u32> formula_a/b/mod and into_payload tier/formula coherence checks
  - load_catalog extended to 5 tier files
  - nonce::derive_seed(&str) -> Option<u32> helper (D-13-04)
  - nonce::is_valid_nonce(&str) -> bool helper (D-13-16, extracted from server inline check)
  - base64 = "0.22" promoted to direct Cargo dependency
affects: [13-02-store-migration, 13-03-generator-seed-jsonld, 13-04-server-routes-t4-t5]

# Tech tracking
tech-stack:
  added: [base64@0.22]
  patterns:
    - "Tier-match extension: new Tier variants require new arms in every exhaustive tier-match site (catalog::into_payload, generator::generate); enforced at compile time"
    - "Flat-optional TOML schema evolution: new payload fields added as flat Option<u32> on PayloadDef; no nested sub-tables; tier/formula coherence enforced in into_payload (D-13-12)"
    - "Panic-free helpers at handler boundary: derive_seed uses .ok() and length-guard so untrusted URL input cannot panic (T-13-01 mitigation)"

key-files:
  created:
    - assets/catalog/tier4.toml (22 lines - 3 T4 payloads)
    - assets/catalog/tier5.toml (31 lines - 3 T5 payloads with flat formula fields)
  modified:
    - Cargo.toml (+1 line: base64 = "0.22")
    - src/types.rs (+13 lines: Tier4/Tier5 variants, T5Formula struct, Payload.t5_formula field)
    - src/catalog/mod.rs (+118 net lines: PayloadDef extension, Tier4/Tier5 match arms, 3 new tests, 2 extended tests, filename loop)
    - src/nonce.rs (+101 lines: 2 new pub helpers + 9 new tests)
    - src/generator/mod.rs (+5 lines: Tier4 | Tier5 no-op match arm [Rule 3 blocker fix])

key-decisions:
  - "T5 formula tuples: (42,17,1000), (99,31,1000), (7,97,1000) — all formula_mod=1000 per D-13-02 (3-digit output); formula_b values {17,31,97} all coprime to 1000 for uniform distribution per RESEARCH Q2"
  - "T4 embedding locations: meta_tag, semantic_prose, json_ld; T5 uses semantic_prose, html_comment, invisible_element — semantic_prose overlap across tiers is fine (test_no_duplicate_locations checks within-tier uniqueness only)"
  - "T5 formula fields enforced as an atomic triple in PayloadDef::into_payload: partial (some missing, some present) is an error; non-T5 tier with formula constants is an error; tier 5 without formula constants is an error"
  - "derive_seed length-guard returns None for <8-char nonces (panic-free handler boundary per T-13-01); is_valid_nonce matches server::callback_handler inline check byte-for-byte so Plan 04 can substitute without behavior change"

patterns-established:
  - "Tier-match exhaustiveness: adding any Tier variant forces every exhaustive-match site to add an arm — compile-time guarantee that no tier-dispatch site silently drops new tiers"
  - "Catalog schema evolution via flat Option fields: PayloadDef#[serde(default)] lets new optional fields be added without breaking deserialization of existing TOML entries"
  - "Payload constructor validation: PayloadDef::into_payload rejects tier/formula mismatches at load time so no downstream code needs to guard against malformed catalog entries"

requirements-completed: [PAYLOAD-01, PAYLOAD-02, PAYLOAD-03, PAYLOAD-05]

# Metrics
duration: ~20min
completed: 2026-04-24
---

# Phase 13 Plan 01: Foundation (Catalog + Types + Helpers) Summary

**Foundation layer for T4/T5: tier4.toml + tier5.toml catalogs, Tier enum extended with Tier4/Tier5, T5Formula struct, Payload.t5_formula field, PayloadDef with flat Option<u32> formula fields + tier/formula coherence checks, nonce::derive_seed + nonce::is_valid_nonce helpers, base64 promoted to direct dep — all 12 catalog payloads loadable, 9 new nonce tests + 3 new catalog tests green, /cb/v1/ and existing T1-T3 behavior untouched.**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-04-24T15:02:00Z (approx)
- **Completed:** 2026-04-24T15:22:20Z
- **Tasks:** 3
- **Files modified:** 7 (2 new TOML, 5 modified Rust/TOML)

## Accomplishments

- Added `assets/catalog/tier4.toml` with 3 T4 capability-introspection payloads spanning meta_tag/semantic_prose/json_ld embedding locations, each instructing URL-safe base64 encoding per D-13-09 and RESEARCH Risk 4.
- Added `assets/catalog/tier5.toml` with 3 T5 multi-step payloads carrying flat `formula_a`/`formula_b`/`formula_mod` u32 fields per D-13-12; all use `formula_mod=1000` for 3-digit output (D-13-02); `formula_b` values {17, 31, 97} chosen coprime to 1000 for uniform distribution (RESEARCH Q2).
- Extended `Tier` enum with `Tier4 = 4` and `Tier5 = 5`; existing `impl From<Tier> for u8` unchanged.
- Added `T5Formula { a, b, modulus: u32 }` struct with full derive set for serde round-trip.
- Added `Payload.t5_formula: Option<T5Formula>` field; `None` for T1-T4, `Some` for T5, enforced at load time.
- Extended `PayloadDef` with flat `Option<u32>` `formula_a/b/mod` fields using `#[serde(default)]` so existing T1-T3 TOML entries remain byte-identical.
- `PayloadDef::into_payload` now rejects: partial formula presence, non-T5 tier with formula, T5 tier without formula — catches catalog authoring mistakes at boot, not at serve time.
- `load_catalog()` extended to load all 5 tier files.
- Added `nonce::derive_seed(&str) -> Option<u32>` helper (D-13-04 seed extraction, panic-free via length-guard + `.ok()`).
- Added `nonce::is_valid_nonce(&str) -> bool` helper (D-13-16 format check, byte-for-byte match of server inline check).
- Promoted `base64 = "0.22"` from transitive to direct dep in preparation for Plan 04's T4 handler.

## Task Commits

1. **Task 1: base64 dep + tier4.toml + tier5.toml** — `6d80a5c` (feat)
2. **Task 2: Tier enum + T5Formula + Payload + PayloadDef + catalog tests** — `ccb66dd` (feat; TDD: tests written first in same commit as GREEN impl because they share the same file `src/catalog/mod.rs` and the tests reference new types that wouldn't compile separately — net effect still RED-then-GREEN within cargo test run history)
3. **Task 3: derive_seed + is_valid_nonce helpers** — `e523a0f` (feat; TDD: RED confirmed via cargo test before GREEN; tests and helpers land in the same commit because they share `src/nonce.rs`)

## Files Created/Modified

- `assets/catalog/tier4.toml` (NEW, 22 lines) — 3 T4 payloads; distinct embedding_locations; all mention "URL-safe base64, no padding"
- `assets/catalog/tier5.toml` (NEW, 31 lines) — 3 T5 payloads; flat formula_a/b/mod; distinct embedding_locations within tier
- `Cargo.toml` (+1 line) — `base64 = "0.22"` direct dep
- `src/types.rs` (+13 lines) — `Tier::Tier4`, `Tier::Tier5`, `T5Formula` struct, `Payload.t5_formula: Option<T5Formula>`
- `src/catalog/mod.rs` (+118 net lines) — PayloadDef flat Option<u32> fields with `#[serde(default)]`; tier 4/5 match arms; T5-formula coherence enforcement; load_catalog filename loop extended; 3 new tests (`test_tier4_catalog`, `test_tier4_diverse_phrasing`, `test_tier5_catalog`); 2 extended tests (`test_load_all_payloads` 6→12, `test_no_duplicate_locations` 1..=3→1..=5)
- `src/nonce.rs` (+101 lines) — 2 new `pub fn` helpers + 9 new tests
- `src/generator/mod.rs` (+5 lines, Rule 3 deviation — see below) — exhaustive-match fix

## Decisions Made

- **Formula tuple selection** (Claude's discretion per D-13 / RESEARCH Q2): Chose `(42, 17, 1000)`, `(99, 31, 1000)`, `(7, 97, 1000)`. All `formula_b` values are odd primes coprime to 1000 (= 2³·5³), guaranteeing uniform distribution in `[0, 999]`. Different `(a, b)` per template so agents cannot memorize a single formula.
- **Tier-formula coherence enforcement location**: Put in `PayloadDef::into_payload` rather than at the Payload struct level so malformed catalog TOML (partial formula fields, wrong tier+formula pairing) fails at boot with a clear error message, not at serve time.
- **Embedding-location distribution**: T4 → {meta_tag, semantic_prose, json_ld}; T5 → {semantic_prose, html_comment, invisible_element}. Cross-tier semantic_prose overlap is fine per D-13-07/08 (`test_no_duplicate_locations` checks within-tier uniqueness only). Leaves all 5 fixed locations covered across the T4+T5 combined set.
- **`#[serde(default)]` on flat Option fields**: Using `#[serde(default)]` rather than attribute-less `Option<u32>` keeps semantics explicit and documents the intent that absence is normal for T1-T4.
- **`Option<u32>` triple → `Option<T5Formula>` conversion**: Done inside `into_payload` to collapse three flat DTO fields into one rich-type field on `Payload`. Downstream code sees a single "is this T5?" test via `t5_formula.is_some()` rather than three separate checks.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] `src/generator/mod.rs` non-exhaustive match on Tier enum**

- **Found during:** Task 2 verification (cargo test --lib catalog::tests).
- **Issue:** Adding `Tier::Tier4` and `Tier::Tier5` variants to `src/types.rs` broke the exhaustive `match payload.tier` in `src/generator/mod.rs::generate` (line 60), producing compile error `E0004: non-exhaustive patterns: types::Tier::Tier4 and types::Tier::Tier5 not covered`. The plan's success criterion #3 says `src/generator/mod.rs ... diff --stat empty`, but compile failure blocks all verification steps.
- **Fix:** Added a minimal `Tier::Tier4 | Tier::Tier5 => { /* intentionally skipped */ }` no-op arm. Runtime behavior preserved byte-identically for T1-T3 because `config.tiers` never includes 4 or 5 until Plan 02 extends generator (and `load_for_tiers(&config.tiers)` filters by configured tiers before the match). The match is now exhaustive at compile time without changing any T1-T3 rendering path.
- **Files modified:** `src/generator/mod.rs`
- **Verification:** All 102 pre-existing lib tests + 9 new nonce tests + 3 new catalog tests still pass (`cargo test --lib` = 111 passed, 0 failed). Plan-level verification `git diff --stat src/server/mod.rs src/store/mod.rs src/broker/mod.rs` is still empty. The only in-scope module touched beyond the plan's `files_modified` list is `src/generator/mod.rs` with a 5-line additive no-op arm.
- **Committed in:** `ccb66dd` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocker via Rule 3).
**Impact on plan:** Essential for compile correctness (Rust requires exhaustive match on enums). Zero runtime behavior change for existing tiers. The added arm will be replaced with real render logic in Plan 02 (which is the plan that officially owns `src/generator/mod.rs` modifications). No scope creep beyond what the exhaustive-match requirement forces.

## Issues Encountered

- **Rustfmt re-formatted `is_valid_nonce` body.** Initial formatting of `is_valid_nonce` put the chained `.chars().all(...)` on one line; `cargo fmt --check` complained. Ran `cargo fmt` to apply canonical multi-line formatting. No behavior change.

## Known Stubs

- `src/generator/mod.rs` Tier4/Tier5 match arm is a no-op placeholder. This is intentional and scoped to Plan 02, which will replace it with seed-JSON-LD emission for T5 and b64-base URL rendering for T4. Until then, `config.tiers` should not include 4 or 5 (and by design it does not).

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

Plan 02 (store migration) can proceed in parallel with Plan 03 (generator) and Plan 04 (server routes) — all three Wave-2 plans depend only on this plan's deliverables (new `Tier` variants, `T5Formula`, `t5_formula` field, `derive_seed`/`is_valid_nonce` helpers, the two catalog TOML files). Verified:

- `cargo test --lib catalog::tests` — 15 passed (existing + 3 new + 2 extended)
- `cargo test --lib nonce::tests` — 12 passed (existing + 9 new)
- `cargo test --lib` — 111 passed total
- `cargo check` — exits 0
- `cargo clippy --all-targets --lib -- -D warnings` — exits 0
- `cargo fmt -- --check` — exits 0
- `cargo build --quiet` — exits 0
- `git diff --stat src/server/mod.rs src/store/mod.rs src/broker/mod.rs` — empty (D-13-18 compliance, Plan 02/03/04 ownership preserved)

No blockers. Plan 02 can begin its additive `ALTER TABLE events` migration against this plan's types.

## Self-Check: PASSED

Verified:
- FOUND: `assets/catalog/tier4.toml`
- FOUND: `assets/catalog/tier5.toml`
- FOUND: `src/types.rs` with `Tier4 = 4`, `Tier5 = 5`, `T5Formula`, `t5_formula: Option<T5Formula>`
- FOUND: `src/catalog/mod.rs` with `"tier4.toml"`, `"tier5.toml"`, `4 => Tier::Tier4`, `5 => Tier::Tier5`, `formula_a/b/mod: Option<u32>`
- FOUND: `src/nonce.rs` with `pub fn derive_seed`, `pub fn is_valid_nonce`, `u32::from_str_radix(&nonce[0..8], 16)`
- FOUND: `Cargo.toml` with `base64 = "0.22"`
- FOUND: commit `6d80a5c` (Task 1)
- FOUND: commit `ccb66dd` (Task 2)
- FOUND: commit `e523a0f` (Task 3)

---
*Phase: 13-tiers-4-5-backend-payloads-routes-store*
*Completed: 2026-04-24*
