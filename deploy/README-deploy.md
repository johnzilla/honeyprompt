# Deploying HoneyPrompt

Docker Compose deployment with auto-TLS via Caddy on a DigitalOcean Droplet.

Architecture: Internet → Caddy (:443 TLS) → honeyprompt (:8080) → SQLite DB (persistent volume)

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

Caddy auto-provisions a Let's Encrypt certificate on first request.

## Redeploy

Build and push a new image locally, then pull on the droplet:

```bash
docker build -t ghcr.io/johnzilla/honeyprompt:latest .
docker push ghcr.io/johnzilla/honeyprompt:latest
ssh root@YOUR_DROPLET_IP "cd /opt/honeyprompt && docker compose pull && docker compose up -d"
```

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
