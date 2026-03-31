# Deploying HoneyPrompt

Auto-deploys on every push to main via GitHub Actions → GHCR → Droplet.

Architecture: Internet → Caddy (:443 TLS) → honeyprompt (:8080) → SQLite DB (persistent volume)

## How it works

1. Push to `main` triggers `.github/workflows/deploy.yml`
2. GitHub Actions builds the Docker image and pushes to `ghcr.io/johnzilla/honeyprompt:latest`
3. GitHub Actions SSHs to the droplet and runs `docker compose pull && up -d`

## One-time droplet setup

1. **Provision a DigitalOcean Droplet** ($4-6/month, Ubuntu 22.04+)

2. **Install Docker:**
   ```bash
   curl -fsSL https://get.docker.com | sh
   ```

3. **Copy deploy files to the droplet:**
   ```bash
   scp deploy/docker-compose.yml deploy/Caddyfile root@YOUR_DROPLET_IP:/opt/honeyprompt/
   ```

4. **Start the stack:**
   ```bash
   ssh root@YOUR_DROPLET_IP "cd /opt/honeyprompt && docker compose up -d"
   ```

5. **Point DNS:** Add an A record for `honeyprompt.sh` → droplet IP

6. **Add GitHub Secrets** (Settings → Secrets → Actions):
   - `DEPLOY_HOST` — droplet IP
   - `DEPLOY_USER` — `root` (or deploy user)
   - `DEPLOY_KEY` — SSH private key for the droplet

After this, every push to main auto-deploys.

## Verify

```bash
curl -I https://honeyprompt.sh       # HTTP/2 200
docker compose logs honeyprompt       # "honeyprompt serve ... ready"
```

## Data

- SQLite DB persists in the `hp-db` Docker volume — survives image updates
- Backup: `ssh root@DROPLET "docker compose -f /opt/honeyprompt/docker-compose.yml cp honeyprompt:/landing/.honeyprompt/events.db -" > events-backup.db`

## Monitor

Set up a free [UptimeRobot](https://uptimerobot.com) HTTP(S) monitor for `https://honeyprompt.sh`.
