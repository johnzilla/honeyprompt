---
phase: 01-generation-pipeline
plan: 03
subsystem: generation
tags: [rust, minijinja, jinja2, rust-embed, rusqlite, honeypot, templates, cli]

# Dependency graph
requires:
  - phase: 01-generation-pipeline-01-01
    provides: CLI structure (Cli, Commands, InitArgs, GenerateArgs), Config struct, config load/write
  - phase: 01-generation-pipeline-01-02
    provides: Payload catalog loader, nonce generator, SQLite store with nonce_map table

provides:
  - Jinja2 HTML template with hard-coded warning banner and 5 embedding location slots
  - robots.txt template with AI bot disallow rules (GPTBot, ClaudeBot, Google-Extended, etc.)
  - ai.txt template with agent policy declarations
  - generator::generate() pipeline: catalog load -> nonce generation -> template rendering -> file output
  - Working `honeyprompt init` command creating full project scaffold
  - Working `honeyprompt generate` command producing deployable 4-file output
  - Integration test suite (test_init.rs + test_generate.rs, 12 tests)

affects:
  - phase 2 (server): callback URLs embed nonces for server-side lookup via nonce_map table
  - phase 3 (tui): output structure (output/ dir, callback-map.json) defines monitoring data source

# Tech tracking
tech-stack:
  added:
    - minijinja 2.18 (Jinja2 template rendering for HTML, robots.txt, ai.txt)
    - rust-embed (embedded templates in binary, separate Templates struct from CatalogAssets)
  patterns:
    - Embedded assets via RustEmbed derive macro, accessed via ::get() and safe UTF-8 parsing
    - minijinja Environment per render call with auto-escaping; | safe filter for raw HTML payloads
    - Payload nonce substitution by tier type (Tier1: single URL, Tier2: two URLs, Tier3: base URL)
    - RenderedPayload intermediate struct decouples catalog Payload from Jinja template context

key-files:
  created:
    - assets/templates/index.html.jinja
    - assets/templates/robots.txt.jinja
    - assets/templates/ai.txt.jinja
    - src/generator/mod.rs
    - tests/test_init.rs
    - tests/test_generate.rs
  modified:
    - src/lib.rs (added pub mod generator)
    - src/main.rs (replaced stubs with full init/generate implementations)

key-decisions:
  - "minijinja auto-escapes HTML by default — all rendered payload instructions use | safe filter to prevent double-encoding of HTML, URLs, and special chars in canary content"
  - "RenderedPayload intermediate struct (embedding_location: String, rendered_instruction: String) decouples catalog types from Jinja context serialization"
  - "Tier 2 payloads generate two distinct nonces (callback_url_a, callback_url_b) — both inserted in nonce_map for server-side lookup of either branch"

patterns-established:
  - "Template rendering: create fresh minijinja Environment per call, embed source via Templates::get(), render with context! macro"
  - "Init scaffold: output/, .honeyprompt/overrides/, honeyprompt.toml, .honeyprompt/events.db"
  - "Generator output: output/index.html, output/robots.txt, output/ai.txt, output/callback-map.json"

requirements-completed: [CLI-01, CLI-02, GEN-01, GEN-02, GEN-04, GEN-05, GEN-06]

# Metrics
duration: 35min
completed: 2026-03-29
---

# Phase 1 Plan 03: Generation Pipeline Summary

**Complete init+generate CLI pipeline producing deployable honeypot with hard-coded warnings, 5-location payload embedding, nonce-keyed callbacks, robots.txt AI disallows, and ai.txt policy declarations**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-03-29T02:00:00Z
- **Completed:** 2026-03-29T02:35:00Z
- **Tasks:** 3 of 3 complete (Task 3 human-verify checkpoint approved)
- **Files modified:** 8

## Accomplishments

- Jinja2 templates with hard-coded warning banner (`id="warning-banner"`) and inline notice — not configurable by design (GEN-02)
- Five payload embedding locations wired in template: html_comment, meta_tag, invisible_element, json_ld, semantic_prose
- Generator pipeline: catalog load -> per-tier nonce generation -> store insertion -> template render -> 4-file output
- `honeyprompt init` creates full scaffold: honeyprompt.toml, output/, .honeyprompt/overrides/, .honeyprompt/events.db
- `honeyprompt generate` produces all 4 output files with live callback URLs
- 12 integration tests covering scaffold creation, re-init guard, config validity, output files, warning presence, payload locations, nonce format/uniqueness, robots.txt AI bots, ai.txt policy, callback-map structure

