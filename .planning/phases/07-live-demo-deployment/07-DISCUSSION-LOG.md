# Phase 7: Live Demo Deployment - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.

**Date:** 2026-03-31
**Phase:** 07-live-demo-deployment
**Areas discussed:** Hosting platform, Persistence strategy

---

## Hosting Platform

| Option | Description | Selected |
|--------|-------------|----------|
| Fly.io | Managed, ~$2/month, fly.toml, auto-TLS | |
| VPS + systemd | Manual setup, full control, $5/month | |
| Docker on any VPS | Dockerfile + docker-compose + Caddy sidecar | |

**User's input:** "I use digital ocean for deployment" — redirected to DO-specific options.

| DO Option | Description | Selected |
|-----------|-------------|----------|
| Droplet + systemd | $4-6/month, upload binary, systemd, Caddy TLS. SQLite persists on disk. | ✓ |
| App Platform (Docker) | $5/month, auto-deploy, but stateless — SQLite resets on deploy. | |
| App Platform + Managed DB | $15+/month, overkill for research demo. | |

**User's choice:** Droplet + systemd

---

## Persistence Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Persistent | SQLite at /var/lib/honeyprompt/events.db, survives restarts and upgrades. | ✓ |
| Ephemeral | DB resets on binary upgrade. Lose historical data. | |

**User's choice:** Persistent

---

## Claude's Discretion

- deploy.sh script vs documented steps
- Caddy config format
- Uptime monitoring provider
- Log rotation approach

## Deferred Ideas

- Fly.io deployment
- Docker Compose with Caddy sidecar
- Automated deploy-on-tag via GitHub Actions SSH
