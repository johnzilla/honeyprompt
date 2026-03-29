# Milestones

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
