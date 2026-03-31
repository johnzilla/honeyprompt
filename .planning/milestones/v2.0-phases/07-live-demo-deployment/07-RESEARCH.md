# Phase 7: Live Demo Deployment — Research

**Researched:** 2026-03-29
**Domain:** Systemd service + Caddy TLS reverse proxy + Docker (local testing) + uptime monitoring
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** DigitalOcean Droplet ($4–6/month). User already uses DO for deployment.
- **D-02:** systemd unit for process management (auto-restart on failure).
- **D-03:** Caddy for TLS reverse proxy — auto-provisions Let's Encrypt certificates for honeyprompt.sh domain.
- **D-04:** Persistent SQLite DB at `/var/lib/honeyprompt/events.db` on the droplet disk. Survives restarts and binary upgrades. This is the evidence store — losing it means losing the data this whole milestone is about.
- **D-05:** Deploy the musl static binary from GitHub Releases (Phase 6 output). No Docker needed on the droplet — just download the binary, `systemctl restart`.
- **D-06:** Dockerfile still useful for local testing and potential future container deployment, even though the droplet runs the binary directly.
- **D-07:** Deployment docs describe the manual steps: provision droplet, install Caddy, copy systemd unit, download binary, configure DNS.

### Claude's Discretion

- Whether to include a `deploy.sh` script or just document the steps
- Caddy config format (Caddyfile vs JSON)
- Uptime monitoring provider (UptimeRobot, Healthchecks.io, or just systemd watchdog)
- Log rotation approach (systemd journal defaults vs explicit logrotate config)

### Deferred Ideas (OUT OF SCOPE)

- Fly.io deployment — works but user prefers DigitalOcean
- Docker Compose with Caddy sidecar — more portable but unnecessary for a single-binary deploy
- Automated deploy-on-tag via GitHub Actions SSH — nice but not needed for v2

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DEPLOY-01 | Repository includes deployment configuration (Dockerfile or systemd unit) for running `honeyprompt serve` as a persistent process | systemd unit + Caddy Caddyfile + Dockerfile in `deploy/` directory |
| DEPLOY-02 | honeyprompt.sh domain serves a live honeypot with canary payloads over HTTPS | Caddy auto-provisions Let's Encrypt; `landing/output/` already generated with correct callback URLs |
| DEPLOY-03 | Live demo has uptime monitoring and process auto-restart | `Restart=always` in systemd unit; UptimeRobot free tier for external HTTP check |

</phase_requirements>

---

## Summary

Phase 7 produces three config files and a deployment runbook. The Rust binary and the honeypot site files (`landing/output/`) are already built in prior phases — this phase is pure infrastructure configuration. No new Rust code is required.

The deployment model is: static musl binary from GitHub Releases runs as a systemd service on a DigitalOcean Droplet. Caddy sits in front on ports 80/443, handles Let's Encrypt TLS automatically, and proxies to the binary on `localhost:8080`. A Docker image is produced for local testing only.

The critical constraint is the SQLite database at `/var/lib/honeyprompt/events.db` — this is the evidence store for the entire project's value proposition. File layout, service permissions, and upgrade procedures must protect this path.

**Primary recommendation:** Write three files (`deploy/honeyprompt.service`, `deploy/Caddyfile`, `Dockerfile`) plus `deploy/README-deploy.md`. Skip the `deploy.sh` script — the manual steps are straightforward enough for a single deployment target and explicit documentation is safer than a script that mutates production state. Use UptimeRobot (free tier) for external uptime monitoring.

---

## Standard Stack

### Core

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| systemd | v252+ (Ubuntu 22.04+) | Process lifecycle — start, restart, stop | D-02 locked. Installed by default on all Debian/Ubuntu-based DigitalOcean droplets. |
| Caddy | v2.9+ | TLS reverse proxy, automatic Let's Encrypt | D-03 locked. Single binary, zero cert management, Caddyfile is 3 lines. APT repo available. |
| musl static binary | (output of Phase 6 release) | The actual `honeyprompt serve` process | D-05 locked. `x86_64-unknown-linux-musl` target; fully static, no libc dependency on the droplet. |
| Docker | 29.x (local dev) | Local integration testing of the container path | D-06 locked. Only for local testing — not used on the droplet. |

