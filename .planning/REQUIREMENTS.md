# Requirements: v3.0 Public Presence

## Identity & Disclosure

- [ ] **IDENT-01**: Honeypot page includes a footer with project name, honeyprompt.dev link, and disclosure contact visible to human visitors
- [ ] **IDENT-02**: /.well-known/security.txt served from honeyprompt.sh with valid RFC 9116 fields (Contact, Expires, Preferred-Languages)

## Stats API

- [ ] **STATS-01**: GET /stats returns JSON with aggregate callback counts (total_sessions, detection_sessions, crawler_sessions, per-tier counts, earliest/latest event timestamps)
- [ ] **STATS-02**: /stats response includes Access-Control-Allow-Origin: * header for cross-origin fetch from honeyprompt.dev
- [ ] **STATS-03**: /stats returns all-zero counts on empty database (not an error response)

## Landing Page

- [ ] **LAND-01**: honeyprompt.dev serves a single-page static site from docs/ folder on main branch via GitHub Pages with custom domain
- [ ] **LAND-02**: Landing page fetches /stats from honeyprompt.sh and displays live aggregate counts with descriptive tier labels (URL Fetch, Conditional, Composed)
- [ ] **LAND-03**: Landing page shows terminal-style loading state (blinking cursor) and graceful error fallback message when stats are unavailable
- [ ] **LAND-04**: Landing page uses JetBrains Mono font, GitHub dark palette (#0d1117 bg, #e6edf3 text, #3fb950 green accent), accessible (WCAG AA contrast, 44px touch targets, visible focus rings, semantic HTML)

## Future Requirements

- Disclosure email (security@honeyprompt.dev) replacing GitHub Advisories URL in security.txt
- Auto-refresh polling on landing page stats counter
- Per-agent breakdown dashboard on honeyprompt.dev

## Out of Scope

- Full design system / DESIGN.md — single-page site, tokens pinned in plan
- Dark/light mode toggle — always dark, security tool aesthetic
- Analytics/tracking on landing page — privacy-first tool
- Blog/changelog section — ship tool page first
- crates.io publish workflow — deferred (existing TODO)

## Traceability

| REQ-ID | Phase | Plan | Status |
|--------|-------|------|--------|
| IDENT-01 | — | — | Pending |
| IDENT-02 | — | — | Pending |
| STATS-01 | — | — | Pending |
| STATS-02 | — | — | Pending |
| STATS-03 | — | — | Pending |
| LAND-01 | — | — | Pending |
| LAND-02 | — | — | Pending |
| LAND-03 | — | — | Pending |
| LAND-04 | — | — | Pending |
