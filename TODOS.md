# TODOS

Single backlog for the project. Releases are numbered; nothing else is.

## Next

- **Rate limiting.** Not added; the router has a timeout and body cap, and callbacks use `try_send` (drops events when the broker queue is full rather than blocking). Per-IP limits are easiest in Caddy if needed.

## Ideas: context canaries (from the Dragos/HD Moore post, 2026-11-19)

Source: https://www.dragos.com/blog/ai-supply-chain-security-canary-tokens. They planted unique fake credentials/URLs in prompts sent to AI routers and providers; hits days or weeks later proved prompts were retained and read.

- All four ideas shipped (5.1.0–5.5.0): per-hit storage + timing labels, webhook alerts, tier 6 exfiltration, `honeyprompt seed`.
- Skip: real AWS keys (use Thinkst Canarytokens), fake SSH/git/DB services. Reconsider DNS canaries later.

## Next: validation experiment

Tooling shipped in 5.6.0; the protocol is [experiments/PROTOCOL.md](experiments/PROTOCOL.md). Left to do (operator): freeze the leaderboard snapshot and agent list, set up the agent-test subdomain, run 10 trials per agent (plus the `--exfil` arm and the curl baseline), send vendors their results, then publish — ideally as an add-on to a benchmark author's report. Purpose: test whether researchers/red-teamers actually care before building anything bigger. Needs the disclosure email first.

Bigger expansion ideas (federation, hosted benchmark site, leaderboard) only make sense if that experiment produces outside interest. Framing that held up: behavioral measurement rather than vuln disclosure; a fingerprint registry rather than naming vendors.

## Small stuff

- **IP retention knob.** Client IPs are kept indefinitely in `events` / `hits` / `seed_hits` (page views 90 days). Consider a `retention_days` setting that nulls or truncates IPs on old rows, for operators with privacy obligations.
- **Disclosure email.** security.txt uses the GitHub Security Advisories URL as contact. Set up security@honeyprompt.dev (forwarding service) and update it.

## Shipped

