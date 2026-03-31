# Deploying HoneyPrompt

Docker Compose deployment with auto-TLS via Caddy.

Architecture: Internet → Caddy (:443 TLS) → honeyprompt (:8080) → SQLite DB

## Prerequisites

- A server with Docker and Docker Compose installed
- A domain (honeyprompt.sh) pointed at the server's IP
- Ports 80 and 443 open

## Deploy

```bash
git clone https://github.com/YOUR_USER/honeyprompt.git
cd honeyprompt/deploy
docker compose up -d
```

Caddy auto-provisions a Let's Encrypt certificate for honeyprompt.sh on first request.

## Verify

```bash
curl -I https://honeyprompt.sh        # Should return HTTP/2 200
docker compose logs honeyprompt        # Should show "honeyprompt serve ... ready"
```

## Data

- SQLite DB persists in the `hp-db` Docker volume
- Callback events survive container restarts and redeployments
- Backup: `docker compose cp honeyprompt:/data/.honeyprompt/events.db ./events-backup.db`

## Upgrade

```bash
cd honeyprompt
git pull
cd deploy
docker compose build honeyprompt
docker compose up -d honeyprompt
```

## Monitor

Set up a free [UptimeRobot](https://uptimerobot.com) HTTP(S) monitor pointing at `https://honeyprompt.sh` for external uptime alerts.
