---
phase: 13
slug: tiers-4-5-backend-payloads-routes-store
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-24
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
> Derived from 13-RESEARCH.md §Validation Architecture.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (stdlib + `tokio::test` for async) |
| **Config file** | None — `#[cfg(test)] mod tests` per module + `tests/*.rs` integration |
| **Quick run command** | `cargo test --lib -p honeyprompt` |
| **Full suite command** | `cargo test` |
| **Lint / format gate** | `cargo clippy --all-targets -- -D warnings && cargo fmt -- --check` |
| **Proof-specific filter** | `cargo test --lib proof` |
| **Estimated runtime** | ~10s quick / ~30s full on a modern machine |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib -p honeyprompt`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite + clippy + fmt must be green
- **Max feedback latency:** 10s (quick) / 30s (full)

---

## Per-Task Verification Map

> Task IDs are placeholders until `gsd-planner` finalizes plans. Each plan MUST assign concrete `{N}-{plan}-{task}` IDs that consume one or more of these requirements. Wave column is an expected-bucket hint; planner sets final wave.

| Req ID | Plan (expected) | Wave | Requirement (short) | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|--------|-----------------|------|---------------------|------------|-----------------|-----------|-------------------|-------------|--------|
| PAYLOAD-01 | catalog | 1 | tier4.toml ships 3 templates | — | curated-only, no user payloads (D-06) | unit | `cargo test --lib catalog::tests::test_tier4_catalog` | ❌ W0 | ⬜ pending |
| PAYLOAD-02 | catalog | 1 | T4 templates use distinct phrasing | — | N/A | unit | `cargo test --lib catalog::tests::test_tier4_diverse_phrasing` | ❌ W0 | ⬜ pending |
| PAYLOAD-03 | catalog | 1 | tier5.toml ships 2–3 templates with `formula_a/b/mod` | — | curated-only (D-06) | unit | `cargo test --lib catalog::tests::test_tier5_catalog` | ❌ W0 | ⬜ pending |
| PAYLOAD-04 | generator | 2 | T5 active ⇒ seed JSON-LD on page | — | seed derived, never leaked outside page | unit | `cargo test --lib generator::tests::test_t5_seed_json_ld_emission` | ❌ W0 | ⬜ pending |
| PAYLOAD-05 | generator | 2 | 5 embedding locations cover T4/T5; no T1–T3 regression | — | no duplicate locations within a tier | unit | `cargo test --lib catalog::tests::test_no_duplicate_locations` (extended 1..=5) | ✅ extend | ⬜ pending |
| SERVER-01 | server | 3 | T4 route decode+sanitize+store+204 | T-13-01 (always 204, no leak) | decode URL-safe b64, regex sanitize, upsert, 204 | integration | `cargo test --test test_serve test_t4_callback_happy_path` | ❌ W0 | ⬜ pending |
| SERVER-02 | server | 3 | T5 route verifies proof against derived seed | T-13-02 (proof correctness) | derive seed, compute expected, store (proof, proof_valid), 204 | integration | `cargo test --test test_serve test_t5_callback_valid_proof`, `..._invalid_proof` | ❌ W0 | ⬜ pending |
| SERVER-03 | server | 3 | `/cb/v1/{nonce}` byte-identical to v4.0 | T-13-03 (zero regression) | existing handler untouched | regression | `cargo test --test test_serve` (existing tests unmodified) + `cargo test --test test_serve test_cb_v1_byte_identical_response` | ✅ extend | ⬜ pending |
| SERVER-04 | server | 3 | Malformed inputs → 204 (oversize b64, non-b64, non-numeric) | T-13-01, T-13-04 | silent 204, nothing stored | integration | `cargo test --test test_serve test_t4_malformed_returns_204`, `test_t5_malformed_returns_204` | ❌ W0 | ⬜ pending |
| STORE-01 | store | 2 | events.t4_capability TEXT NULL column added | — | additive, no drops | unit | `cargo test --lib store::tests::test_schema_t4_columns` | ❌ W0 | ⬜ pending |
| STORE-02 | store | 2 | events.t5_proof TEXT NULL + t5_proof_valid INTEGER NULL added | — | additive, no drops | unit | `cargo test --lib store::tests::test_schema_t5_columns` | ❌ W0 | ⬜ pending |
| STORE-03 | store | 2 | v4.0 DB opens unchanged; T1–T3 rows read identically | T-13-05 (back-compat) | `user_version` gating; idempotent | integration | `cargo test --test test_migration test_v4_db_opens_unchanged` | ❌ W0 (new file) | ⬜ pending |
| STORE-04 | store | 2 | Replay detection identical for T4/T5 | — | `ON CONFLICT(nonce) DO UPDATE` pattern; new columns NOT in SET clause | unit | `cargo test --lib store::tests::test_insert_callback_event_replay_t4`, `..._t5` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Critical Cross-Cutting Tests (must exist)

### Proof computation (HIGH-risk correctness — u32 overflow trap)

- `test_compute_expected_proof_zero_seed` — seed=0, a=0, b=1, mod=1000 → 0
- `test_compute_expected_proof_max_seed_no_overflow` — seed=u32::MAX, a=1_000_000, b=1_000_000, mod=1000 — MUST NOT PANIC in debug build (proves u64 promotion)
- `test_compute_expected_proof_known_vector` — hard-code a seed/a/b/mod tuple with its expected 3-digit output
- `test_derive_seed_valid_nonce` — `"abcdef1234567890"` → `Some(0xabcdef12)`
- `test_derive_seed_short_nonce` — `"abc"` → `None` (no panic)

### Base64 decode + sanitize

- `test_t4_decode_valid_payload`
- `test_t4_decode_rejects_oversize` (>400 char b64 → None)
- `test_t4_decode_rejects_invalid_chars` (decoded contains `!@#` → None)
- `test_t4_decode_normalizes_case_and_whitespace`

