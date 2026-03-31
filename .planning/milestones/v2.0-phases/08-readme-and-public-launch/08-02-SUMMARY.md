---
phase: 08-readme-and-public-launch
plan: "02"
subsystem: infra
tags: [launch, seo, social-media, x-post, google-search-console]

# Dependency graph
requires:
  - phase: 08-01
    provides: Polished README with live demo link, corrected URLs, ethics section
provides:
  - Draft X post copy (3 variations) for user to post at discretion
  - Google Search Console submission steps presented to user
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: []

key-files:
  created:
    - .planning/phases/08-readme-and-public-launch/x-post-drafts.md
  modified: []

key-decisions:
  - "X post drafting only — user posts manually (D-06)"
  - "Google Search Console submission is a human-only task (D-07)"

patterns-established: []

requirements-completed:
  - LAUNCH-02

# Metrics
duration: 5min
completed: 2026-03-31
---

# Phase 8 Plan 02: Public Launch Tasks Summary

**3 X post draft variations (under 280 chars, GitHub-linked) ready for user; Google Search Console submission steps presented as human-action checkpoint**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-03-31T17:20:22Z
- **Completed:** 2026-03-31T17:25:00Z
- **Tasks:** 1 of 2 (Task 2 is a human-action checkpoint)
- **Files modified:** 1

## Accomplishments

- Drafted 3 X post variations announcing HoneyPrompt — all under 280 characters, all linking to github.com/johnzilla/honeyprompt
- Variations A and C also reference the live demo at honeyprompt.sh
- Google Search Console submission steps documented for user to complete manually

## X Post Draft Variations

### Variation A — Problem-first, punchy (232 chars)

> AI agents follow prompt injection from untrusted web pages. HoneyPrompt proves it — deploy a honeypot, watch the callbacks come in. Open source, written in Rust. https://github.com/johnzilla/honeyprompt — live demo at honeyprompt.sh

### Variation B — Tool description, threat context (255 chars)

> HoneyPrompt: generate honeypot pages with hidden prompt-injection canaries, serve them, and record every AI agent that follows the injected instructions. Graduated evidence, no secrets collected. Rust, open source. https://github.com/johnzilla/honeyprompt

### Variation C — Minimal, direct (233 chars)

> Deployed a honeypot to see if AI agents follow prompt injection from untrusted pages. They do. HoneyPrompt records the callbacks as proof — open source, written in Rust. https://github.com/johnzilla/honeyprompt — demo: honeyprompt.sh

## Google Search Console Steps (Human Action Required)

1. Go to https://search.google.com/search-console
2. Click "Add property" and select "URL prefix"
3. Enter: `https://honeyprompt.sh`
4. Complete domain verification (DNS TXT record, HTML file upload, or other method)
5. Once verified, go to URL Inspection
6. Enter: `https://honeyprompt.sh`
7. Click "Request Indexing"

## Task Commits

Each task was committed atomically:

1. **Task 1: Draft X post copy for launch** - `56e47aa` (chore)
2. **Task 2: Submit to Google Search Console** — human-action checkpoint (no commit)

**Plan metadata:** (see final commit)

## Files Created/Modified

- `.planning/phases/08-readme-and-public-launch/x-post-drafts.md` — 3 draft X post variations

## Decisions Made

- D-06: X post content is drafted for user — user posts manually, no automation attempted
- D-07: Google Search Console submission is human-only — documented steps, presented as checkpoint

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

**Google Search Console requires manual configuration:**
- Add and verify `honeyprompt.sh` property in Search Console
- Request indexing for `https://honeyprompt.sh` via URL Inspection

## Next Phase Readiness

- Phase 8 (README and Public Launch) is the final phase of v2.0
- After Google Search Console submission, v2.0 milestone is complete
- Deferred: HN Show post, Reddit r/netsec post — pending X post traction

---
*Phase: 08-readme-and-public-launch*
*Completed: 2026-03-31*