### Supporting

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| UptimeRobot | free tier | External HTTP(S) uptime monitor, 5-min polling, email/Slack alerts | Recommended for DEPLOY-03 — zero cost, no infrastructure required, adequate for a live demo |
| systemd journal | built-in | Log collection for honeyprompt service output | Default; no logrotate config needed. `journalctl -u honeyprompt -f` for streaming. Journal rotates by size automatically. |
| `curl` | system | Binary download in deploy steps | Pre-installed on Ubuntu; used in runbook to fetch release binary from GitHub Releases. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| UptimeRobot | Healthchecks.io (for cron/heartbeat) or systemd watchdog | UptimeRobot is better fit: it checks the HTTP endpoint externally, not just process liveness. Healthchecks.io is heartbeat-oriented (cron-style push) — honeyprompt is a long-running server, not a cron job. systemd watchdog alone doesn't alert on network-level failures. |
| Caddyfile format | Caddy JSON API | Caddyfile is simpler, human-readable, idempotent. JSON API is for dynamic config management — not needed here. |
| `deploy/README-deploy.md` | `deploy.sh` script | Script automates but silently corrupts if run twice or against a partially-provisioned server. README is auditable and safe. |

**Installation (on the droplet):**
```bash
# Install Caddy via APT
sudo apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt-get update && sudo apt-get install caddy

# Create honeyprompt service user
sudo useradd --system --no-create-home --shell /usr/sbin/nologin honeyprompt

# Create data directory
sudo mkdir -p /var/lib/honeyprompt
sudo chown honeyprompt:honeyprompt /var/lib/honeyprompt
```

---

## Architecture Patterns

### Recommended Directory Layout (on droplet)

```
/usr/local/bin/honeyprompt          # static binary
/etc/honeyprompt/                   # config + site files
  honeyprompt.toml                  # bind_address, callback_base_url
  landing/                          # rsync'd from repo
    output/                         # pre-generated honeypot files
/var/lib/honeyprompt/
  events.db                         # SQLite evidence store (D-04)
/etc/systemd/system/
  honeyprompt.service               # from deploy/honeyprompt.service
/etc/caddy/
  Caddyfile                         # from deploy/Caddyfile
```

### Recommended Repository Layout

```
deploy/
  honeyprompt.service       # systemd unit file
  Caddyfile                 # Caddy reverse proxy config
  README-deploy.md          # Manual deployment runbook
Dockerfile                  # Multi-stage build for local testing (repo root)
```

### Pattern 1: systemd Unit for Long-Running Rust Binary

```ini
# deploy/honeyprompt.service
[Unit]
Description=HoneyPrompt honeypot server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=honeyprompt
Group=honeyprompt
WorkingDirectory=/etc/honeyprompt
ExecStart=/usr/local/bin/honeyprompt serve /etc/honeyprompt/landing
Restart=always
RestartSec=5
StartLimitIntervalSec=600
StartLimitBurst=5

# Hardening
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ReadWritePaths=/var/lib/honeyprompt

[Install]
WantedBy=multi-user.target
```

Key points:
- `After=network-online.target` — do not start until DNS is available (Caddy and honeyprompt both need the network)
- `Restart=always` — restart on any exit (crash, OOM, explicit stop except `systemctl stop`)
- `RestartSec=5` + `StartLimitBurst=5` — prevents tight restart loops if the binary crashes on startup
- `ProtectSystem=strict` + `ReadWritePaths=/var/lib/honeyprompt` — system is read-only except the DB path
- `WorkingDirectory=/etc/honeyprompt` — `honeyprompt serve` resolves the landing dir relative to cwd

**Note on serve argument:** The `serve` command takes a project directory path. That directory must contain `honeyprompt.toml` and `output/`. On the droplet, the path is `/etc/honeyprompt/landing` (i.e., `landing/honeyprompt.toml` + `landing/output/`). The existing `landing/` directory in the repo is the source of truth.

### Pattern 2: Caddy Reverse Proxy with Automatic TLS

```
# deploy/Caddyfile
honeyprompt.sh {
    reverse_proxy localhost:8080
}
```

That is the complete Caddyfile. Caddy auto-provisions a Let's Encrypt certificate for `honeyprompt.sh` on first startup, provided:
1. DNS A record for `honeyprompt.sh` points to the droplet's public IP
2. Ports 80 and 443 are open (DigitalOcean Firewall or `ufw`)
3. Caddy is running as a systemd service with the `caddy` user (which has permission to bind port 80/443 via `AmbientCapabilities=CAP_NET_BIND_SERVICE` in the official unit file)