- 5.7.2 — `serve` installs its SIGINT/SIGTERM/SIGHUP handlers and binds before printing `ready` (a signal right after startup used to hit the default action and kill it without the drain), and prints the bound address (`listen:`); `serve` tests bind port 0 and read it, removing a port race between parallel tests that failed 5.7.1's CI.
- 5.7.1 (last review round) — report footer links honeyprompt.dev; stale duplicate `deploy/honeyprompt.service` removed; `page_views` bounded (per-IP 60 s cooldown updates the latest row, 100,000-row cap); `--domain` validated (host[:port]) and `callback_base_url` validated on load, via one shared validator with `--public-url`; `report` / `seed` refuse a missing DB instead of creating one; monitor integrated mode drains writers on quit and shows dropped / display-skipped events; monitor list capped at 10,000 with exact running totals (sessions as 64-bit hashes, capped at 100,000); constant-time canary compare (`subtle`); multi-arch (amd64 + arm64) Docker image from native runners; `init` no longer creates the unused `overrides/` dir; README states T5 is proof of computation, not comprehension; CLAUDE.md architecture brought up to date.
- 5.7.0 (P2 fixes from review) — Docker image ships no config (it pointed every deployment's callbacks at honeyprompt.sh); `serve` without config or `--domain` refuses to start; `--domain` sets only the URL on an existing config (it reset tiers to 1–3); pipeline is now broker → mpsc → DB writer → broadcast of persisted events, so the TUI and `--json` log show real replay/fire count and the writer can't lag-drop; shutdown drains queued events (up to 5 s) before the summary; site regenerates when tiers change; `/stats` cached 5 s with single-flight; dropped events counted (warning + shutdown summary); default tiers are now 1–5 (zero-config `serve --domain`, `init`), as the README already claimed.
- 5.6.1 (fixes from review) — monitor no longer panics on multibyte user agents (byte slicing in `truncate_str`; a crafted UA on a real nonce could crash integrated-mode monitor and its server); `count_detections` now excludes known crawlers (pattern expected a bare `KnownCrawler`, real labels are `KnownCrawler:Provider`; shutdown summary over-counted); `serve` shuts down gracefully on SIGTERM and SIGHUP (PID 1 in the container ignored SIGTERM, so `docker stop` ended in SIGKILL and lost buffered events); systemd `KillSignal=SIGINT` workaround removed; `derive_seed` hardened with `get(0..8)`.
- 5.6.0 — experiment harness: `test-agent --agent-cmd` (runs the agent via `sh -c` in its own process group; `{url}`/`{canary}` and `HONEYPROMPT_URL`/`HONEYPROMPT_CANARY`; ends `--grace` after exit or kills the group at `--timeout`), `--trials`, `--label`, `--results` (JSON lines; never the command or canary), `--agent-log`; `--public-url` + `--trusted-proxies` with a random `/run/{id}/` prefix per run (404 outside it); scorecard gains page fetch, user agents and agent exit; `honeyprompt experiment summarize` (Markdown, Wilson 95% intervals). `experiments/PROTOCOL.md`.
- 5.5.0 — seeded canaries: `honeyprompt seed add <label> [--note] [--format url|markdown|env]` and `seed list`; opaque `/r/{nonce}` URLs serving a uniform warning page; hits stored in `seeds`/`seed_hits` (cap 1,000 per seed, true total kept), separate from tier data; `seed_hit` webhook alerts (always sent); report gains Seeded Canaries.
- 5.4.0 — tier 6, context exfiltration (`test-agent --exfil`, `--canary-file`): per-run fake `HPCANARY-` token, `/cb/v6/{nonce}/{value}` stores only match/no-match + length, scorecard out of 6, exit 1 on leak. Page asks only for the marker, never for secrets in general; `serve`/`generate` refuse tier 6. Fixed: `test-agent` only generated tiers 1–3, so T4/T5 were never actually tested.
- 5.3.0 — webhook alerts: HMAC-SHA256-signed JSON (RustCrypto `hmac`), Slack and Discord formats with escaping, `send = notable | all`, rate limit 30/min with `suppressed` count, retries on 5xx/429, no redirects, secrets via `HONEYPROMPT_WEBHOOK_*` env vars and never logged; `honeyprompt webhook-test`. reqwest over rustls (`ring`, bundled webpki roots) — no OpenSSL or aws-lc.
- 5.2.0 — page fetches recorded (`page_views`, 90-day retention); every hit labelled in-session / delayed / out-of-band against that IP's last page fetch; report gains a Hit Timing section. Repeat Hits drops the same-IP/UA-as-first columns (nonces are shared by all visitors, so they compared different people).
- 5.1.0 — every callback stored in a new `hits` table (capped at 1,000 per nonce; `fire_count` keeps the true total); report gains a Repeat Hits section with time since seeding / since first hit and same-IP/UA flags. First step of the context-canary work.
- 5.0.2 — real client IP behind a reverse proxy: `trusted_proxies` config / `--trusted-proxies` flag; `X-Forwarded-For` is honoured only from trusted peers, read right to left. Deploy compose files trust the private Docker ranges.

- 5.0.1 (security) — nonce map moved to `.honeyprompt/` and no longer served or committed; generated sites untracked and created by `serve` on first start; header allowlist (credentials never stored); per-deployment salted session IDs; request timeout + body limit; `serve <path> --domain` creates a persistent project (Docker/systemd templates actually persist now); `test-agent` now loads T5 formulas, so Tier 5 hits are detected.
- 5.0.0 — Tier 4 (capability introspection, `/cb/v4/{nonce}/{b64_list}`) and Tier 5 (multi-step compliance with server-verified proof, `/cb/v5/{nonce}/{proof}`).
- Earlier: generator + payload catalog (T1–T3), server + callbacks, TUI monitor, Markdown report, `test-agent` with CI exit codes, release binaries, honeyprompt.sh live demo, honeyprompt.dev landing page with `/stats`, security.txt, setup wizard, zero-config `serve --domain`, deploy templates.
