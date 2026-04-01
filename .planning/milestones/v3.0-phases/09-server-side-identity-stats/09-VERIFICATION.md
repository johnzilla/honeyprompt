---
phase: 09-server-side-identity-stats
verified: 2026-03-31T00:00:00Z
status: passed
score: 5/5 must-haves verified
re_verification: false
---

# Phase 9: Server-Side Identity & Stats Verification Report

**Phase Goal:** honeyprompt.sh has a verifiable identity footer and serves live aggregate stats via a public JSON endpoint
**Verified:** 2026-03-31
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #   | Truth                                                                                                                | Status     | Evidence                                                                                                      |
| --- | -------------------------------------------------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------- |
| 1   | Every generated honeypot page displays a footer with project name, honeyprompt.dev link, and disclosure contact      | ✓ VERIFIED | `<footer>` block with `honeyprompt.dev` and security advisories link in `index.html.jinja` lines 48-51        |
| 2   | GET /.well-known/security.txt returns a valid RFC 9116 document with Contact, Expires, and Preferred-Languages       | ✓ VERIFIED | `security.txt.jinja` contains all 3 fields; generator writes to `output/.well-known/security.txt`; ServeDir serves it |
| 3   | GET /stats returns JSON with total_sessions, detection_sessions, crawler_sessions, per-tier counts, and timestamps   | ✓ VERIFIED | `stats_handler` in `server/mod.rs` calls `query_report_summary` and returns `axum::Json(summary)`            |
| 4   | /stats response includes Access-Control-Allow-Origin: * header                                                       | ✓ VERIFIED | `stats_handler` inserts `access-control-allow-origin: *` header at `server/mod.rs` lines 98-101              |
| 5   | /stats returns all-zero counts (not an error) when the database has no events                                        | ✓ VERIFIED | `query_report_summary` returns zero-valued `ReportSummary`; `test_stats_empty_db_returns_json` confirms this  |

**Score:** 5/5 truths verified

---

### Required Artifacts

#### Plan 01 Artifacts (IDENT-01, IDENT-02)

| Artifact                                    | Expected                                             | Status     | Details                                                                                          |
| ------------------------------------------- | ---------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------ |
| `assets/templates/index.html.jinja`         | Footer section before `</body>` with `honeyprompt.dev` | ✓ VERIFIED | Footer at lines 48-51, before `</body>` at line 52; contains `honeyprompt.dev` and advisories link |
| `assets/templates/security.txt.jinja`       | RFC 9116 security.txt template with Contact field    | ✓ VERIFIED | 3-line file with Contact, Expires, Preferred-Languages; trailing newline present                |
| `src/generator/mod.rs`                      | security.txt rendering and output to .well-known/    | ✓ VERIFIED | Lines 174-175 render template; lines 184-195 create `.well-known/` and write security.txt       |

#### Plan 02 Artifacts (STATS-01, STATS-02, STATS-03)

| Artifact              | Expected                                                | Status     | Details                                                                                                         |
| --------------------- | ------------------------------------------------------- | ---------- | --------------------------------------------------------------------------------------------------------------- |
| `src/store/mod.rs`    | ReportSummary with `#[derive(serde::Serialize)]`        | ✓ VERIFIED | `#[derive(serde::Serialize)]` present at line 144; all 8 fields serialized                                      |
| `src/server/mod.rs`   | `/stats` handler with CORS header and DB query          | ✓ VERIFIED | `stats_handler` at lines 84-106; CORS header injected; `query_report_summary` called via `conn.call()`          |
| `tests/test_serve.rs` | Integration tests for /stats endpoint                   | ✓ VERIFIED | 3 tests: `test_stats_empty_db_returns_json`, `test_stats_populated_db_returns_counts`, `test_stats_has_cors_header` |

---

### Key Link Verification

#### Plan 01 Key Links

| From                        | To                                        | Via                                   | Status     | Details                                                                      |
| --------------------------- | ----------------------------------------- | ------------------------------------- | ---------- | ---------------------------------------------------------------------------- |
| `src/generator/mod.rs`      | `assets/templates/security.txt.jinja`     | `render_template("security.txt.jinja", ...)` | ✓ WIRED | Line 174-175: `render_template("security.txt.jinja", context! {})`         |
| `src/generator/mod.rs`      | `output/.well-known/security.txt`         | `fs::write`                           | ✓ WIRED    | Lines 184-185: `create_dir_all(&well_known_dir)` + line 194: `fs::write(well_known_dir.join("security.txt"), ...)` |

#### Plan 02 Key Links

| From                                  | To                                          | Via                                                         | Status     | Details                                                                                              |
| ------------------------------------- | ------------------------------------------- | ----------------------------------------------------------- | ---------- | ---------------------------------------------------------------------------------------------------- |
| `src/server/mod.rs stats_handler`     | `src/store/mod.rs query_report_summary`     | `conn.call(\|c\| store::query_report_summary(c)...)`        | ✓ WIRED    | Line 93: `crate::store::query_report_summary(c)` inside `conn.call()` block                         |
| `src/server/mod.rs AppState`          | `tokio_rusqlite::Connection`                | `conn` field in AppState                                    | ✓ WIRED    | Line 28: `pub conn: tokio_rusqlite::Connection` in AppState struct                                   |
| `src/server/mod.rs build_router`      | `stats_handler`                             | `.route("/stats", get(stats_handler))`                      | ✓ WIRED    | Line 114: `.route("/stats", get(stats_handler))` in `build_router`                                   |
| `src/server/mod.rs serve()`           | AppState conn field                         | `conn: conn.clone()` in AppState construction               | ✓ WIRED    | Lines 170-175: AppState constructed with `conn: conn.clone()`                                        |

---

### Data-Flow Trace (Level 4)

