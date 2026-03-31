# Phase 7: Live Demo Deployment - Context

**Gathered:** 2026-03-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Create deployment configuration for running `honeyprompt serve` as a persistent process on a DigitalOcean Droplet. Includes Dockerfile, systemd unit, Caddy TLS reverse proxy config, and deployment documentation. The actual deployment (provisioning the droplet, DNS, etc.) is a manual human step — this phase produces the config files and docs needed to deploy.

</domain>

<decisions>
## Implementation Decisions

### Hosting Platform
- **D-01:** DigitalOcean Droplet ($4-6/month). User already uses DO for deployment.
- **D-02:** systemd unit for process management (auto-restart on failure).
- **D-03:** Caddy for TLS reverse proxy — auto-provisions Let's Encrypt certificates for honeyprompt.sh domain.

### Persistence
- **D-04:** Persistent SQLite DB at `/var/lib/honeyprompt/events.db` on the droplet disk. Survives restarts and binary upgrades. This is the evidence store — losing it means losing the data this whole milestone is about.

### Binary Deployment
- **D-05:** Deploy the musl static binary from GitHub Releases (Phase 6 output). No Docker needed on the droplet — just download the binary, systemctl restart.

### Infrastructure Files (in-repo)
- **D-06:** Dockerfile still useful for local testing and potential future container deployment, even though the droplet runs the binary directly.
- **D-07:** Deployment docs describe the manual steps: provision droplet, install Caddy, copy systemd unit, download binary, configure DNS.

### Claude's Discretion
- Whether to include a deploy.sh script or just document the steps
- Caddy config format (Caddyfile vs JSON)
- Uptime monitoring provider (UptimeRobot, Healthchecks.io, or just systemd watchdog)
- Log rotation approach (systemd journal defaults vs explicit logrotate config)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Landing Page
- `landing/honeyprompt.toml` — Config with `callback_base_url = "https://honeyprompt.sh"`, bind `0.0.0.0:8080`
- `landing/output/` — Pre-generated honeypot files (index.html, callback-map.json, robots.txt, ai.txt)

### Server Code
- `src/server/mod.rs` — `serve()` function that the systemd unit will run
- `src/main.rs` — `Commands::Serve` dispatch

### Design Doc
- `~/.gstack/projects/johnzilla-honeyprompt/john-main-design-20260329-180748.md` — Deployment Checklist section

### Research
- `.planning/research/STACK.md` — Deployment section (Fly.io/Docker patterns, transferable to DO)
- `.planning/research/PITFALLS.md` — Docker image size, distroless base, static binary verification

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `landing/` directory — Complete honeypot ready to serve. Just point `honeyprompt serve landing/` at it.
- `.github/workflows/release.yml` — Produces the musl binary that gets deployed to the droplet
- `Cargo.toml` — Already has all deps; musl static linking works

### Established Patterns
- `honeyprompt serve` takes a project directory argument and serves from `output/`
- Binds to `0.0.0.0:8080` by default (configurable in honeyprompt.toml)
- Graceful shutdown on SIGTERM (systemd sends this on `systemctl stop`)

### Integration Points
- New files: `deploy/Caddyfile`, `deploy/honeyprompt.service`, `deploy/README-deploy.md`
- Optionally: `Dockerfile` for container testing

</code_context>

<specifics>
## Specific Ideas

- The landing page at `landing/output/` is already generated with callbacks pointing to `https://honeyprompt.sh` — no regeneration needed for deployment
- systemd unit should use `Restart=always` with `RestartSec=5` for auto-recovery
- Caddy reverse proxy: `honeyprompt.sh { reverse_proxy localhost:8080 }` — Caddy handles TLS automatically

</specifics>

<deferred>
## Deferred Ideas

- Fly.io deployment — works but user prefers DigitalOcean
- Docker Compose with Caddy sidecar — more portable but unnecessary for a single-binary deploy
- Automated deploy-on-tag via GitHub Actions SSH — nice but not needed for v2

</deferred>

---

*Phase: 07-live-demo-deployment*
*Context gathered: 2026-03-31*