Reload Caddy after config changes: `sudo systemctl reload caddy`

### Pattern 3: Multi-Stage Dockerfile for Local Testing

```dockerfile
# Source: .planning/research/PITFALLS.md (Pitfall 7)
FROM rust:1.87-slim AS builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    strip target/x86_64-unknown-linux-musl/release/honeyprompt

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/honeyprompt /usr/local/bin/honeyprompt
ENTRYPOINT ["/usr/local/bin/honeyprompt"]
```

Target image size: under 20 MB. Verify with `docker images honeyprompt`.

### Pattern 4: Binary Upgrade Procedure (in runbook)

```bash
# Download new binary from GitHub Releases
curl -L https://github.com/user/honeyprompt/releases/latest/download/honeyprompt-x86_64-unknown-linux-musl.tar.gz \
  | tar -xz -C /tmp/

# Replace binary (service auto-restarts)
sudo systemctl stop honeyprompt
sudo mv /tmp/honeyprompt /usr/local/bin/honeyprompt
sudo chmod +x /usr/local/bin/honeyprompt
sudo systemctl start honeyprompt
```

SQLite DB at `/var/lib/honeyprompt/events.db` is untouched by this procedure.

### Anti-Patterns to Avoid

- **Running honeyprompt as root:** Use a dedicated `honeyprompt` system user. The binary handles untrusted HTTP traffic — least privilege is essential.
- **Storing events.db inside /etc/honeyprompt:** `/etc/` is config, not data. Events DB goes in `/var/lib/honeyprompt/` per FHS conventions and is excluded from config changes.
- **Bind-mounting SQLite into Docker via overlay2:** Known WAL-mode issue (documented in PITFALLS.md). Use named Docker volumes if ever running in Docker.
- **Opening port 8080 to the public firewall:** Only port 80 and 443 should be public. Port 8080 is localhost-only; Caddy proxies to it.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TLS certificate provisioning | Custom ACME client, cert cron job | Caddy automatic TLS | Caddy handles ACME, renewal, and redirect from HTTP to HTTPS automatically. Rolling this by hand is days of ops work and cert expiry is a common failure mode. |
| Process supervisor / restart loop | Shell `while true; do ./honeyprompt; done` | systemd `Restart=always` | systemd tracks PID, handles stdout/stderr capture, integrates with journal, starts on boot, supports `StartLimitBurst` to prevent restart storms. |
| External uptime check | Polling script on the same server | UptimeRobot (external) | A check that runs on the same server cannot detect network-level failures, ISP outages, or Caddy/systemd failures that leave the machine up but the service unreachable. Must be external. |

---

## Common Pitfalls

### Pitfall 1: Caddy Fails to Obtain Certificate — DNS Not Propagated

**What goes wrong:** Caddy starts, attempts ACME HTTP-01 challenge for `honeyprompt.sh`, but DNS has not propagated to point at the droplet. Challenge fails silently, Caddy falls back to HTTP or logs the error. The operator sees `curl https://honeyprompt.sh` fail or return a self-signed cert.

**Why it happens:** Droplet is provisioned, DNS is updated, and Caddy is started all within minutes. DNS TTL and propagation take 1–15 minutes even with a low TTL.

**How to avoid:** After updating DNS, verify propagation before starting Caddy: `dig +short honeyprompt.sh @1.1.1.1` must return the droplet IP. Then start Caddy. Check Caddy logs on first start: `journalctl -u caddy -f`.

**Warning signs:** `journalctl -u caddy` shows "failed to get certificate" or "ACME challenge failed". `curl -I https://honeyprompt.sh` returns a self-signed cert or SSL error.

---

### Pitfall 2: honeyprompt Serve Cannot Find landing/ or events.db

**What goes wrong:** The systemd unit starts but honeyprompt immediately exits because it can't find `honeyprompt.toml`, `output/callback-map.json`, or can't write to `events.db`.

**Why it happens:** Three common causes:
1. `ExecStart` path is wrong — the `serve` argument does not point at the directory containing `honeyprompt.toml`
2. `ReadWritePaths` does not include `/var/lib/honeyprompt`, so `ProtectSystem=strict` blocks the DB write
3. The `honeyprompt` service user has no write permission on `/var/lib/honeyprompt`

