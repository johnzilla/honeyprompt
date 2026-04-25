# Milestones

## v5.0 Tiers 4 & 5 — Capability Introspection + Multi-step Compliance (Shipped: 2026-04-25)

**Phases completed:** 3 phases, 10 plans, ~30 tasks

**Git range:** `8644aa2..a3bf38e` (75 commits, +2806/−87 Rust LOC across 15 files)

**Key accomplishments:**

- Tier 4 (Capability Introspection) — 3 payload templates probing distinct dimensions (tools, model-identity, permissions), base64-encoded sorted lists submitted via `GET /cb/v4/{nonce}/{b64_list}`, server-sanitized with `^[a-z0-9_,.\-]{1,256}$` regex; agent-chosen safe menu, never secrets
- Tier 5 (Multi-step Compliance Chain) — 3 payload templates with numbered dependent steps, deterministic `verification_seed` embedded in JSON-LD derived from first 8 hex chars of nonce, 3-digit proof `((seed+a)*b) %mod` submitted via `GET /cb/v5/{nonce}/{proof}` and server-verified against a re-computed expected value
- Additive SQLite migration adds `t4_capability`, `t5_proof`, `t5_proof_valid` columns — v4.0 databases open unchanged and T1–T3 rows read back byte-identical; replay detection and session grouping behave uniformly across all 5 tiers
- `/cb/v1/{nonce}` route response and stored-row shape unchanged — verified by existing integration tests passing without modification (backward-compat anchor)
- Monitor TUI extended with compact EVIDENCE column + always-visible detail pane (T1–T3 shows payload_id/embedding_loc/full-nonce; T4 shows decoded capability list; T5 shows proof + formula + VALID/INVALID), 6-state tier filter cycle (`All → T1 → T2 → T3 → T4 → T5 → All`), and "always-show chrome" policy that keeps T4/T5 rows visible even on zero-count v4.0 databases
- Markdown disclosure report gains Evidence column interleaving T4 tool lists and T5 proofs with existing T1–T3 evidence; executive summary extends to all 5 tiers
- `honeyprompt test-agent` scorecard extended to 5-tier shape (`tiers: [bool; 5]`, `tier_counts: [u32; 5]`, `"n/5"` score) while preserving the v2.0 exit-code contract (0=no canaries, 1=any tier triggered including T4-only or T5-only, 2=error) — 2 new unit tests (`test_exit_code_t4_only`, `test_exit_code_t5_only`) enforce the backward-compat guarantee
- Public docs sync: README Proof Levels gains concrete inline examples per tier (including a worked T5 formula `seed 137 → ((137+42)·17) %1000 → "043"` matching the real `t5-semantic-prose` catalog constants), Ethics/Safety explicitly reaffirms no-secrets guarantees for T4 (agent-chosen safe menu) and T5 (page-visible arithmetic), Project Status table extended through Phase 15, TODOS.md gets a `## Shipped` section referencing v5.0 phases, and landing page (`docs/index.html`) live-stats display extended with T4 `capability` and T5 `multistep` session counts (with `|| 0` fallback for older server deployments)

**Timeline:** 2026-04-24 → 2026-04-25 (single-day milestone). CONTEXT.md-driven planning with zero scope creep; every D-15-* decision byte-identically honored in committed code/docs. Phase 15 closed with 212/212 tests passing, `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt --check` clean.

**Archived files:**
- `.planning/milestones/v5.0-ROADMAP.md` — full phase-by-phase details + decisions + deferred items
- `.planning/milestones/v5.0-REQUIREMENTS.md` — all 25 requirements validated (PAYLOAD-01..05, SERVER-01..04, STORE-01..04, UI-01..05, TESTAGENT-01..03, DOCS-01..04)

---

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
