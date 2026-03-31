# Deploying HoneyPrompt to DigitalOcean

HoneyPrompt runs as a static musl binary managed by systemd with Caddy handling TLS. A $4–6/month
DigitalOcean Droplet (1 GB RAM, 25 GB disk, Ubuntu 22.04+) is sufficient for the live demo at
honeyprompt.sh.

Architecture: Internet → Caddy (:443 TLS) → honeyprompt (:8080) → SQLite DB

---

## Prerequisites

Before starting, ensure you have:

- A DigitalOcean account with a provisioned Ubuntu 22.04+ Droplet (1 GB RAM, 25 GB disk)
- SSH access to the Droplet
- DNS A record for `honeyprompt.sh` pointing to the Droplet's public IP
- A GitHub Release binary available at `https://github.com/johnzilla/honeyprompt/releases/latest`

---

## Section 1: Initial Server Setup

Connect to the Droplet via SSH, then run:

```bash
# Create a dedicated system user (no login shell, no home dir)
sudo useradd --system --no-create-home --shell /usr/sbin/nologin honeyprompt

# Create data directory (owned by honeyprompt user)
sudo mkdir -p /var/lib/honeyprompt
sudo chown honeyprompt:honeyprompt /var/lib/honeyprompt

# Create landing directory inside data dir
sudo mkdir -p /var/lib/honeyprompt/landing/output
sudo chown -R honeyprompt:honeyprompt /var/lib/honeyprompt/landing
```

Copy the landing site files from the repository to the Droplet:

```bash
# From your local machine (adjust paths as needed)
scp landing/honeyprompt.toml root@<DROPLET_IP>:/tmp/honeyprompt.toml
scp -r landing/output/ root@<DROPLET_IP>:/tmp/landing-output/

# On the Droplet: move files into place
sudo mv /tmp/honeyprompt.toml /var/lib/honeyprompt/landing/honeyprompt.toml
sudo mv /tmp/landing-output/* /var/lib/honeyprompt/landing/output/
sudo chown -R honeyprompt:honeyprompt /var/lib/honeyprompt/landing
```

Verify the expected file structure:

```bash
# These must exist before starting the service
sudo -u honeyprompt ls /var/lib/honeyprompt/landing/honeyprompt.toml
sudo -u honeyprompt ls /var/lib/honeyprompt/landing/output/index.html
sudo -u honeyprompt ls /var/lib/honeyprompt/landing/output/callback-map.json

# Verify the honeyprompt user can write to the data directory (for events.db)
sudo -u honeyprompt touch /var/lib/honeyprompt/test && \
  sudo -u honeyprompt rm /var/lib/honeyprompt/test && \
  echo "Write permission OK"
```

**Note on DB path:** `honeyprompt serve` creates `events.db` at
`<landing-dir>/.honeyprompt/events.db`. With `ExecStart` pointing at
`/var/lib/honeyprompt/landing`, the evidence store lives at
`/var/lib/honeyprompt/landing/.honeyprompt/events.db` — within
`ReadWritePaths=/var/lib/honeyprompt`, which satisfies the systemd sandbox constraint.

---

## Section 2: Install Binary

```bash
# Download the latest musl static binary from GitHub Releases
curl -L https://github.com/johnzilla/honeyprompt/releases/latest/download/honeyprompt-x86_64-unknown-linux-musl.tar.gz \
  | tar -xz -C /tmp/

# Install to system path
sudo mv /tmp/honeyprompt /usr/local/bin/honeyprompt
sudo chmod +x /usr/local/bin/honeyprompt

# Verify binary works
/usr/local/bin/honeyprompt --version
```

---

## Section 3: Configure systemd

Copy the unit file from this repository:

```bash
# From the repository root
sudo cp deploy/honeyprompt.service /etc/systemd/system/honeyprompt.service
```

Or create it manually if you don't have the repo on the Droplet:

```bash
sudo tee /etc/systemd/system/honeyprompt.service << 'EOF'
[Unit]
Description=HoneyPrompt honeypot server
Documentation=https://github.com/jthwho/honeyprompt
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=honeyprompt
Group=honeyprompt
WorkingDirectory=/var/lib/honeyprompt
ExecStart=/usr/local/bin/honeyprompt serve /var/lib/honeyprompt/landing
Restart=always
RestartSec=5
StartLimitIntervalSec=600
StartLimitBurst=5

# Graceful shutdown: server listens for SIGINT (Ctrl+C), not SIGTERM.
# Without this, systemctl stop sends SIGTERM which bypasses graceful shutdown
# and may lose in-flight events from the mpsc channel.
KillSignal=SIGINT
TimeoutStopSec=10

# Sandbox hardening
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ReadWritePaths=/var/lib/honeyprompt

[Install]
WantedBy=multi-user.target
EOF
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable honeyprompt
sudo systemctl start honeyprompt
```

