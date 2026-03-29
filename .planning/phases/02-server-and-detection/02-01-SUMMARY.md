---
phase: 02-server-and-detection
plan: "01"
subsystem: types-and-fingerprinting
tags: [types, fingerprinting, crawler-catalog, phase2-foundation]
dependency_graph:
  requires: []
  provides: [AgentClass, AgentFingerprint, RawCallbackEvent, AppEvent, fingerprint::extract, fingerprint::compute_session_id, crawler_catalog::classify]
  affects: [02-02-event-pipeline, 02-03-server]
tech_stack:
  added: [tokio@1, axum@0.8, tower-http@0.6, tokio-rusqlite@0.7, axum-client-ip@1.3, sha2@0.10]
  patterns: [RustEmbed for embedded TOML catalog, SHA-256 for session ID computation, pure-function modules]
key_files:
  created:
    - src/fingerprint/mod.rs
    - src/crawler_catalog/mod.rs
    - assets/crawlers/known_crawlers.toml
  modified:
    - Cargo.toml
    - src/types.rs
    - src/lib.rs
decisions:
  - rusqlite downgraded from 0.39 to 0.37 for tokio-rusqlite 0.7 compatibility
  - axum::http::HeaderMap used directly instead of adding http crate as explicit dependency
metrics:
  duration: "3m 14s"
  completed_date: "2026-03-28"
  tasks_completed: 2
  files_modified: 6
---

# Phase 2 Plan 1: Types and Fingerprinting Summary

**One-liner:** Phase 2 foundational types (AgentClass, AgentFingerprint, AppEvent) plus pure-function fingerprint extraction and embedded TOML crawler catalog with SHA-256 session IDs.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add Phase 2 dependencies and shared types | 1569119 | Cargo.toml, Cargo.lock, src/types.rs |
| 2 | Implement fingerprint extraction and crawler catalog modules | 687df25 | src/fingerprint/mod.rs, src/crawler_catalog/mod.rs, assets/crawlers/known_crawlers.toml, src/lib.rs |

## What Was Built

### Phase 2 Dependencies (Cargo.toml)

Added six new dependencies required for the async server pipeline:
- `tokio = { version = "1", features = ["full"] }` — async runtime
- `axum = "0.8"` — HTTP framework
- `tower-http = { version = "0.6", features = ["fs"] }` — static file serving
- `tokio-rusqlite = "0.7"` — async SQLite bridge
- `axum-client-ip = "1.3"` — proxy-aware IP extraction
- `sha2 = "0.10"` — SHA-256 for session ID computation

### Shared Types (src/types.rs)

Added to the existing types module:

- `AgentClass` enum: three-tier per D-06 (`KnownCrawler { provider }`, `KnownAgent { provider }`, `Unknown`)
- `AgentFingerprint` struct: `source_ip (IpAddr)`, `user_agent (String)`, `headers (HashMap<String,String>)`, `received_at (u64)`
- `RawCallbackEvent` struct: nonce, tier, payload_id, embedding_loc, fingerprint, classification, received_at
- `AppEvent` struct: RawCallbackEvent fields plus session_id, is_replay, fire_count

### Fingerprint Module (src/fingerprint/mod.rs)

Pure functions with no side effects:

- `extract(source_ip: IpAddr, headers: &HeaderMap) -> AgentFingerprint`: extracts UA, converts headers to HashMap (UTF-8 only), strips control characters, records unix timestamp
- `compute_session_id(ip: &str, ua: &str) -> String`: SHA-256(ip + ua) first 8 bytes → 16-char hex, deterministic per D-07

### Crawler Catalog (src/crawler_catalog/mod.rs + assets/crawlers/known_crawlers.toml)

- `CrawlerCatalogAssets` struct with `#[derive(RustEmbed)]` using same pattern as payload catalog
- `CrawlerCatalog::load() -> Result<Self>`: reads embedded TOML via rust-embed
- `CrawlerCatalog::classify(&self, user_agent: &str) -> AgentClass`: UA-primary matching per D-05

Ten crawler entries: GPTBot (OpenAI/crawler), ChatGPT-User (OpenAI/agent), ClaudeBot (Anthropic/crawler), claude-web (Anthropic/agent), Googlebot (Google/crawler), Google-Extended (Google/crawler), Bingbot (Microsoft/crawler), PerplexityBot (Perplexity/crawler), CCBot (Common Crawl/crawler), anthropic-ai (Anthropic/crawler).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Downgraded rusqlite from 0.39 to 0.37 for tokio-rusqlite compatibility**
- **Found during:** Task 1 — cargo check
- **Issue:** `tokio-rusqlite 0.7` requires `rusqlite ^0.37`, but project had `rusqlite = "0.39"`. Cargo rejected the conflicting `libsqlite3-sys` native library links.
- **Fix:** Changed `rusqlite = { version = "0.37", features = ["bundled"] }` in Cargo.toml. The `bundled` feature ensures SQLite is compiled in regardless of version.
- **Files modified:** Cargo.toml, Cargo.lock
- **Commit:** 1569119

**2. [Rule 3 - Blocking] Used axum::http::HeaderMap instead of http::HeaderMap**
- **Found during:** Task 2 — cargo check
- **Issue:** `http` crate was a transitive dependency through axum but not directly importable as `http::HeaderMap` without explicit declaration.
- **Fix:** Changed import to `axum::http::HeaderMap` which re-exports the same type.
- **Files modified:** src/fingerprint/mod.rs
- **Commit:** 687df25

## Test Results

```
test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All tests pass:
- 6 types tests (AgentClass, AgentFingerprint, RawCallbackEvent, AppEvent)
- 6 fingerprint tests (extract, compute_session_id determinism, hex format)
- 6 crawler_catalog tests (GPTBot, ClaudeBot, Googlebot, Unknown, empty UA, ChatGPT-User)
- 25 existing Phase 1 tests (no regressions)

## Known Stubs

None — all functions are fully implemented with correct behavior.

## Self-Check: PASSED
