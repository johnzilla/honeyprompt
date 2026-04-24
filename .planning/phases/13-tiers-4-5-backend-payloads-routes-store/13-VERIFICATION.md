---
phase: 13-tiers-4-5-backend-payloads-routes-store
verified: 2026-04-24T00:00:00Z
status: passed
score: 13/13 must-haves verified
overrides_applied: 0
---

# Phase 13: Tiers 4 & 5 Backend Verification Report

**Phase Goal:** "The honeypot can emit Tier 4 and Tier 5 payloads, receive their callbacks at new `/cb/v4/` and `/cb/v5/` routes, verify T5 proofs server-side, and persist results in an additively-migrated SQLite schema — all without changing `/cb/v1/` behavior or breaking existing T1–T3 rows."

**Verified:** 2026-04-24
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| #   | Truth                                                                                                  | Status     | Evidence                                                                                                                                                                         |
| --- | ------------------------------------------------------------------------------------------------------ | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Generate produces HTML with 2–3 T4 + 2–3 T5 payloads across all five embedding locations               | VERIFIED   | tier4.toml: 3 payloads (meta_tag, semantic_prose, json_ld); tier5.toml: 3 payloads (semantic_prose, html_comment, invisible_element). Union = all 5 locations. test_generate: 11 passing. |
| 2   | T4 route decodes + sanitizes + stores + 204 on happy path; malformed → 204, nothing stored             | VERIFIED   | tests/test_serve.rs:474 `test_t4_callback_happy_path` + :504 `test_t4_malformed_returns_204` both pass. Handler at src/server/mod.rs:97 always returns NO_CONTENT.                |
| 3   | T5 route verifies proof server-side, stores proof + proof_valid, 204 always                            | VERIFIED   | tests/test_serve.rs:544 `test_t5_callback_valid_proof` + :575 `test_t5_callback_invalid_proof` + :608 `test_t5_malformed_returns_204` all pass. Handler at src/server/mod.rs:144.  |
| 4   | v4.0 DB file opens unchanged; T1–T3 rows read identically                                              | VERIFIED   | tests/test_migration.rs:67 `test_v4_db_opens_unchanged` + `test_v4_db_migration_idempotent_across_reopen` both pass. Additive-only migration at src/store/mod.rs:60-62.            |
| 5   | /cb/v1/ byte-identical                                                                                 | VERIFIED   | tests/test_serve.rs:644 `test_cb_v1_byte_identical_response` passes. callback_handler (src/server/mod.rs:45-90) preserves inline nonce validation from v4.0.                     |

**Score:** 5/5 roadmap success criteria verified

### Required Artifacts