Verify it is running:

```bash
# Should show Active: active (running)
sudo systemctl status honeyprompt

# Should show "honeyprompt serve" startup with bind address and nonce count
journalctl -u honeyprompt -n 20
```

---

## Section 4: Install and Configure Caddy

Install Caddy via the official APT repository:

```bash
sudo apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' \
  | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' \
  | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt-get update && sudo apt-get install caddy
```

Copy the Caddyfile from this repository:

```bash
sudo cp deploy/Caddyfile /etc/caddy/Caddyfile
```

**IMPORTANT — Verify DNS before starting Caddy.**

Caddy uses the ACME HTTP-01 challenge to obtain a Let's Encrypt certificate. If DNS has not
propagated, the challenge fails and Caddy falls back to HTTP or logs an error. Wait until:

```bash
# This must return the Droplet's public IP before you proceed
dig +short honeyprompt.sh @1.1.1.1
```

Once DNS is verified, reload Caddy:

```bash
sudo systemctl reload caddy
```

Watch Caddy logs to confirm certificate provisioning:

```bash
journalctl -u caddy -f
# Look for: "certificate obtained successfully" or "serving certificate"
```

Verify HTTPS is working:

```bash
curl -I https://honeyprompt.sh
# Expected: HTTP/2 200
```

---

## Section 5: Firewall Configuration

**CRITICAL: Port 8080 must NOT be publicly accessible.**

honeyprompt binds to `0.0.0.0:8080` — if the firewall does not block this port, agents can
reach the server directly over HTTP, bypassing Caddy and the expected HTTPS URL scheme.

Configure the DigitalOcean Cloud Firewall (recommended) or `ufw`:

**Using DigitalOcean Cloud Firewall:**
1. Go to DigitalOcean Dashboard → Networking → Firewalls
2. Create a new firewall with inbound rules: SSH (22), HTTP (80), HTTPS (443)
3. Apply the firewall to the Droplet
4. Verify port 8080 is blocked: from an external machine, `curl --max-time 5 http://<DROPLET_IP>:8080` should timeout

**Using ufw:**

```bash
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 22/tcp   # SSH
sudo ufw allow 80/tcp   # HTTP (ACME challenge)
sudo ufw allow 443/tcp  # HTTPS
sudo ufw enable
sudo ufw status
```

---

## Section 6: Uptime Monitoring

External uptime monitoring provides alerting that on-server checks cannot — it detects network
failures, ISP outages, and Caddy/systemd failures that leave the machine up but the service
unreachable.

1. Sign up for the UptimeRobot free tier at https://uptimerobot.com (no credit card required)
2. Click "Add New Monitor"
3. Monitor type: HTTPS
4. Friendly name: `honeyprompt.sh`
5. URL: `https://honeyprompt.sh`
6. Monitoring interval: 5 minutes
7. Configure an email alert contact under "My Contacts"
8. Save the monitor

The free tier provides 50 monitors and 5-minute polling — sufficient for the live demo.

Verify the monitor shows "Up" status within the first polling interval (up to 5 minutes after setup).

---

## Section 7: Binary Upgrade Procedure

When a new release is published, follow this exact procedure. The SQLite evidence store is
**never** touched during upgrades — only the binary at `/usr/local/bin/honeyprompt` is replaced.

```bash
# Step 1: Pre-upgrade backup of the evidence store
sudo cp /var/lib/honeyprompt/landing/.honeyprompt/events.db \
  ~/events.db.$(date +%Y%m%d)

# Step 2: Download the new binary
curl -L https://github.com/johnzilla/honeyprompt/releases/latest/download/honeyprompt-x86_64-unknown-linux-musl.tar.gz \
  | tar -xz -C /tmp/

# Step 3: Stop the service (graceful shutdown via SIGINT per KillSignal in unit file)
sudo systemctl stop honeyprompt

# Step 4: Replace the binary
sudo mv /tmp/honeyprompt /usr/local/bin/honeyprompt
sudo chmod +x /usr/local/bin/honeyprompt

# Step 5: Start the service
sudo systemctl start honeyprompt

# Step 6: Verify new version is running
/usr/local/bin/honeyprompt --version
sudo systemctl status honeyprompt
```

