# TODOS

Single backlog for the project. Releases are numbered; nothing else is.

## Next

- **Rate limiting.** Not added; the router has a timeout and body cap, and callbacks use `try_send` (drops events when the broker queue is full rather than blocking). Per-IP limits are easiest in Caddy if needed.

## Ideas: context canaries (from the Dragos/HD Moore post, 2026-11-19)

Source: https://www.dragos.com/blog/ai-supply-chain-security-canary-tokens. They planted unique fake credentials/URLs in prompts sent to AI routers and providers; hits days or weeks later proved prompts were retained and read.

- **`honeyprompt seed`.** Mint labeled canary URLs for system prompts, CLAUDE.md, MCP tool descriptions, RAG docs, `.env.example`. Report which placement/route leaked.
- Skip: real AWS keys (use Thinkst Canarytokens), fake SSH/git/DB services. Reconsider DNS canaries later.

## Parked: validation experiment

Run named browsing agents (e.g. BrowserUse on a droplet) through honeyprompt.sh, taking the top N from an existing leaderboard so selection isn't self-curated, and publish results — ideally as an add-on to a benchmark author's report. Purpose: test whether researchers/red-teamers actually care before building anything bigger. Do the security fixes first; a leaked nonce map would invalidate it.

Bigger expansion ideas (federation, hosted benchmark site, leaderboard) only make sense if that experiment produces outside interest. Framing that held up: behavioral measurement rather than vuln disclosure; a fingerprint registry rather than naming vendors.

## Small stuff

- **Disclosure email.** security.txt uses the GitHub Security Advisories URL as contact. Set up security@honeyprompt.dev (forwarding service) and update it.

## Shipped

- 5.4.0 — tier 6, context exfiltration (`test-agent --exfil`, `--canary-file`): per-run fake `HPCANARY-` token, `/cb/v6/{nonce}/{value}` stores only match/no-match + length, scorecard out of 6, exit 1 on leak. Page asks only for the marker, never for secrets in general; `serve`/`generate` refuse tier 6. Fixed: `test-agent` only generated tiers 1–3, so T4/T5 were never actually tested.
- 5.3.0 — webhook alerts: HMAC-SHA256-signed JSON (RustCrypto `hmac`), Slack and Discord formats with escaping, `send = notable | all`, rate limit 30/min with `suppressed` count, retries on 5xx/429, no redirects, secrets via `HONEYPROMPT_WEBHOOK_*` env vars and never logged; `honeyprompt webhook-test`. reqwest over rustls (`ring`, bundled webpki roots) — no OpenSSL or aws-lc.
- 5.2.0 — page fetches recorded (`page_views`, 90-day retention); every hit labelled in-session / delayed / out-of-band against that IP's last page fetch; report gains a Hit Timing section. Repeat Hits drops the same-IP/UA-as-first columns (nonces are shared by all visitors, so they compared different people).
- 5.1.0 — every callback stored in a new `hits` table (capped at 1,000 per nonce; `fire_count` keeps the true total); report gains a Repeat Hits section with time since seeding / since first hit and same-IP/UA flags. First step of the context-canary work.
- 5.0.2 — real client IP behind a reverse proxy: `trusted_proxies` config / `--trusted-proxies` flag; `X-Forwarded-For` is honoured only from trusted peers, read right to left. Deploy compose files trust the private Docker ranges.

- 5.0.1 (security) — nonce map moved to `.honeyprompt/` and no longer served or committed; generated sites untracked and created by `serve` on first start; header allowlist (credentials never stored); per-deployment salted session IDs; request timeout + body limit; `serve <path> --domain` creates a persistent project (Docker/systemd templates actually persist now); `test-agent` now loads T5 formulas, so Tier 5 hits are detected.
- 5.0.0 — Tier 4 (capability introspection, `/cb/v4/{nonce}/{b64_list}`) and Tier 5 (multi-step compliance with server-verified proof, `/cb/v5/{nonce}/{proof}`).
- Earlier: generator + payload catalog (T1–T3), server + callbacks, TUI monitor, Markdown report, `test-agent` with CI exit codes, release binaries, honeyprompt.sh live demo, honeyprompt.dev landing page with `/stats`, security.txt, setup wizard, zero-config `serve --domain`, deploy templates.
