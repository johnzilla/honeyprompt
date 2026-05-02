---
title: v6.0 dual-product expansion — federation primitives + canonical benchmark layer
planted_date: 2026-05-02
trigger_condition: ≥3 unsolicited use signals from people outside immediate circle within 60 days of v5.1 writeup publication
status: parked
---

# Seed: v6.0 dual-product expansion

## Idea (in one paragraph)

Expand HoneyPrompt from a single-project Rust binary into a **dual-product**:

1. **Open-source layer** — Rust binary, payload catalog, callback infra. Stays fully self-hostable, no regression, Apache 2.0. Adds federation primitives: site keys, `--report-to` flag, multi-tenant SQLite (additive migration), so self-hosters can optionally feed observational data to the canonical layer.
2. **Canonical layer** — `honeyprompt.dev` (editorial: benchmark leaderboard, fingerprint census, methodology docs) + `honeyprompt.sh` (operational: live canary site, callback endpoint, eval target agents run against). Versioned methodology, reproducibility commitment, trademark on "HoneyPrompt" name (Linux Foundation / Mozilla pattern: free downstream forks, protected branding upstream).

Reframes the project from "honeypot in the wild" to "reproducible safety-eval suite for browsing agents" (LMSys Arena / HELM / MTBench / AgentBench lineage).

## Why parked, not killed

Deferred on 2026-05-02 because the strategic edifice rested on an untested demand assumption (researchers/vendor red-teamers will care). No external citations, no third-party deployments, no use signals from outside immediate circle to date. Building federation + multi-tenant + hosted infrastructure for users who haven't been validated to exist is the path most likely to produce regret.

The v5.1 validation experiment (run named browsing agents through HoneyPrompt, publish results, distribute) is the cheap test that resolves the question. If signal comes back, this seed surfaces and v6.0 is justified.

## Trigger conditions

Surface this seed for reconsideration if **any** of the following fires within 60 days of the v5.1 writeup publication date:

- ≥3 unsolicited "hey I'm using this" notes from people outside immediate circle (email, GitHub issues, Discord/Slack DMs, blog mentions)
- ≥1 citation in a published security writeup, academic paper, or vendor disclosure document
- ≥1 PR or substantive issue from a contributor not previously involved
- ≥10 GitHub stars from accounts identifiable as researchers, vendor red-team engineers, or AI security folk (not generic "starred for later" accounts)
- A benchmark author / leaderboard maintainer reaches out asking to incorporate HoneyPrompt as an eval dimension

If *none* of those fire after 60 days of distributed effort, this seed should be marked `status: rejected` rather than `parked` — the answer was no, and that's a clean answer.

## What's *in* the v6.0 envelope (if it ever ships)

- Federation primitives in OSS: site keys, `--report-to` flag, multi-tenant SQLite (additive migration, won't break self-hosters)
- Canonical site infrastructure on `honeyprompt.dev`: leaderboard renderer, fingerprint census dashboard, methodology version index
- First benchmark v1.0 publication: 5+ reproducible browsing-agent harnesses, ~20 natural research tasks, methodology paper draft
- Trademark / license / governance posture: explicit policy, contribution guide, license header sweep
- Disclosure / contribution policy doc

## What's explicitly *not* in v6.0 even if triggered

- Coordinated disclosure / embargo machinery (frame is behavioral measurement, not vuln research)
- Vendor leaderboard / naming-and-shaming (fingerprint census instead — community/press do attribution)
- Site-owner SaaS pitch / snippet-based JS library (different audience, different product, separate seed if it ever surfaces)
- Foundation / non-profit structure (premature; revisit only if scale forces it)

## Strategic context

See `.planning/notes/honeyprompt-direction-2026-05-02.md` for the full reasoning trail behind the deferral, including audience analysis, attribution / fingerprint-registry framing, and rejected alternatives (vuln management, droplet mesh + SEO, site-owner SaaS).