**How to avoid:** Follow the directory layout exactly. After copying files, verify:
```bash
sudo -u honeyprompt ls /etc/honeyprompt/landing/honeyprompt.toml
sudo -u honeyprompt ls /etc/honeyprompt/landing/output/callback-map.json
sudo -u honeyprompt touch /var/lib/honeyprompt/test && sudo -u honeyprompt rm /var/lib/honeyprompt/test
```

**Warning signs:** `systemctl status honeyprompt` shows `Active: activating (auto-restart)` cycling. `journalctl -u honeyprompt -n 50` shows "Failed to read callback-map.json" or "permission denied" on the DB path.

---

### Pitfall 3: Port 8080 Exposed Publicly via DigitalOcean Firewall

**What goes wrong:** The firewall allows inbound traffic on all ports. Agents reach `http://honeyprompt.sh:8080` directly, bypassing Caddy, receiving HTTP without the expected HTTPS URL scheme. Callbacks fired point to `https://honeyprompt.sh/cb/...` (correct), but the server is reachable unencrypted.

**Why it happens:** DigitalOcean Droplets default to allowing all inbound traffic unless a Cloud Firewall is configured. New operators assume "the app is internal" without verifying firewall rules.

**How to avoid:** Configure DigitalOcean Cloud Firewall (or `ufw`) to allow only ports 22 (SSH), 80 (HTTP/ACME), and 443 (HTTPS). Port 8080 must be blocked from public access.

---

### Pitfall 4: SQLite events.db Lost During Binary Upgrade

**What goes wrong:** The runbook uses `sudo rm /usr/local/bin/honeyprompt && sudo cp new_binary /usr/local/bin/honeyprompt` but the operator mistakenly targets the wrong path and deletes `/var/lib/honeyprompt/` instead.

**Why it happens:** Copy-paste error in a manual procedure. The evidence DB is irreplaceable.

**How to avoid:** The runbook must make the upgrade path explicit:
- Binary lives at `/usr/local/bin/honeyprompt` — this is the only file replaced during upgrades
- DB lives at `/var/lib/honeyprompt/events.db` — this is never touched during upgrades
- Suggest a pre-upgrade backup: `cp /var/lib/honeyprompt/events.db ~/events.db.$(date +%Y%m%d)`

---

### Pitfall 5: Docker FROM scratch Runtime Crash

**What goes wrong:** `docker run honeyprompt serve /data` exits immediately with code 127 or shows `libdl.so.2: cannot open shared object file`.

**Why it happens:** Even with `--target x86_64-unknown-linux-musl`, Tokio (via mio) and rusqlite (bundled) may still have a dynamic dependency. `FROM scratch` has no dynamic linker at all.

**How to avoid:** Use `FROM gcr.io/distroless/static-debian12` (not `scratch`). Verify before publishing: `ldd target/x86_64-unknown-linux-musl/release/honeyprompt` must output "statically linked" or "not a dynamic executable". If not, add `RUSTFLAGS="-C target-feature=+crt-static"` to the build command.

See PITFALLS.md Pitfall 6 for full details.

---

## Code Examples

### Complete systemd Unit

```ini
# deploy/honeyprompt.service
# Source: systemd docs + Caddy docs running guide + existing serve() analysis
[Unit]
Description=HoneyPrompt honeypot server
Documentation=https://github.com/user/honeyprompt
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=honeyprompt
Group=honeyprompt
WorkingDirectory=/etc/honeyprompt
ExecStart=/usr/local/bin/honeyprompt serve /etc/honeyprompt/landing
Restart=always
RestartSec=5
StartLimitIntervalSec=600
StartLimitBurst=5

# Sandbox hardening
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ReadWritePaths=/var/lib/honeyprompt

[Install]
WantedBy=multi-user.target
```

### Complete Caddyfile

```
# deploy/Caddyfile
# Source: https://caddyserver.com/docs/quick-starts/reverse-proxy
honeyprompt.sh {
    reverse_proxy localhost:8080
}
```

### honeyprompt.toml for Production Deployment

```toml
# /etc/honeyprompt/landing/honeyprompt.toml
callback_base_url = "https://honeyprompt.sh"
bind_address = "0.0.0.0:8080"
tiers = [1, 2, 3]
page_title = "HoneyPrompt — AI Agent Canary Detection"
theme = "default"
```

