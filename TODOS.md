# TODOS

Single backlog for the project. Releases are numbered; nothing else is.

## Next: security fixes (5.0.1)

From a 2026-05-04 review, plus one found 2026-11-19. No users yet, so breaking changes and a full nonce regeneration are fine.

- **Committed nonce map.** `landing/output/callback-map.json` (and `test-site/output/callback-map.json`) are in the public repo, and the Dockerfile serves `landing/` to honeyprompt.sh. Every live nonce is public, so callbacks can be forged. Move the map out of the served tree (e.g. `.honeyprompt/`), stop committing it, regenerate all nonces, redeploy. Treat existing honeyprompt.sh data as unverified.
- **Served map.** `build_router` uses `fallback_service(ServeDir)` over `output/`, so `/callback-map.json` is fetchable. Fixed by the move above; add a test that it 404s.
- **Header allowlist.** `fingerprint::extract` stores every header, including `Authorization`, `Cookie` and `Proxy-Authorization`. That contradicts the README safety model. Keep only an allowlist (User-Agent, Accept, X-Forwarded-For, ...).
- **Salted session_id.** `compute_session_id` is unsalted `SHA-256(ip + ua)`. Add a per-deployment salt generated on first run and stored in `.honeyprompt/`.
- **Request middleware.** No timeout, body limit or rate limit on the router. Add `TimeoutLayer` and `DefaultBodyLimit`.

## Ideas: context canaries (from the Dragos/HD Moore post, 2026-11-19)

Source: https://www.dragos.com/blog/ai-supply-chain-security-canary-tokens. They planted unique fake credentials/URLs in prompts sent to AI routers and providers; hits days or weeks later proved prompts were retained and read.

- **Record every hit.** The store upserts on nonce, so replays keep only the first IP/UA. For leak detection the later hits *are* the evidence. Add a per-hit table and "time since seeding" in reports. Prerequisite for the rest.
- **Exfiltration test tier.** Plant a HoneyPrompt-minted fake token in the agent's private context; the honeypot page asks the agent to include it in a callback. Proves private data + untrusted content + exfil channel end to end, with synthetic data only. Fits `test-agent` as a CI gate.
- **`honeyprompt seed`.** Mint labeled canary URLs for system prompts, CLAUDE.md, MCP tool descriptions, RAG docs, `.env.example`. Report which placement/route leaked.
- **Obeyed-now vs harvested-later.** Classify callbacks as in-session compliance vs delayed out-of-band fetches (different network, days later).
- **Webhook alerts.** Delayed hits need outbound notification, not someone watching the TUI.
- Skip: real AWS keys (use Thinkst Canarytokens), fake SSH/git/DB services. Reconsider DNS canaries later.

## Parked: validation experiment

Run named browsing agents (e.g. BrowserUse on a droplet) through honeyprompt.sh, taking the top N from an existing leaderboard so selection isn't self-curated, and publish results — ideally as an add-on to a benchmark author's report. Purpose: test whether researchers/red-teamers actually care before building anything bigger. Do the security fixes first; a leaked nonce map would invalidate it.

Bigger expansion ideas (federation, hosted benchmark site, leaderboard) only make sense if that experiment produces outside interest. Framing that held up: behavioral measurement rather than vuln disclosure; a fingerprint registry rather than naming vendors.

## Small stuff

- **Disclosure email.** security.txt uses the GitHub Security Advisories URL as contact. Set up security@honeyprompt.dev (forwarding service) and update it.

## Shipped

- 5.0.0 — Tier 4 (capability introspection, `/cb/v4/{nonce}/{b64_list}`) and Tier 5 (multi-step compliance with server-verified proof, `/cb/v5/{nonce}/{proof}`).
- Earlier: generator + payload catalog (T1–T3), server + callbacks, TUI monitor, Markdown report, `test-agent` with CI exit codes, release binaries, honeyprompt.sh live demo, honeyprompt.dev landing page with `/stats`, security.txt, setup wizard, zero-config `serve --domain`, deploy templates.