**CRITICAL:** The evidence store is at `/var/lib/honeyprompt/landing/.honeyprompt/events.db`.
This file contains all recorded agent detections — it is the primary output of the deployment.
Only `/usr/local/bin/honeyprompt` is replaced during upgrades. Do not rm or overwrite the
`/var/lib/honeyprompt/` directory.

---

## Section 8: Troubleshooting

### Service cycling (auto-restart loop)

**Symptom:** `systemctl status honeyprompt` shows `Active: activating (auto-restart)` cycling.

```bash
journalctl -u honeyprompt -n 50
```

Common causes:
- **Missing files:** `honeyprompt.toml` or `output/callback-map.json` not found at expected path
- **Permission denied on DB:** honeyprompt user cannot write to `/var/lib/honeyprompt/`
- **Binary not found:** `/usr/local/bin/honeyprompt` does not exist or is not executable

Verify file access as the service user:

```bash
sudo -u honeyprompt ls /var/lib/honeyprompt/landing/honeyprompt.toml
sudo -u honeyprompt ls /var/lib/honeyprompt/landing/output/callback-map.json
sudo -u honeyprompt touch /var/lib/honeyprompt/landing/.honeyprompt/test && \
  sudo -u honeyprompt rm /var/lib/honeyprompt/landing/.honeyprompt/test && \
  echo "Write permission OK"
```

### Caddy TLS failure

**Symptom:** `curl -I https://honeyprompt.sh` returns an SSL error or self-signed cert warning.

```bash
journalctl -u caddy -n 50
```

Common causes:
- DNS has not propagated yet (`dig +short honeyprompt.sh @1.1.1.1` returns wrong IP or nothing)
- Port 80 is blocked by firewall (ACME HTTP-01 challenge requires port 80 to be open)
- Caddy config file is malformed (check `sudo caddy validate --config /etc/caddy/Caddyfile`)

After fixing DNS or firewall, retry cert provisioning:

```bash
sudo systemctl restart caddy
journalctl -u caddy -f
```

### DB permission denied

```bash
# Verify the service user can write to the data directory
sudo -u honeyprompt touch /var/lib/honeyprompt/test
# If this fails: fix ownership
sudo chown -R honeyprompt:honeyprompt /var/lib/honeyprompt
```

### Confirming graceful shutdown works (SIGINT)

The systemd unit uses `KillSignal=SIGINT` because the server's `shutdown_signal()` function
listens for SIGINT (Ctrl+C), not SIGTERM. To verify graceful shutdown completes before timeout:

```bash
sudo systemctl stop honeyprompt
journalctl -u honeyprompt -n 10
# Should show shutdown log lines, NOT "Killed" (which would indicate SIGKILL timeout)
```

---

## Section 9: Architecture Reference

```
Internet
    |
    | HTTPS :443
    v
Caddy (auto Let's Encrypt TLS)
    |
    | HTTP :8080 (localhost only)
    v
honeyprompt serve /var/lib/honeyprompt/landing
    |
    v
/var/lib/honeyprompt/landing/.honeyprompt/events.db  (SQLite evidence store)
```

### File Layout on Droplet

```
/usr/local/bin/honeyprompt                          # static musl binary (replaced on upgrade)
/etc/systemd/system/honeyprompt.service             # from deploy/honeyprompt.service
/etc/caddy/Caddyfile                                # from deploy/Caddyfile
/var/lib/honeyprompt/                               # owned by honeyprompt:honeyprompt
  landing/                                          # rsync'd from repo landing/
    honeyprompt.toml                                # callback_base_url, bind_address
    output/                                         # pre-generated honeypot files
      index.html                                    # honeypot page with canaries
      callback-map.json                             # nonce → callback URL map
      robots.txt                                    # disallow rules for crawlers
      ai.txt                                        # AI-specific disallow rules
    .honeyprompt/
      events.db                                     # SQLite evidence store (never delete!)
```

### Key Configuration Values

| Setting | Value | Source |
|---------|-------|--------|
| `ExecStart` arg | `/var/lib/honeyprompt/landing` | `deploy/honeyprompt.service` |
| `KillSignal` | `SIGINT` | Required for graceful shutdown |
| `ReadWritePaths` | `/var/lib/honeyprompt` | Covers DB path under landing dir |
| `callback_base_url` | `https://honeyprompt.sh` | `landing/honeyprompt.toml` |
| `bind_address` | `0.0.0.0:8080` | `landing/honeyprompt.toml` |
| Caddy domain | `honeyprompt.sh` | `deploy/Caddyfile` |
| Caddy proxy target | `localhost:8080` | `deploy/Caddyfile` |
| Events DB path | `/var/lib/honeyprompt/landing/.honeyprompt/events.db` | Derived from `ExecStart` path |