| Artifact                            | Expected                                                            | Status     | Details                                                                                                                                    |
| ----------------------------------- | ------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `assets/catalog/tier4.toml`         | 3 T4 payloads, no formula fields                                    | VERIFIED   | 3 `[[payloads]]`; 0 `formula_a`; 3 distinct embedding_locations (meta_tag, semantic_prose, json_ld); all 3 contain "URL-safe base64".      |
| `assets/catalog/tier5.toml`         | 3 T5 payloads, each with formula_a/b/mod u32 fields                 | VERIFIED   | 3 `[[payloads]]`; each has formula_a/b/mod; all 3 `formula_mod = 1000`; 3 distinct embedding_locations (semantic_prose, html_comment, invisible_element). |
| `src/types.rs`                      | Tier::Tier4/Tier5 variants + T5Formula + event fields                | VERIFIED   | Lines 9-10 (Tier4=4, Tier5=5), line 22 (T5Formula), line 58 (Payload.t5_formula), lines 99/102/105 (RawCallbackEvent), lines 122/124/126 (AppEvent). |
| `src/catalog/mod.rs`                | load_catalog 5 tier files; PayloadDef formula fields; Tier 4/5 arms   | VERIFIED   | Filename loop includes tier4.toml/tier5.toml; PayloadDef gains formula_a/b/mod Option<u32>; tier match has 4 => Tier::Tier4, 5 => Tier::Tier5. |
| `src/nonce.rs`                      | derive_seed + is_valid_nonce public helpers                         | VERIFIED   | src/nonce.rs:15 `pub fn derive_seed`, :26 `pub fn is_valid_nonce`. 9 unit tests covering edge cases.                                       |
| `src/store/mod.rs`                  | PRAGMA user_version gate, additive ALTER, extended insert signature | VERIFIED   | Lines 56-63: gated ALTER TABLE block (3 adds + PRAGMA user_version=1). Line 85: extended signature. ON CONFLICT clause does NOT include new columns. |
| `src/broker/mod.rs`                 | broker_task propagates, db_writer_task passes real values           | VERIFIED   | Lines 32-34 propagate raw.t4_capability/t5_proof/t5_proof_valid. Lines 80-81 pass via `.as_deref()` + value (NOT None placeholders).       |
| `src/generator/mod.rs`              | Tier 4/5 match arms + seed JSON-LD emission                         | VERIFIED   | Lines 153, 181: Tier::Tier4 / Tier::Tier5 arms. Lines 192, 201-204: /cb/v5/{nonce}, derive_seed, verification_seed JSON-LD format.         |
| `src/server/mod.rs`                 | NonceMeta + t4/t5 handlers + routes + serve() loads catalog         | VERIFIED   | NonceMeta.t5_formula:28. t4 handler:97, t5 handler:144. Routes at 225-227. serve() loads catalog at 252 before AppState at 305.             |
| `assets/templates/index.html.jinja` | seed_scripts_json safe-rendered inside <head>                       | VERIFIED   | Line 16: `{{ seed_scripts_json | safe }}`. No other template modifications.                                                                |
| `tests/test_migration.rs`           | v4 → v5 additive migration integration tests                        | VERIFIED   | 2 tests pass (test_v4_db_opens_unchanged, test_v4_db_migration_idempotent_across_reopen). No binary fixtures.                              |
| `tests/test_serve.rs`               | 6 new integration tests for T4/T5 + /cb/v1/ byte-identical          | VERIFIED   | All 6 new tests exist at expected lines (474, 504, 544, 575, 608, 644) and pass. No existing test bodies modified.                         |
| `Cargo.toml`                        | base64 = "0.22" direct dependency                                    | VERIFIED   | Confirmed by orchestrator schema-drift check and compilation.                                                                              |

### Key Link Verification

| From                                        | To                                                       | Via                                | Status   | Details                                                                                                                            |
| ------------------------------------------- | -------------------------------------------------------- | ---------------------------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| tier4.toml                                  | catalog::PayloadDef                                      | TOML deserialization               | WIRED    | load_catalog iterates `["tier1.toml",...,"tier4.toml","tier5.toml"]`; 12-payload test_load_all_payloads passes.                   |
| tier5.toml                                  | types::T5Formula                                         | PayloadDef→Payload conversion      | WIRED    | Flat formula_a/b/mod fields deserialize; into_payload validates partial fields error, non-T5 with formula errors; test_tier5_catalog. |
| nonce::derive_seed                          | u32::from_str_radix(&nonce[0..8], 16).ok()               | D-13-04 seed extraction            | WIRED    | src/nonce.rs:15-20 implements exactly this.                                                                                        |
| run_migrations                              | PRAGMA user_version gate                                 | Query + conditional ALTER          | WIRED    | src/store/mod.rs:56 reads PRAGMA user_version; line 58 conditional on `version < 1`; line 63 sets user_version=1.                  |
| insert_callback_event                       | ON CONFLICT SET w/o new columns (first-write-wins)       | D-13-19 replay semantics           | WIRED    | src/store/mod.rs:108-111: SET clause contains only last_seen_at, fire_count, is_replay. Replay tests pass.                         |
| t5_callback_handler                         | nonce::derive_seed + compute_expected_proof              | D-13-02 proof verification         | WIRED    | src/server/mod.rs:172 derive_seed, :176 compute_expected_proof; u64 promotion at 361-366.                                          |
| compute_expected_proof                      | u64 arithmetic                                           | D-13-02 overflow-safe formula      | WIRED    | Cast each input to u64 BEFORE arithmetic; wrapping_add/wrapping_mul for defense-in-depth.                                          |
| build_router                                | /cb/v4/ + /cb/v5/ routes                                 | Axum 0.8 tuple-Path                | WIRED    | src/server/mod.rs:226-227. /cb/v1/ at 225 preserved byte-identical.                                                                 |
| generator T5 arm                            | seed_scripts_json accumulator → template                 | minijinja `| safe`                 | WIRED    | Accumulator at 224, join at 233, context! key at 241; template renders at line 16 of index.html.jinja.                             |

