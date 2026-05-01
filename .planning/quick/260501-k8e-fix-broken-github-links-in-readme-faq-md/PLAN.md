---
quick_id: 260501-k8e
slug: fix-broken-github-links-in-readme-faq-md
date: 2026-05-01
status: in-progress
---

# Quick Task: Fix broken GitHub links in README

## Problem

The README references `FAQ.md` (and `LICENSE`) using bare `https://github.com/johnzilla/honeyprompt/FAQ.md` URLs. GitHub requires `/blob/<branch>/<path>` to render a file in the web UI — bare paths return 404.

Reported: README has a link to FAQ.md that returns 404 on github.com.

## Affected lines

- `README.md:7` — `[FAQ](https://github.com/johnzilla/honeyprompt/FAQ.md)` → 404
- `README.md:37` — `[FAQ](https://github.com/johnzilla/honeyprompt/FAQ.md)` → 404
- `README.md:318` — `[MIT](https://github.com/johnzilla/honeyprompt/LICENSE)` → 404 (same root cause; fix in same patch)

`FAQ.md` and `LICENSE` both exist at the repo root, so no new files are needed.

## Fix

Rewrite the three URLs to use GitHub's blob path:

- `https://github.com/johnzilla/honeyprompt/FAQ.md` → `https://github.com/johnzilla/honeyprompt/blob/main/FAQ.md`
- `https://github.com/johnzilla/honeyprompt/LICENSE` → `https://github.com/johnzilla/honeyprompt/blob/main/LICENSE`

## Verification

1. `grep -n "github.com/johnzilla/honeyprompt/FAQ.md\|github.com/johnzilla/honeyprompt/LICENSE" README.md` — should return no matches after the fix.
2. `grep -c "blob/main/FAQ.md" README.md` — should return 2.
3. `grep -c "blob/main/LICENSE" README.md` — should return 1.

## Out of scope

- Other broken links in the repo (not reported, not investigating).
- Changing the FAQ content itself.
