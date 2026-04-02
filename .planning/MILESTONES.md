# Milestones

## v4.0 Self-Hosted UX (Shipped: 2026-04-02)

**Phases completed:** 2 phases, 4 plans, 7 tasks

**Key accomplishments:**

- `honeyprompt setup` interactive wizard with dialoguer prompts for domain/bind/tiers/title, DNS warning, and TOML write with permission error handling
- `--domain` flag on `honeyprompt serve` generates an ephemeral tempdir honeypot and serves it immediately with https://{domain} callback URLs, bind 0.0.0.0:8080, and all tiers enabled — no init/generate steps required
- README rewritten with four-subsection Deploy Your Own guide, clear persona separation between honeyprompt.sh demo and self-hosted path, and Project Status table extended through Phase 12.
- Three parameterized deployment templates in deploy/templates/ — Docker Compose + Caddy + systemd — each using {DOMAIN} placeholder users replace once to get a working self-hosted honeyprompt instance

---

## v3.0 Public Presence (Shipped: 2026-04-01)

**Phases completed:** 2 phases, 3 plans, 3 tasks

**Key accomplishments:**

- `src/store/mod.rs`
- STATUS: CHECKPOINT PENDING — Task 2 (DNS + GitHub Pages) awaits human action

---

## v2.0 Ship & Learn (Shipped: 2026-03-31)

**Phases completed:** 4 phases, 9 plans, 15 tasks

**Key accomplishments:**

- GitHub Actions CI with three SHA-pinned parallel jobs (fmt, clippy, test) using dtolnay/rust-toolchain and Swatinem/rust-cache v2.9.1
- Ephemeral generate-serve-wait-score pipeline via CancellationToken timeout, pre-bound TcpListener, and per-tier SQLite scorecard query
- Per-tier text and JSON scorecard rendering wired to honeyprompt test-agent with D-05 exit codes (0/1/2)
- README rewritten with two clear install paths — prebuilt binaries via GitHub Releases curl one-liner and cargo install --git from source — plus platform table covering all four targets and updated Project Status through Phase 6
- systemd unit with KillSignal=SIGINT + Caddyfile reverse proxy + distroless Dockerfile for honeyprompt.sh live deployment
- Status:
- README polished with honeyprompt.sh live demo link, all GitHub URLs corrected to johnzilla/honeyprompt, Project Status updated to show 8 phases complete, and Safety Model expanded to Ethics and Safety with what-HoneyPrompt-is-NOT framing
- 3 X post draft variations (under 280 chars, GitHub-linked) ready for user; Google Search Console submission steps presented as human-action checkpoint

---

## v1.0 MVP (Shipped: 2026-03-29)

**Phases completed:** 4 phases, 10 plans, 15 tasks

**Key accomplishments:**

- Compilable Rust project with clap derive CLI (init/generate), serde+toml Config with round-trip test, and shared domain types (Tier, EmbeddingLocation, Payload, NonceMapping) used by all downstream plans
- 6-payload curated catalog embedded via rust-embed, 16-char CSPRNG nonces, and WAL-mode SQLite schema with replay detection fields locked before any network code
- Complete init+generate CLI pipeline producing deployable honeypot with hard-coded warnings, 5-location payload embedding, nonce-keyed callbacks, robots.txt AI disallows, and ai.txt policy declarations
- One-liner:
- One-liner:
- Axum HTTP server on single port serving static honeypot pages and /cb/v1/{nonce} callback beacons with 204-always handler, full event pipeline, and graceful shutdown
- AppState TUI business logic (filter/sort/replay/stats) with 17 unit tests plus MonitorArgs CLI wiring — the testable logic layer for Plan 02's Ratatui rendering
- Ratatui-based real-time event monitor with 4-panel layout, integrated Axum server mode, DB attach mode, and terminal panic safety
- 1. [Rule 3 - Blocking] Removed conflicting src/report.rs stub
- honeyprompt.toml config for honeyprompt.sh with all 3 tiers, dogfooded landing page generated via `honeyprompt generate landing/` and committed as durable repo artifacts

---
