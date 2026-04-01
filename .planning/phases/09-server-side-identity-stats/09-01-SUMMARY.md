---
phase: 09-server-side-identity-stats
plan: "01"
subsystem: generator
tags: [identity, security-txt, rfc-9116, footer, templates]
dependency_graph:
  requires: []
  provides: [footer-on-generated-pages, security-txt-well-known]
  affects: [generator, assets/templates]
tech_stack:
  added: []
  patterns: [jinja-template-static-file, rust-embed-asset-embedding]
key_files:
  created:
    - assets/templates/security.txt.jinja
  modified:
    - assets/templates/index.html.jinja
    - src/generator/mod.rs
    - tests/test_generate.rs
decisions:
  - "security.txt generated as static file to output/.well-known/ by generator (not served dynamically)"
  - "GitHub Security Advisories URL used as Contact field per design review decision"
  - "Expires set to 2027-12-31T23:59:59z (static, ~2 years out)"
metrics:
  duration: "~8 minutes"
  completed: "2026-04-01T01:48:43Z"
  tasks_completed: 2
  files_changed: 4
---

# Phase 09 Plan 01: Identity and Disclosure Artifacts Summary

Footer with project identity and RFC 9116 security.txt added to all generated honeypot sites.

## What Was Built

- **Footer in index.html.jinja:** Visible footer before `</body>` with `honeyprompt.dev` link and GitHub Security Advisories disclosure contact. Renders on every generated page.
- **security.txt.jinja template:** RFC 9116 compliant template with `Contact:`, `Expires:`, and `Preferred-Languages:` fields. Static content, no Jinja variables needed.
- **Generator updated:** `generate()` now renders `security.txt.jinja` and writes to `output/.well-known/security.txt` after creating the `.well-known/` subdirectory.
- **4 new test assertions:** `test_generate_security_txt`, `test_generate_html_has_footer` (integration), plus assertions in `test_generate_creates_output_files` and `test_render_template_index`.

## Tasks

### Task 1: Add footer to index.html.jinja and create security.txt.jinja
- Inserted footer block before `</body>` with `honeyprompt.dev` and security advisories link
- Created `assets/templates/security.txt.jinja` with 3 RFC 9116 required fields
- Commit: `63b2da2`

### Task 2: Update generator and tests
- Added `render_template("security.txt.jinja", context! {})` call in `generate()`
- Created `output/.well-known/` directory and wrote `security.txt`
- Added 2 new integration tests and 2 new inline assertions
- All 131 tests pass (88 unit + 43 integration), clippy clean
- Commit: `cbd7ad8`

## Verification

- `cargo test` — 131 tests pass, 0 failures
- `cargo clippy -- -D warnings` — no warnings
- `grep "well-known" src/generator/mod.rs` — matches
- `grep "security.txt" src/generator/mod.rs` — matches
- `grep "honeyprompt.dev" assets/templates/index.html.jinja` — matches

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all content is static and fully rendered. The `security.txt.jinja` uses no Jinja variables intentionally (RFC 9116 fields are fixed values).

## Self-Check: PASSED

- `assets/templates/security.txt.jinja` — FOUND
- `assets/templates/index.html.jinja` (modified) — FOUND
- `src/generator/mod.rs` (modified) — FOUND
- `tests/test_generate.rs` (modified) — FOUND
- Commit `63b2da2` — FOUND
- Commit `cbd7ad8` — FOUND
