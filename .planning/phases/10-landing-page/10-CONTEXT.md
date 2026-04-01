# Phase 10: Landing Page - Context

**Gathered:** 2026-03-31
**Status:** Ready for planning

<domain>
## Phase Boundary

honeyprompt.dev serves a single-page static site from docs/ on main branch via GitHub Pages. The page displays project identity, live aggregate stats from the /stats endpoint (implemented in Phase 9), a How It Works explanation, and a Quick Start guide. No frameworks, no build step, single HTML file.

</domain>

<decisions>
## Implementation Decisions

### Information Architecture (from design review)
- **D-01:** Section order: Hero (name + tagline) → Live Stats → How It Works → Quick Start → Footer
- **D-02:** Stats section is above How It Works. The data is the pitch, explanation comes after proof.

### Copy & Content
- **D-03:** Tagline: "Honeypot canaries for AI browsing agents"
- **D-04:** How It Works steps: Generate → Deploy → Detect. "Generate a honeypot with embedded canaries" → "Deploy to any public URL" → "Watch agents trigger callbacks in real-time"
- **D-05:** How It Works displayed as ASCII pipeline diagram, NOT a card grid. Left-aligned.
- **D-06:** Quick Start shows `cargo install` first, binary download as alternative below.
- **D-07:** Footer includes: GitHub repo link, responsible disclosure (GitHub Security Advisories), MIT license.

### Stats Display
- **D-08:** Inline terminal output style: `$ honeyprompt stats\n  sessions: 47  |  url_fetch: 12  |  conditional: 8  |  composed: 3`
- **D-09:** Descriptive tier labels: "url_fetch", "conditional", "composed" (lowercase, underscore, terminal style)
- **D-10:** Loading state: `Connecting...` with blinking block cursor (CSS animation)
- **D-11:** Error state: `Stats unavailable. Honeypot may be offline.` with link to GitHub repo
- **D-12:** Zero-count state: Show zeros normally (the endpoint returns valid JSON with zeros)

### Design Tokens (from design review)
- **D-13:** Font: JetBrains Mono via Google Fonts, fallback to monospace
- **D-14:** Background: #0d1117, text: #e6edf3, accent green: #3fb950, accent amber: #d29922
- **D-15:** Max-width: 720px centered, padding: 2rem, spacing: 8px base grid
- **D-16:** No borders except 1px #30363d for section dividers
- **D-17:** Left-aligned body text. Only the hero project name is centered.

### Responsive & Accessibility (from design review)
- **D-18:** Breakpoint at 768px: reduce padding to 1rem, stats display wraps naturally (single line becomes multi-line)
- **D-19:** Touch targets: all links min 44px height
- **D-20:** Visible focus ring: 2px #3fb950 outline on all interactive elements
- **D-21:** Semantic HTML: h1 for project name, h2 for sections, nav for footer links
- **D-22:** WCAG AA contrast: all proposed color combinations pass

### GitHub Pages Setup
- **D-23:** docs/ folder on main branch. Single index.html file + CNAME file.
- **D-24:** CNAME file contains `honeyprompt.dev`. DNS configuration is a human checkpoint.
- **D-25:** Plan includes DNS verification checkpoint before marking deployment complete.

### Claude's Discretion
- HTML structure and CSS specifics within the design token constraints
- Exact spacing values within the 8px grid system
- fetch() implementation details (timeout, retry logic)
- Meta tags (title, description, og:tags) for SEO basics

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design Review Decisions
- `.planning/STATE.md` §Decisions — All v3.0 eng review and design review decisions

### Phase 9 Implementation (stats endpoint)
- `src/server/mod.rs` — stats_handler() returns JSON with ReportSummary fields. Landing page fetch() must match this shape.
- `src/store/mod.rs` — ReportSummary struct defines the JSON field names: total_sessions, detection_sessions, crawler_sessions, tier1_sessions, tier2_sessions, tier3_sessions, earliest_event, latest_event

### Design Doc
- `~/.gstack/projects/johnzilla-honeyprompt/john-main-design-20260331-152120.md` — Original design doc with approach rationale

### Eng Review Test Plan
- `~/.gstack/projects/johnzilla-honeyprompt/john-main-eng-review-test-plan-20260331-153552.md` — QA test plan for the landing page

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None in docs/ (new directory). The landing page is standalone HTML, not built from the Rust codebase.

### Established Patterns
- The honeypot template (assets/templates/index.html.jinja) is NOT a reference for the landing page style. The honeypot is intentionally plain; the landing page has its own dark terminal aesthetic.

### Integration Points
- GET https://honeyprompt.sh/stats returns JSON with CORS: * header. Landing page fetches this on page load.
- docs/CNAME file tells GitHub Pages which custom domain to serve.

</code_context>

<specifics>
## Specific Ideas

- Stats styled as terminal output, not dashboard counters. Should look like you ran a command and got results.
- "The data IS the pitch" — stats section is visually prominent, not a sidebar widget.
- Security researcher audience: dense, technical, no marketing fluff.
- Reference aesthetic: Nmap, Wireshark, Shodan landing pages. Terminal-first.

</specifics>

<deferred>
## Deferred Ideas

- Auto-refresh polling on stats (setInterval) — trivial to add later, not needed for v1
- Per-agent breakdown or Wall of Shame dashboard — Approach C from design doc, deferred
- Blog or changelog section — ship tool page first

</deferred>

---

*Phase: 10-landing-page*
*Context gathered: 2026-03-31*
