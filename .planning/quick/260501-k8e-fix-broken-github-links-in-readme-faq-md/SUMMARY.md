---
quick_id: 260501-k8e
slug: fix-broken-github-links-in-readme-faq-md
date: 2026-05-01
status: complete
---

# Summary: Fix broken GitHub links in README

## Outcome

Fixed three 404-returning links in `README.md`. Reported issue was the FAQ link; same root cause (missing `/blob/main/` segment) also affected the LICENSE link, fixed in the same patch.

## Changes

| File | Lines | Change |
|------|-------|--------|
| `README.md` | 7 | `…/honeyprompt/FAQ.md` → `…/honeyprompt/blob/main/FAQ.md` |
| `README.md` | 37 | `…/honeyprompt/FAQ.md` → `…/honeyprompt/blob/main/FAQ.md` |
| `README.md` | 318 | `…/honeyprompt/LICENSE` → `…/honeyprompt/blob/main/LICENSE` |

## Verification

- `grep -nE "github\.com/johnzilla/honeyprompt/(FAQ\.md\|LICENSE)"` filtered to non-`/blob/` matches → 0 hits
- `blob/main/FAQ.md` count → 2 (matches the two FAQ references)
- `blob/main/LICENSE` count → 1 (matches the one LICENSE reference)

Both `FAQ.md` and `LICENSE` exist at the repo root, so no new files were needed — pure URL-pattern fix.

## Commits

- `5ca281d` — docs: fix 404 GitHub links in README (FAQ.md + LICENSE)

## Notes

Doc-only change; skipped the Rust fmt/clippy/test pipeline (no Rust code touched).
