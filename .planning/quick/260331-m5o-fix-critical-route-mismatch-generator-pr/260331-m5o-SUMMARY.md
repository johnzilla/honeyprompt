---
phase: quick
plan: 260331-m5o
subsystem: generator
tags: [bugfix, routing, callback-url, critical]
dependency_graph:
  requires: []
  provides: [correct-callback-urls-in-generated-output]
  affects: [server, test-agent, end-to-end-detection-pipeline]
tech_stack:
  added: []
  patterns: []
key_files:
  modified:
    - src/generator/mod.rs
    - tests/test_generate.rs
  created:
    - test-site/output/index.html
    - test-site/output/callback-map.json
    - test-site/output/ai.txt
    - test-site/output/robots.txt
    - test-site/honeyprompt.toml
key_decisions:
  - "Generator callback URLs fixed to /cb/v1/{nonce} to match Axum route in server/mod.rs"
metrics:
  duration: "~10 min"
  completed: "2026-03-31T20:02:09Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 2
  files_created: 5
---

# Quick Task 260331-m5o: Fix Critical Route Mismatch Generator PR Summary

**One-liner:** Fixed generator callback URLs from bare `/cb/{nonce}` to `/cb/v1/{nonce}` to match the Axum server route, restoring end-to-end callback detection.

## What Was Done

### Task 1: Fix callback URL format strings and update test assertion

All 4 `format!` calls in `src/generator/mod.rs` that produced callback URLs were using the wrong path segment:

- Tier1 (line 63): `/cb/{nonce}` → `/cb/v1/{nonce}`
- Tier2 nonce_a (line 87): `/cb/{nonce}` → `/cb/v1/{nonce}`
- Tier2 nonce_b (line 88): `/cb/{nonce}` → `/cb/v1/{nonce}`
- Tier3 (line 126): `/cb/{nonce}` → `/cb/v1/{nonce}`

The integration test assertion in `tests/test_generate.rs` (test `test_generate_html_has_payloads`) was also tightened from `/cb/` to `/cb/v1/` so it actually validates the versioned path segment.

**Commit:** 7470234

### Task 2: Regenerate test-site output with corrected URLs

Ran the fixed `honeyprompt generate` binary against `test-site/` to regenerate all output files. Verified 6 `/cb/v1/` occurrences in `index.html` and 8 in `callback-map.json`. No bare `/cb/` without `v1` remain in any generated file.

**Commit:** c736588

## Verification Results

- `cargo test`: 88 tests pass across all test suites (unit, integration, doc)
- `grep -r "/cb/" src/generator/mod.rs` shows only `/cb/v1/` entries
- `grep -rn "cb/" test-site/output/index.html | grep -v "v1"` returns nothing
- Server route in `src/server/mod.rs` (`/cb/v1/{nonce}`) now matches generated URLs

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Self-Check: PASSED

- src/generator/mod.rs: FOUND
- tests/test_generate.rs: FOUND
- test-site/output/index.html: FOUND (committed c736588)
- test-site/output/callback-map.json: FOUND (committed c736588)
- Commits 7470234, c736588: FOUND in git log
