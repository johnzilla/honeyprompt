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

Images are built by GitHub Actions (`.github/workflows/docker.yml`) on GitHub's x86_64 runner — never locally. Pushing a `v*` tag builds and pushes `ghcr.io/johnzilla/honeyprompt:latest` and `:<version>`. To rebuild an existing tag: Actions → Docker Image → Run workflow → enter the tag.

Once the workflow is green, pull on the droplet:

```bash
ssh root@YOUR_DROPLET_IP "cd /opt/honeyprompt && docker compose pull && docker compose up -d"
```

One-time setup: the ghcr package must allow this repo's Actions to write to it (package settings → Manage Actions access → add `johnzilla/honeyprompt` with Write).

## Verify

```bash
curl -I https://honeyprompt.sh       # HTTP/2 200
docker compose logs honeyprompt       # "honeyprompt serve ... ready"
```

## Webhook alerts (optional)

Secrets go in `/opt/honeyprompt/.env` on the droplet (read by `docker-compose.yml`), not in the image or the repo:

```bash
cd /opt/honeyprompt
install -m 600 /dev/null .env
nano .env
```

```
HONEYPROMPT_WEBHOOK_URL=https://alerts.example.com/honeyprompt
HONEYPROMPT_WEBHOOK_SECRET=<output of: openssl rand -hex 32>
```

Then apply it and send a test alert:

```bash
docker compose up -d
docker compose exec honeyprompt /usr/local/bin/honeyprompt webhook-test /landing
docker compose logs honeyprompt | grep webhook    # "webhook: alerts.example.com (signed: yes, ...)"
```

`format` and `send` (see the main README) go in `honeyprompt.toml` inside the volume, or keep the defaults (`json`, `notable`).

## Data

- Config, generated site, nonce map and SQLite DB persist in the `hp-data` Docker volume — survives image updates. To rotate all nonces, stop the stack, `docker volume rm honeyprompt_hp-data`, and start it again
- Backup (stop first so the WAL is consistent; `docker cp` works on stopped containers). On the droplet:
  ```bash
  cd /opt/honeyprompt && docker compose stop honeyprompt
  docker cp "$(docker compose ps -aq honeyprompt)":/landing/.honeyprompt /root/hp-backup
  docker compose start honeyprompt
  ```
  Then `scp -r root@DROPLET:/root/hp-backup .` from your machine. The directory holds `events.db` plus its `-wal`/`-shm` files and the session salt.

## Monitor

Set up a free [UptimeRobot](https://uptimerobot.com) HTTP(S) monitor for `https://honeyprompt.sh`.
