---
title: HoneyPrompt direction — strategic reasoning behind v5.1 validation pivot
date: 2026-05-02
context: explore session pressure-testing whether to expand HoneyPrompt to v6.0 (dual-product split) or take a different path
type: note
---

# HoneyPrompt direction — 2026-05-02

Durable record of the strategic reasoning from the `/gsd-explore` session that led to deferring v6.0 in favor of a one-shot validation experiment (v5.1).

## Starting question

Should HoneyPrompt expand into v6.0 (dual-product: open-source primitives + canonical benchmark/data layer at `honeyprompt.dev` + `honeyprompt.sh`), or stay at v5.0 and move on?

## Honest motivation assessment

The v6.0 expansion impulse was driven by, in honest order:

1. **Genuine excitement about the technical work** (federation primitives, benchmark methodology, agent harnesses sound fun to build) — primary
2. **Wanting external validation** — would feel great to get a "hey, I'm using this" note from someone outside the immediate circle — secondary
3. **Portfolio value** — substantive finished side project in a hot domain (AI security) — tertiary

Critically: *not* filling time. Has many other ideas competing for attention; opportunity cost is the binding constraint.

## Audience analysis

Earlier conversation eliminated several candidate audiences:

- **Site owners worried about agent abuse** → considered, rejected. At best a compliance issue; unclear anyone would pay attention.
- **Vuln-management / coordinated disclosure** → rejected. Prompt-injection compliance isn't a CVE-shape vulnerability (canaries can't exfiltrate secrets, can't escalate privilege — ethics work already established this). The right frame is *behavioral measurement*, not vulnerability research. This avoids embargo machinery, intake forms, and vendor-relationship overhead.
- **Vendor leaderboard / naming-and-shaming** → rejected. Attribution is genuinely hard (UA strings unreliable, IP/ASN ambiguous). Confidence to say "Vendor X bad" is shaky without UA self-identification or vendor cooperation.

Audiences that *do* fit:

- **Researchers / vendor red-teamers** (primary) — need a canonical evaluation target that gets cited in benchmark suites and security papers. The `test-agent` CLI and a stable canary catalog are what they consume.
- **Press / vendor accountability** (secondary) — interested in aggregate behavioral patterns, not individual catches.

## Reframings that landed

1. **Behavioral measurement, not vulnerability research.** No embargo windows, no disclosure policy, no vendor relationships required. Lineage: METR / Apollo Research / AISI evals / LMSys Arena / HELM, *not* CVE coordination.
2. **Fingerprint registry, not vendor leaderboard.** Publish behavioral profiles (header tuples, timing patterns, tier-compliance distributions per fingerprint). Community/press do attribution downstream. Vendors who want correct attribution will self-identify; ones who don't, won't, and that asymmetry is itself data. Sidesteps the legal/PR exposure of naming companies wrongly. Lineage: JA3/JA4, Spamhaus, Project Honeypot.
3. **Discovery problem solved by controlled experiment, not SEO/PBN.** Running named browsing agents through tasks that lead them to the canary is *first-party attribution data* — no fingerprint guesswork, no Google indexing dance. Lineage: AgentBench, WebArena, OSWorld.
4. **Dual-product split (eventually): OSS primitives + canonical layer.** OSS stays free / self-hostable / unrestricted. Canonical layer (`honeyprompt.dev` editorial, `honeyprompt.sh` operational) is the citation target with versioned methodology and a reproducibility commitment. Lineage: Sigstore + sigstore.dev, lm-eval-harness + HELM, PostgreSQL + Supabase, Linux Foundation / Mozilla trademark posture.

## The pivot: validate before building v6.0

Pressure-testing surfaced that the entire v6.0 strategic edifice rested on an untested assumption: *"researchers and vendor red-teamers will care about a HoneyPrompt benchmark."*

Honest answer to the demand-reality question:

- No external citations to date
- No third-party deployments
- No "hey I'm using this" notes from people outside immediate circle
- No issues/PRs from real users

Conclusion: building v6.0 infrastructure (federation, multi-tenant, hosted) for users that haven't been validated to exist is the path most likely to produce regret in 6 months. **A 1-2 week validation experiment is dramatically cheaper than a multi-month build.**

Also: portfolio value is *already mostly captured at v5.0*. A finished Rust binary with a novel mechanism, ethics-bounded design, 15 disciplined GSD phases, CI, docs, and a live demo is more impressive as a portfolio piece than an in-progress v6.0. v6.0 only adds portfolio value if it succeeds externally.

## Decision

Defer v6.0. Run a one-shot validation experiment as v5.1.

Smart refinement (user's contribution): rather than a standalone HoneyPrompt-only writeup, **piggyback on an existing benchmark publication**. Take the top N from a curated leaderboard (HuggingFace OpenLLM, AgentBench, OSWorld, LMSys, etc.), run them through HoneyPrompt with a browsing harness (likely BrowserUse on a DO droplet), publish results as either a standalone post or — better — an add-on contribution to a benchmark author's regular report. Selection bias inoculated by piggyback (their list, their reputation), distribution problem solved by piggyback (their audience).

## What's parked, not abandoned

The v6.0 dual-product idea is preserved as `.planning/seeds/v6-dual-product-expansion.md` with measurable trigger conditions. If validation generates real external signal, it surfaces automatically. If not, it stays parked without occupying mental space.

## What's *not* in scope going forward

Until v5.1 validation generates signal:

- Multi-tenancy / site keys / federation primitives
- Hosted SaaS layer
- Methodology paper / formal benchmark version stability
- Trademark policy / governance structures
- Coordinated disclosure / vendor relationships
- Droplet mesh / AI-content sites / Search Console submission (rejected outright — Google deindexes this pattern, and the experiment frame replaces ambient discovery anyway)

## References

- Conversation history with Claude on 2026-05-02 (full strategic context)
- `.planning/seeds/v6-dual-product-expansion.md` (parked idea with trigger)
- ROADMAP.md → v5.1 milestone (forthcoming via `/gsd-new-milestone`)