## Task Commits

Each task was committed atomically:

1. **Task 1: Templates and generator module** - `27b3d35` (feat)
2. **Task 2: Wire init and generate commands with integration tests** - `8198334` (feat)
3. **Task 3: Verify generated honeypot page** - APPROVED (checkpoint:human-verify — user confirmed page functional, feedback noted)

## Files Created/Modified

- `assets/templates/index.html.jinja` - Honeypot HTML with hard-coded warning banner, | safe filter on all 5 embedding location slots
- `assets/templates/robots.txt.jinja` - AI bot disallow rules (GPTBot, ClaudeBot, anthropic-ai, Google-Extended, PerplexityBot, CCBot)
- `assets/templates/ai.txt.jinja` - Agent policy declarations (Disallow: Scraping/Training/Indexing)
- `src/generator/mod.rs` - Template rendering pipeline, per-tier nonce substitution, callback-map serialization
- `src/lib.rs` - Added pub mod generator
- `src/main.rs` - Full init and generate command implementations replacing stubs
- `tests/test_init.rs` - 3 integration tests for init command
- `tests/test_generate.rs` - 9 integration tests for generate command

## Decisions Made

- minijinja HTML auto-escaping required adding `| safe` filter to all rendered_instruction slots — without it, callback URLs like `http://localhost:8080/cb/abc123` were rendered as `http:&#x2f;&#x2f;localhost:8080&#x2f;cb&#x2f;abc123`, breaking the canary functionality
- Tier 2 payloads generate two distinct nonces, both stored in `nonce_map` — this preserves server-side lookup capability for either branch of the conditional test
- RenderedPayload intermediate struct (not derived from Payload) keeps template context clean and avoids serializing private catalog fields into HTML

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added | safe filter to all payload embedding slots in index.html.jinja**
- **Found during:** Task 2 (integration test test_generate_html_has_payloads failed)
- **Issue:** minijinja auto-escapes HTML content in templates by default. Rendered instructions containing HTML chars (/, ", ', <, >) were double-encoded, breaking callback URLs and making HTML tags visible as escaped text
- **Fix:** Added `| safe` filter to all 5 `{{ p.rendered_instruction }}` occurrences in index.html.jinja
- **Files modified:** assets/templates/index.html.jinja
- **Verification:** `test_generate_html_has_payloads` and `test_generate_html_multiple_locations` pass; visual inspection of /tmp/hp-verify/output/index.html confirms raw HTML tags and proper URLs
- **Committed in:** 8198334 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug)
**Impact on plan:** Required fix for correct payload delivery. The | safe filter is appropriate because the rendered instructions are trusted catalog content, not user input.

## Issues Encountered

None beyond the auto-fixed escaping bug above.

## Human Verification Feedback

**Checkpoint approved** — user confirmed the generated page is functional.

**Feedback noted (deferred improvement):** The generated honeypot page content is repetitive — the same message text appears repeated across multiple embedding locations. This is because the current payload catalog entries for semantic_prose and other locations use similar instruction text.

**Decision:** Not fixed in this plan. The payload catalog content diversification is a future improvement. The repetitiveness does not affect functional correctness — nonces are unique, all embedding locations are populated, and warning banners are present. The issue should be addressed when the payload catalog is expanded or refined (likely Phase 1 follow-up or Phase 2 planning).

**Deferred item logged:** Diversify payload instruction text across embedding locations so each location presents distinct content to the reader/scanner.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- Phase 1 complete: `honeyprompt init` and `honeyprompt generate` produce fully deployable 4-file output
- Phase 2 (HTTP server) can now build on the nonce_map table and existing DB schema
- The `/cb/{nonce}` URL pattern established here is the contract for Phase 2's callback handler
- Concern carried forward: Phase 2 must decide per-visitor nonce injection vs static nonce generation (STATE.md blocker)

---
*Phase: 01-generation-pipeline*
*Completed: 2026-03-29*

## Self-Check: PASSED

All files verified present. All task commits verified in git log:
- 27b3d35: templates and generator module
- 8198334: init/generate commands and integration tests
- 12554f3: metadata commit (SUMMARY, STATE, ROADMAP)

Task 3 (human-verify checkpoint) approved by user on 2026-03-28. Feedback about repetitive content noted as deferred improvement.