| Artifact              | Data Variable      | Source                                                  | Produces Real Data | Status      |
| --------------------- | ------------------ | ------------------------------------------------------- | ------------------ | ----------- |
| `src/server/mod.rs`   | `summary`          | `store::query_report_summary(c)` via 7 SQLite queries   | Yes — DB queries   | ✓ FLOWING   |
| `src/store/mod.rs`    | `ReportSummary`    | 7 `conn.query_row(...)` calls against `events` table    | Yes — DB queries   | ✓ FLOWING   |

The `stats_handler` calls `conn.call(|c| store::query_report_summary(c)...)` which executes 7 real SQL queries (`COUNT(DISTINCT ...)`, `MIN/MAX`) against the `events` table. The result is returned directly as the JSON response body — no static fallback, no hardcoded data.

---

### Behavioral Spot-Checks

| Behavior                                              | Check                                    | Result                                      | Status   |
| ----------------------------------------------------- | ---------------------------------------- | ------------------------------------------- | -------- |
| All 8 serve integration tests pass (including /stats) | `cargo test` on `tests/test_serve.rs`    | 8/8 pass                                    | ✓ PASS   |
| All 11 generate integration tests pass (inc footer)   | `cargo test` on `tests/test_generate.rs` | 11/11 pass                                  | ✓ PASS   |
| Full suite passes (89 unit + 11 + 8 + 9 + 4 others)  | `cargo test` all                         | 131 tests, 0 failures                       | ✓ PASS   |
| Clippy clean                                          | `cargo clippy -- -D warnings`            | No warnings, exits 0                        | ✓ PASS   |
| Phase commits present in git history                  | `git log --oneline`                      | 63b2da2, cbd7ad8, 6baaecd, c422779 confirmed | ✓ PASS   |

---

### Requirements Coverage

| Requirement | Source Plan | Description                                                                                                     | Status       | Evidence                                                                                       |
| ----------- | ----------- | --------------------------------------------------------------------------------------------------------------- | ------------ | ---------------------------------------------------------------------------------------------- |
| IDENT-01    | Plan 01     | Honeypot page includes footer with project name, honeyprompt.dev link, and disclosure contact for human visitors | ✓ SATISFIED  | Footer in `index.html.jinja` lines 48-51; rendered to `output/index.html` by generator        |
| IDENT-02    | Plan 01     | /.well-known/security.txt served from honeyprompt.sh with valid RFC 9116 fields (Contact, Expires, Preferred-Languages) | ✓ SATISFIED | `security.txt.jinja` with all 3 RFC 9116 fields; written to `output/.well-known/security.txt`; served by ServeDir |
| STATS-01    | Plan 02     | GET /stats returns JSON with aggregate callback counts (total_sessions, detection_sessions, crawler_sessions, per-tier counts, earliest/latest timestamps) | ✓ SATISFIED | `stats_handler` returns `axum::Json(ReportSummary)` with all 8 fields from live DB queries    |
| STATS-02    | Plan 02     | /stats response includes Access-Control-Allow-Origin: * header                                                  | ✓ SATISFIED  | `stats_handler` inserts `access-control-allow-origin: *` header at `server/mod.rs` lines 98-101 |
| STATS-03    | Plan 02     | /stats returns all-zero counts on empty database (not an error response)                                        | ✓ SATISFIED  | `query_report_summary` returns zero-valued struct on empty DB; confirmed by `test_stats_empty_db_returns_json` and `test_query_report_summary_empty_db` |

**Requirements coverage: 5/5 — all IDENT and STATS requirements satisfied.**

Phase 10 requirements (LAND-01 through LAND-04) are correctly scoped to Phase 10 and are not claimed by any Phase 9 plan. No orphaned requirements found.

---

### Anti-Patterns Found

No blocker or warning anti-patterns found.

- `security.txt.jinja` uses no Jinja variables intentionally — this is correct per RFC 9116 (all fields are static).
- `return StatusCode::INTERNAL_SERVER_ERROR.into_response()` in `stats_handler` on DB error is correct behavior per the plan spec; it is not a stub.
- Initial `useState` or `return []` patterns do not apply (Rust codebase).

---

### Human Verification Required

#### 1. /.well-known/security.txt served over HTTP from live honeyprompt.sh

**Test:** On the live honeyprompt.sh server, run: `curl -I https://honeyprompt.sh/.well-known/security.txt`
**Expected:** HTTP 200, Content-Type text/plain, body with Contact/Expires/Preferred-Languages fields
**Why human:** Requires a running deployment — cannot test without a live server binding.

#### 2. /stats CORS header works from honeyprompt.dev origin

**Test:** From a browser console on honeyprompt.dev, run: `fetch('https://honeyprompt.sh/stats').then(r => r.json()).then(console.log)`
**Expected:** JSON with all 8 fields, no CORS error in browser console
**Why human:** Cross-origin fetch behavior requires a real browser + real server with matching domain.

#### 3. Footer visibility on generated honeypot page

**Test:** Open the generated `output/index.html` in a browser, scroll to the bottom.
**Expected:** Footer visible with "HoneyPrompt" link (honeyprompt.dev) and "Report a Security Issue" link (GitHub advisories).
**Why human:** Visual rendering check cannot be automated.

---

### Gaps Summary

No gaps. All 5 observable truths verified. All 6 required artifacts exist and are substantive. All 5 key links are wired. Data flows from live SQLite queries through to the JSON response. Full test suite passes (131 tests, 0 failures). Clippy clean. All 5 requirement IDs (IDENT-01, IDENT-02, STATS-01, STATS-02, STATS-03) satisfied.

Three items are routed to human verification — all are deployment-environment checks that pass programmatic automated verification.

---

_Verified: 2026-03-31_
_Verifier: Claude (gsd-verifier)_