### Migration idempotency

- `test_migration_idempotent` — run `run_migrations` twice; second run must not error
- `test_migration_from_v4_schema` — construct v4.0 schema, migrate, assert columns added + pre-existing rows untouched

### Regression (MUST NOT BE MODIFIED)

- `tests/test_serve.rs` — all existing `/cb/v1/` tests pass byte-identically
- `tests/test_generate.rs` — existing assertions hold

---

## Wave 0 Requirements

Wave 0 establishes test harnesses and extends existing test files before any feature code lands.

- [ ] `src/catalog/mod.rs::tests` — extend `test_load_all_payloads` total count to include T4+T5 counts, extend `test_no_duplicate_locations` tier range `1..=5`
- [ ] `src/store/mod.rs::tests` — stubs for `test_schema_t4_columns`, `test_schema_t5_columns`, `test_migration_idempotent`, `test_migration_from_v4_schema`, `test_insert_callback_event_replay_t4`, `test_insert_callback_event_replay_t5`
- [ ] `src/server/mod.rs::tests` (or `tests/test_serve.rs`) — stubs for T4/T5 handler tests
- [ ] `tests/test_migration.rs` — NEW integration test file; `test_v4_db_opens_unchanged`. Construct v4.0 schema programmatically (no binary fixture) to avoid repo bloat
- [ ] `src/generator/mod.rs::tests` — stub `test_t5_seed_json_ld_emission`
- [ ] Proof + sanitize unit test stubs in whichever module owns proof verification (planner decides — `src/server/` or new `src/verify/`)
- [ ] Framework install: **none needed** — `cargo test` + `tokio::test` already in tree; `base64 = "0.22"` promoted to direct dep in Cargo.toml

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Instruction text matches chosen base64 variant (URL-safe no-pad) | PAYLOAD-01..03 | Copy-editing correctness — no automated English-to-format check | Review each T4/T5 catalog entry; confirm instruction text says "URL-safe base64, no padding" or equivalent example. Risk 4 in RESEARCH.md. |
| Formula constants produce well-distributed `[0, 999]` outputs | PAYLOAD-03 | Distribution check is aesthetic | For each T5 template, compute proofs for seeds `{0x00000000, 0x11111111, ..., 0xFFFFFFFF}` and confirm output values span the range. Optional. |

---

## Validation Sign-Off

- [ ] All 13 REQ-IDs have an `<automated>` command in the per-task map
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all ❌ W0 references
- [ ] No watch-mode flags (all commands terminate)
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter once plans assign concrete task IDs
- [ ] Every security-relevant task references a T-13-xx threat from PLAN threat_model

**Approval:** pending
