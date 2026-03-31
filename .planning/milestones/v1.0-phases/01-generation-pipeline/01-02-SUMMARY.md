---
phase: 01-generation-pipeline
plan: 02
subsystem: database
tags: [rust, toml, rust-embed, rusqlite, sqlite, getrandom, hex, catalog, nonce, sql-injection-prevention]

# Dependency graph
requires:
  - phase: 01-generation-pipeline
    plan: 01
    provides: "Tier/EmbeddingLocation/Payload/NonceMapping types, Cargo workspace with dependencies"

provides:
  - "Curated payload catalog: 6 payloads across Tiers 1-3, embedded TOML via rust-embed"
  - "Nonce generator: 16-char lowercase hex from OS CSPRNG"
  - "SQLite schema: events table with replay detection (fire_count, is_replay, first_seen_at, last_seen_at, session_id) and nonce_map"
  - "load_catalog() and load_for_tiers() catalog API"
  - "open_or_create_db(), run_migrations(), insert_nonce() store API"

affects:
  - 01-03-generation
  - 02-server
  - 03-tui-monitor

# Tech tracking
tech-stack:
  added:
    - "rust-embed 8.11 (already in Cargo.toml) — used via RustEmbed derive macro for embedded TOML"
    - "rusqlite 0.39 (already in Cargo.toml) — parameterized queries via params![]"
    - "getrandom 0.4 (already in Cargo.toml) — OS CSPRNG via getrandom::fill"
    - "hex 0.4 (already in Cargo.toml) — hex::encode for nonce formatting"
  patterns:
    - "Intermediate serde structs for TOML deserialization — PayloadDef → Payload conversion avoids coupling TOML schema to domain types"
    - "Parameterized SQL only — params![] everywhere, no format!() for query construction (Pitfall 3)"
    - "Embedded assets via RustEmbed — catalog TOML compiled into binary at build time"

key-files:
  created:
    - assets/catalog/tier1.toml
    - assets/catalog/tier2.toml
    - assets/catalog/tier3.toml
    - src/catalog/mod.rs
    - src/nonce.rs
    - src/store/mod.rs
  modified:
    - src/lib.rs

key-decisions:
  - "Tier 3 second payload uses html_comment embedding location instead of a second semantic_prose — preserves D-06 uniqueness constraint (one payload per location per tier)"
  - "chrono_now() uses std::time::SystemTime epoch seconds to avoid adding a time crate dependency in Phase 1 — Phase 2 can upgrade to chrono/time when async is introduced"
  - "PayloadDef intermediate struct with u8 tier and String embedding_location — decouples TOML schema from domain enums, makes tier/location validation explicit"

patterns-established:
  - "Pattern: Always use params![] macro for rusqlite queries — never build SQL strings with format!()"
  - "Pattern: Embedded assets via RustEmbed derive — catalog TOML is part of the binary, no runtime file paths needed"
  - "Pattern: Intermediate deserialization structs — parse into loose types first, validate/convert to domain types second"

requirements-completed: [GEN-03, GEN-06, GEN-07, PROOF-01, PROOF-02, PROOF-03, SRV-02]

# Metrics
duration: 8min
completed: 2026-03-29
---

# Phase 01 Plan 02: Payload Catalog, Nonce Generator, and SQLite Schema Summary

**6-payload curated catalog embedded via rust-embed, 16-char CSPRNG nonces, and WAL-mode SQLite schema with replay detection fields locked before any network code**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-03-29T01:57:31Z
- **Completed:** 2026-03-29T02:05:00Z
- **Tasks:** 2
- **Files modified:** 7 (6 created, 1 modified)

## Accomplishments

- Created 6 curated payloads across 3 tiers: Tier 1 (arbitrary callback), Tier 2 (conditional branch with `{callback_url_a}`/`{callback_url_b}`), Tier 3 (computed callback with `{callback_url_base}`)
- Catalog loader uses RustEmbed to embed TOML at compile time — no runtime file paths, no custom payload authoring API (GEN-07)
- Nonce generator produces 16-char lowercase hex strings from 8 bytes of OS CSPRNG (getrandom::fill + hex::encode)
- SQLite WAL-mode schema with `events` table (replay detection: `fire_count`, `is_replay`, `first_seen_at`, `last_seen_at`, `session_id`) and `nonce_map` table — locked before Phase 2 server code touches it
- SQL injection prevention verified by test: malicious nonce containing `'; DROP TABLE nonce_map; --` is stored literally

## Task Commits

1. **Task 1: Payload catalog TOML files and loader module** - `0b36fdf` (feat)
2. **Task 2: Nonce generation and SQLite store with schema** - `f62877b` (feat)

## Files Created/Modified

- `assets/catalog/tier1.toml` - 2 Tier 1 payloads: html_comment and semantic_prose locations
- `assets/catalog/tier2.toml` - 2 Tier 2 conditional-branch payloads: meta_tag and json_ld locations
- `assets/catalog/tier3.toml` - 2 Tier 3 computed-callback payloads: invisible_element and html_comment locations
- `src/catalog/mod.rs` - CatalogAssets (RustEmbed), load_catalog(), load_for_tiers(), 6 tests
- `src/nonce.rs` - generate_nonce() via getrandom::fill + hex::encode, 3 tests
- `src/store/mod.rs` - open_or_create_db(), run_migrations(), insert_nonce() with params![], 4 tests
- `src/lib.rs` - Added `pub mod catalog;`, `pub mod nonce;`, `pub mod store;`

## Decisions Made

- Tier 3's second payload uses `html_comment` embedding location (not a second `semantic_prose`) to maintain D-06 uniqueness constraint within each tier
- `chrono_now()` uses `std::time::SystemTime` for timestamps — avoids adding chrono/time crate in Phase 1; Phase 2 can upgrade when async runtime is in play
- Intermediate `PayloadDef` struct (u8 tier, String embedding_location) decouples TOML schema from domain enums; conversion validates both fields explicitly

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Known Stubs

None — all catalog payloads are fully specified, store schema is complete, and nonce generation is real CSPRNG output.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Plan 03 (generator) can consume `load_for_tiers()`, `generate_nonce()`, and `insert_nonce()` directly
- SQLite schema is locked; Phase 2 server code can extend `events` inserts using the existing schema without migrations
- The `session_id` column in `events` is nullable TEXT — Phase 2 HTTP handler populates it from request context

---
*Phase: 01-generation-pipeline*
*Completed: 2026-03-29*