### Requirements Coverage

| Requirement | Source Plan | Description                                                            | Status    | Evidence                                                                                                                                     |
| ----------- | ----------- | ---------------------------------------------------------------------- | --------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| PAYLOAD-01  | 13-01       | 2–3 T4 introspection payload templates                                 | SATISFIED | tier4.toml has 3 `[[payloads]]`; test_tier4_catalog passes.                                                                                  |
| PAYLOAD-02  | 13-01       | Distinct T4 phrasings across capability dimensions                     | SATISFIED | tools/model/scopes dimensions (t4-tools-meta, t4-model-prose, t4-scopes-jsonld); test_tier4_diverse_phrasing passes.                          |
| PAYLOAD-03  | 13-01       | 2–3 T5 multi-step compliance payload templates                         | SATISFIED | tier5.toml has 3 `[[payloads]]` with formula constants; test_tier5_catalog passes.                                                           |
| PAYLOAD-04  | 13-03       | T5 deterministic `verification_seed` in JSON-LD                        | SATISFIED | Generator emits `{"verification_seed":<seed>,"nonce":"<nonce>"}` per T5 nonce; test_t5_seed_json_ld_emission passes (seed = derive_seed).     |
| PAYLOAD-05  | 13-01       | T4/T5 render across 5 embedding locations without regressing T1–T3     | SATISFIED | T4: {meta_tag, semantic_prose, json_ld}; T5: {semantic_prose, html_comment, invisible_element}. Union = all 5. test_no_duplicate_locations 1..=5. |
| SERVER-01   | 13-04       | /cb/v4/{nonce}/{b64_list} decodes, sanitizes, stores, returns 204       | SATISFIED | src/server/mod.rs:97 t4_callback_handler; test_t4_callback_happy_path passes.                                                                 |
| SERVER-02   | 13-04       | /cb/v5/{nonce}/{proof} stores proof + computes expected from seed      | SATISFIED | src/server/mod.rs:144 t5_callback_handler; stores submitted proof + proof_valid; test_t5_callback_valid_proof passes.                          |
| SERVER-03   | 13-04       | /cb/v1/{nonce} behavior unchanged (frozen)                             | SATISFIED | callback_handler body preserves inline nonce validation; route unchanged; test_cb_v1_byte_identical_response passes; all existing test_serve tests pass unmodified. |
| SERVER-04   | 13-04       | T4/T5 routes return 204 on all malformed inputs                         | SATISFIED | test_t4_malformed_returns_204 + test_t5_malformed_returns_204 collectively cover 10 failure branches; all 204.                                |
| STORE-01    | 13-02       | T4 capability summary column (text)                                    | SATISFIED | events.t4_capability TEXT column added via ALTER TABLE; test_schema_t4_columns passes.                                                        |
| STORE-02    | 13-02       | T5 proof + proof_valid columns                                         | SATISFIED | events.t5_proof TEXT + events.t5_proof_valid INTEGER columns added; test_schema_t5_columns passes.                                            |
| STORE-03    | 13-02       | Additive non-destructive migration from v4.0                           | SATISFIED | Gated ALTER TABLE ADD COLUMN only (no DROP, no ALTER COLUMN); test_v4_db_opens_unchanged confirms T1–T3 rows read identically.                |
| STORE-04    | 13-02       | Replay + session grouping identical for T4/T5                          | SATISFIED | ON CONFLICT SET clause excludes new columns (first-write-wins D-13-19); test_insert_callback_event_replay_t4/t5_first_write_wins pass.         |

