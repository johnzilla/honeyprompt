# Phase 8: README and Public Launch - Context

**Gathered:** 2026-03-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Polish the README (update project status, add live demo link, expand ethics section), submit honeyprompt.sh to Google Search Console for discoverability, and prepare an X post for launch. No new code — documentation and launch prep only.

</domain>

<decisions>
## Implementation Decisions

### Ethics Section
- **D-01:** Expand the existing "Safety Model" section — rename to "Ethics & Safety" and add an introductory paragraph about what HoneyPrompt is NOT (not an exploit tool, not for surveillance, not for collecting secrets). Keep the existing bullet points. Integrated, not a separate section.

### README Polish
- **D-02:** Update Project Status table — all phases 1-7 should show "Complete", Phase 8 as current.
- **D-03:** Add live demo link to honeyprompt.sh near the top of the README so visitors see it immediately.
- **D-04:** Fix GitHub URLs — currently point to `honeyprompt/honeyprompt`, should point to `johnzilla/honeyprompt`.

### Launch Channels
- **D-05:** X (Twitter) post only for initial launch. No HN Show post or Reddit r/netsec at this time. Low-key — see if anyone cares.
- **D-06:** The X post content is a human task (the user writes and posts it). The plan should draft suggested copy but not automate posting.

### Discoverability
- **D-07:** Submit honeyprompt.sh to Google Search Console — this is a manual human step (requires domain verification). Document the steps.
- **D-08:** honeyprompt.sh is already linked from the README (via the live demo link in D-03), which gives it crawler discoverability from GitHub indexing.

### Claude's Discretion
- Exact wording of the ethics paragraph
- Whether to add a "Live Demo" badge/shield at the top of README
- README section ordering if it needs rearranging

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### README
- `README.md` — Current state with install instructions from Phase 6. Needs: status update, live demo link, ethics expansion, URL fixes.

### Design Doc
- `~/.gstack/projects/johnzilla-honeyprompt/john-main-design-20260329-180748.md` — Discoverability Tactics section, Success Criteria section

### Live Demo
- `https://honeyprompt.sh` — Live and serving canary payloads (Phase 7)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `README.md` — Already has: What This Is, Proof Levels, Installation, Usage (all 6 subcommands), How It Works, Project Status, Safety Model, License. Comprehensive foundation.

### What Needs Changing
- Project Status table: Phase 6 says "In Progress", Phases 7-8 missing
- GitHub URLs: `honeyprompt/honeyprompt` → `johnzilla/honeyprompt`
- Safety Model → rename to "Ethics & Safety", add intro paragraph
- Add live demo link near top

### Integration Points
- `README.md` — Only file being modified (plus Google Search Console which is external)

</code_context>

<specifics>
## Specific Ideas

No specific requirements beyond what's captured in decisions.

</specifics>

<deferred>
## Deferred Ideas

- HN Show post — may do later if the X post gets traction
- Reddit r/netsec post — may do later
- Blog post with live demo results — deferred until there's actual data from honeyprompt.sh

</deferred>

---

*Phase: 08-readme-and-public-launch*
*Context gathered: 2026-03-31*