Note: `landing/honeyprompt.toml` already exists in the repo with these exact values — it is copied to the droplet as-is.

### Dockerfile (local testing only)

```dockerfile
# Dockerfile
# Source: .planning/research/PITFALLS.md Pitfall 6 & 7 prevention
FROM rust:1.87-slim AS builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target x86_64-unknown-linux-musl && \
    strip target/x86_64-unknown-linux-musl/release/honeyprompt

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/honeyprompt /usr/local/bin/honeyprompt
ENTRYPOINT ["/usr/local/bin/honeyprompt"]
```

Local test command:
```bash
docker build -t honeyprompt .
docker run --rm honeyprompt --version
docker run --rm -v $(pwd)/landing:/data -p 8080:8080 honeyprompt serve /data
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Nginx + certbot cron | Caddy with automatic HTTPS | ~2020 onward | Caddy eliminates cert renewal cron, nginx config boilerplate, and the 2-command cert setup. For single-app reverse proxy this is strictly better. |
| `FROM ubuntu` Docker base | `FROM gcr.io/distroless/static-debian12` | 2019+ (distroless) | 20x smaller images, no shell attack surface, CA certs included. The right default for static Rust binaries. |
| `actions-rs/*` GitHub Actions | `dtolnay/rust-toolchain` + direct `cargo` | 2023 (actions-rs archived) | actions-rs is dead — see PITFALLS.md Pitfall 5. This phase doesn't add new CI but the Dockerfile is tested locally, not in CI. |

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Docker | Dockerfile local testing (D-06) | Yes | 29.3.1 | — |
| systemd | systemd unit file authoring | Yes (local) | v252 (Fedora 43) | Unit file is authored locally, deployed to droplet |
| Caddy | TLS reverse proxy (D-03) | Not on dev machine | — | N/A — installed on DigitalOcean droplet as part of manual deploy |
| DigitalOcean Droplet | Live deployment (D-01) | Not yet provisioned | — | Manual step in deploy runbook |
| `honeyprompt` binary (musl) | Droplet deployment (D-05) | Yes (via Phase 6 releases) | Phase 6 output | `cargo build --release --target x86_64-unknown-linux-musl` locally |

**Missing dependencies with no fallback:**
- DigitalOcean Droplet — must be provisioned manually. This is explicitly a human step (D-07).

**Missing dependencies with fallback:**
- Caddy — not on dev machine, but all Caddy artifacts are config files authored locally and deployed to the droplet.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — standard `cargo test` |
| Quick run command | `cargo test` |
| Full suite command | `cargo test` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DEPLOY-01 | `deploy/honeyprompt.service` exists and contains required directives | manual smoke | verify file exists + grep key fields | No config test — file inspection |
| DEPLOY-01 | `deploy/Caddyfile` exists with correct reverse_proxy directive | manual smoke | `grep -q "reverse_proxy localhost:8080" deploy/Caddyfile` | No config test |
| DEPLOY-01 | `Dockerfile` builds successfully | manual | `docker build -t honeyprompt . && docker run --rm honeyprompt --version` | Dockerfile not yet created (Wave 0) |
| DEPLOY-02 | honeyprompt.sh serves HTTPS with honeypot content | manual e2e | `curl -s https://honeyprompt.sh | grep -q "SECURITY RESEARCH CANARY"` | Depends on live deployment |
| DEPLOY-03 | systemd unit uses `Restart=always` | manual smoke | `grep -q "Restart=always" deploy/honeyprompt.service` | File not yet created (Wave 0) |
| DEPLOY-03 | UptimeRobot monitor configured | manual | Human verification — login to UptimeRobot, confirm monitor is up | External service |

### Sampling Rate

- **Per task commit:** `cargo test` (ensures no regressions in server logic)
- **Per wave merge:** `cargo test` + `docker build -t honeyprompt . && docker run --rm honeyprompt --version`
- **Phase gate:** All config files present + Docker image builds + `cargo test` green

### Wave 0 Gaps

- [ ] `Dockerfile` — covers DEPLOY-01 (container path)
- [ ] `deploy/honeyprompt.service` — covers DEPLOY-01 + DEPLOY-03
- [ ] `deploy/Caddyfile` — covers DEPLOY-01 + DEPLOY-02
- [ ] `deploy/README-deploy.md` — covers D-07 (manual deploy steps)

No new Rust test files needed — existing `cargo test` suite covers server logic. These are config/infra files verified by inspection and a Docker smoke build.

---

## Open Questions

1. **Where does `honeyprompt serve` expect to find `events.db`?**
   - What we know: `src/server/mod.rs` line 99 constructs `db_path = project_path.join(".honeyprompt").join("events.db")`. So if the serve argument is `/etc/honeyprompt/landing`, the DB would be at `/etc/honeyprompt/landing/.honeyprompt/events.db`.
   - What's unclear: D-04 says "persistent SQLite DB at `/var/lib/honeyprompt/events.db`" — but the current code derives the DB path from the project directory, not from a config option. Either the systemd unit must use `ReadWritePaths=/etc/honeyprompt/landing/.honeyprompt` instead of `/var/lib/honeyprompt`, or the `honeyprompt.toml` needs a configurable `db_path` field, or the deploy layout must place the landing dir at a path where `.honeyprompt/` resolves to `var/lib/honeyprompt`.
   - Recommendation: The simplest approach is to adjust the deploy layout so the landing directory is `/var/lib/honeyprompt/landing`. Then `db_path` resolves to `/var/lib/honeyprompt/landing/.honeyprompt/events.db`, which is within `ReadWritePaths=/var/lib/honeyprompt`. D-04's spirit is preserved — the DB is in a persistent, non-ephemeral location separate from the binary. The planner should resolve this and document the exact `ExecStart` argument path.

2. **Should `SIGTERM` from systemd trigger graceful shutdown or instant kill?**
   - What we know: `src/server/mod.rs` uses `shutdown_signal()` which listens on `Ctrl+C` (SIGINT), not SIGTERM. `systemctl stop` sends SIGTERM, not SIGINT. This means `systemctl stop honeyprompt` will use the default systemd `KillSignal=SIGTERM` but the server's graceful shutdown is listening for SIGINT.
   - What's unclear: The DB write pipeline uses `mpsc` channels — in-flight events may be lost if the process is killed before the channel drains.
   - Recommendation: Add `KillSignal=SIGINT` to the systemd unit, or patch `shutdown_signal()` in the server to also listen for SIGTERM. The planner should pick one and the implementer should verify journal output after `systemctl stop` shows "Shutdown complete."

---

## Sources

### Primary (HIGH confidence)

- Caddy official documentation — https://caddyserver.com/docs/quick-starts/reverse-proxy (Caddyfile syntax verified)
- Caddy official documentation — https://caddyserver.com/docs/running (systemd unit installation process)
- systemd documentation — `man systemd.service` patterns for `Restart=always`, `StartLimitBurst`, `ProtectSystem`
- `.planning/research/PITFALLS.md` — Pitfall 6 (distroless vs scratch), Pitfall 7 (Docker multi-stage), confirmed against existing research
- `.planning/research/STACK.md` — Deployment section (Fly.io/Docker patterns adapted to DO/binary)
- `src/server/mod.rs` — direct code inspection of `serve()`, `shutdown_signal()`, and DB path derivation
- `landing/honeyprompt.toml` — confirmed `callback_base_url = "https://honeyprompt.sh"` and `bind_address = "0.0.0.0:8080"`

### Secondary (MEDIUM confidence)

- UptimeRobot — https://uptimerobot.com/pricing/ — free tier confirmed: 50 monitors, 5-min polling, external HTTP checks
- DigitalOcean Caddy tutorial — https://www.digitalocean.com/community/tutorials/how-to-host-a-website-with-caddy-on-ubuntu-22-04 — APT install method and systemd unit confirmed

### Tertiary (LOW confidence)

- None — all critical claims are backed by primary sources.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — systemd, Caddy, musl binary are all verified tools with official docs
- Architecture: HIGH — config file contents derived from code inspection + official Caddy docs
- Pitfalls: HIGH — drawn from existing verified PITFALLS.md research and direct code review
- Open Question 1 (DB path): HIGH confidence the problem is real; MEDIUM on recommended resolution (need code inspection of config struct for potential `db_path` override)
- Open Question 2 (SIGTERM): HIGH confidence the gap is real (SIGINT vs SIGTERM mismatch confirmed in code)

**Research date:** 2026-03-29
**Valid until:** 2026-06-29 (Caddy and systemd patterns are stable; re-check if upgrading Caddy major version)