**13/13 phase requirements satisfied.** No orphaned requirements — every REQ ID listed in the phase mapping is accounted for across plans 01-04.

### Requirements Accounting

- Plan 01 requirements: [PAYLOAD-01, PAYLOAD-02, PAYLOAD-03, PAYLOAD-05] → 4
- Plan 02 requirements: [STORE-01, STORE-02, STORE-03, STORE-04] → 4
- Plan 03 requirements: [PAYLOAD-04] → 1
- Plan 04 requirements: [SERVER-01, SERVER-02, SERVER-03, SERVER-04] → 4
- **Sum:** 13 unique IDs = phase REQ set. No duplicates. No gaps.
- All 4 SUMMARY.md `requirements-completed:` fields match their PLAN `requirements:` exactly.

### Critical Invariant Checks

| # | Invariant                                                         | Status | Evidence                                                                                                                        |
| - | ----------------------------------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------- |
| 1 | Always 204 in T4/T5 handlers (D-13-15 / T-13-01)                   | PASSED | grep for StatusCode:: in t4/t5 handler bodies returns ONLY StatusCode::NO_CONTENT (10 occurrences). No 4xx/5xx anywhere.          |
| 2 | /cb/v1/ byte-identity (D-13-18 / T-13-03)                          | PASSED | callback_handler lines 45-90 preserves inline nonce validation (`nonce.len() == 16 && chars().all(...)`). Returns NO_CONTENT. RawCallbackEvent constructed with t4_capability/t5_proof/t5_proof_valid = None. test_cb_v1_byte_identical_response asserts 204 + empty body. All pre-existing test_serve tests pass unmodified. |
| 3 | Additive migration only (D-13-17 / T-13-05)                        | PASSED | grep for DROP or ALTER COLUMN in src/store/mod.rs migration block returns only a test string ("'; DROP TABLE nonce_map; --" at line 774, a SQL-injection regression test). Migration at 56-64 contains only 3× ALTER TABLE ADD COLUMN + PRAGMA user_version=1, gated by `version < 1`. |
| 4 | First-write-wins replay (D-13-19)                                 | PASSED | `ON CONFLICT(nonce) DO UPDATE SET` at src/store/mod.rs:108-111 contains ONLY `last_seen_at = ?5, fire_count = fire_count + 1, is_replay = 1`. No t4_capability/t5_proof/t5_proof_valid in SET clause. Replay tests confirm behavior. |
| 5 | u64 overflow guard (T-13-02)                                      | PASSED | compute_expected_proof casts seed/a/b/m to u64 BEFORE arithmetic. Uses wrapping_add/wrapping_mul on u64. test_compute_expected_proof_max_seed_no_overflow exists at src/server/mod.rs:414 and passes. |
| 6 | Parenthesization guard (T-13-02)                                  | PASSED | test_compute_expected_proof_seed_zero_nontrivial_a_b at src/server/mod.rs:432 uses seed=1, a=42, b=17, mod=1000, asserts == 731. Correctly distinguishes `((seed+a)*b)` from `(seed+(a*b))`. Test passes. |
| 7 | Structural ordering invariant (serve() init order)                | PASSED | src/server/mod.rs line 252 (`catalog::load_catalog()`) < line 253 (`t5_formulas_by_payload_id` HashMap) < line 258 (nonce_map loop) < line 305 (`Arc::new(app_state)`). All strictly ordered correctly. |
| 8 | 13 REQ-IDs covered (PLAN ∩ SUMMARY consistency)                   | PASSED | See Requirements Accounting above. 4+4+1+4=13 unique REQ IDs across plans; SUMMARY `requirements-completed:` fields match PLAN `requirements:` exactly for all 4 plans. |

### Threat Register Verification

