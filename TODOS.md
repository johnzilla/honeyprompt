# TODOS

## Shipped

- **Tier 4: Capability Introspection** — shipped in v5.0 (Phases 13–15). Agents self-report sorted, base64-encoded tool lists via `/cb/v4/{nonce}/{b64_list}`.
- **Tier 5: Multi-step Compliance Chain** — shipped in v5.0 (Phases 13–15). Agents extract a page-visible seed, apply a deterministic formula, and submit a 3-digit proof the server re-verifies via `/cb/v5/{nonce}/{proof}`.

## Set up project-specific disclosure email
- **What:** Configure security@honeyprompt.dev (or similar) and update security.txt Contact field
- **Why:** RFC 9116 security.txt currently uses GitHub Security Advisories URL as contact. A dedicated email is more professional and discoverable.
- **Pros:** Direct communication channel for disclosure. More trusted by security researchers.
- **Cons:** Requires DNS MX records or email forwarding service. Ongoing maintenance.
- **Context:** security.txt ships with GitHub Security Advisories URL as v1 contact. This TODO captures the upgrade to a real email once honeyprompt.dev DNS is configured. MX records or a forwarding service (e.g., ImprovMX, Cloudflare Email Routing) needed.
- **Depends on:** honeyprompt.dev DNS setup (this plan)
- **Added:** 2026-03-31 via /plan-eng-review