| Threat ID     | Component                                   | Mitigation Present? | Evidence                                                                                                                           |
| ------------- | ------------------------------------------- | ------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| T-13-01       | Always 204 in T4/T5 handlers                 | YES                 | See Invariant 1. All handler returns are StatusCode::NO_CONTENT.                                                                    |
| T-13-02       | compute_expected_proof correctness           | YES                 | See Invariants 5 + 6. u64 promotion + parenthesization + 4 unit tests (zero_seed, max_seed_no_overflow, known_vector, parenthesization). |
| T-13-03       | /cb/v1/ byte-identity                        | YES                 | See Invariant 2. Handler unchanged; dedicated regression test + all existing tests pass unmodified.                                 |
| T-13-04       | T4 stored capability sanitization             | YES                 | decode_t4_payload validates against `^[a-z0-9_,.\-]{1,256}$` via hand-rolled is_valid_t4_payload. test_t4_decode_rejects_invalid_chars + test_t4_decode_normalizes_case_and_whitespace pass. |
| T-13-05       | Additive migration (no DROP)                  | YES                 | See Invariant 3. Only ALTER TABLE ADD COLUMN.                                                                                       |
| T-13-XSS-SEED | seed_scripts_json `| safe` filter             | YES                 | Accumulator only contains strings from `format!` with u32 seed + 16-hex-char nonce — no user-controlled input. Documented at plan 03 threat section; implementation matches. |

### Anti-Patterns Found

| File                                  | Line | Pattern                                 | Severity | Impact                                                                                              |
| ------------------------------------- | ---- | --------------------------------------- | -------- | --------------------------------------------------------------------------------------------------- |
| (none)                                | —    | —                                       | —        | No blocker/warning anti-patterns detected. No TODO/FIXME/PLACEHOLDER/HACK markers introduced in phase 13 code. No empty `return null`/stub patterns. No unexplained hardcoded empties. |

### Behavioral Spot-Checks

| Behavior                                              | Command                                                                           | Result                   | Status |
| ----------------------------------------------------- | --------------------------------------------------------------------------------- | ------------------------ | ------ |
| Full test suite passes (incl. T4/T5 + migration tests) | `cargo test`                                                                      | 181 passed, 0 failed     | PASS   |
| Library tests include T4/T5 unit coverage              | `cargo test --lib`                                                                | 133 passed, 0 failed     | PASS   |
| Integration tests include T4/T5 + migration            | `cargo test --test test_serve` + `cargo test --test test_migration`                 | All pass (validated in suite) | PASS   |
| Clippy clean                                           | `cargo clippy --all-targets -- -D warnings`                                       | exit 0                   | PASS   |
| fmt clean                                              | `cargo fmt -- --check`                                                            | exit 0                   | PASS   |

All spot-checks relied on orchestrator pre-verification results (documented as pre-verified: `181 passed, 0 failed, 0 ignored`; clippy exit 0; fmt exit 0; schema-drift clean).

### Human Verification Required

None. All goal-achievement truths are verifiable via tests and static grep checks. Visual/UI concerns (monitor TUI rendering, report formatting) are explicitly out of scope for phase 13 (deferred to phase 14).

### Gaps Summary

No gaps found. Every roadmap success criterion is backed by a passing test. Every declared must-have is present in the codebase. Every critical invariant holds. Threat register fully mitigated. All 13 phase requirements satisfied with traceable test evidence.

Phase 13 completes the Tier 4 and Tier 5 backend end-to-end:
- Catalog ships 3 T4 + 3 T5 payloads across all 5 embedding locations.
- Generator emits per-T5 `verification_seed` JSON-LD blocks deterministically derived from nonces.
- SQLite migrates additively via PRAGMA user_version gating; no data loss for existing v4.0 DBs.
- `/cb/v4/` decodes + sanitizes + stores; `/cb/v5/` verifies proof server-side with u64-safe arithmetic.
- `/cb/v1/` is byte-identical to v4.0 — frozen contract preserved.
- Every failure mode on new routes returns 204 (no diagnostic leak).

---

## VERIFICATION PASSED

_Verified: 2026-04-24_
_Verifier: Claude (gsd-verifier)_
