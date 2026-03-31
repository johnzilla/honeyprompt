---
gsd_state_version: 1.0
milestone: v2.0
milestone_name: Ship & Learn
status: verifying
stopped_at: "Checkpoint: Task 2 of 08-02 — awaiting Google Search Console submission by user"
last_updated: "2026-03-31T17:46:43.641Z"
last_activity: 2026-03-31
progress:
  total_phases: 4
  completed_phases: 3
  total_plans: 9
  completed_plans: 8
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-29)

**Core value:** Provide graduated, verifiable proof that AI agents follow prompt-injection instructions from untrusted web content — without requiring secrets or causing harm.
**Current focus:** Phase 08 — readme-and-public-launch

## Current Position

Phase: 08
Plan: Not started
Status: Phase complete — ready for verification
Last activity: 2026-03-31

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**

- Total plans completed: 0 (v2.0) / 10 (v1.0)
- Average duration: ~25 min/plan (v1.0 baseline)
- Total execution time: —

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- v2.0 approach: "Ship & Learn" — deploy live demo + build test-agent, collect evidence before building SaaS infrastructure
- v2.0: test-agent does NOT bundle a tunnel — users provide their own public endpoint for remote agents
- v2.0: CI workflow (REL-01) is a Phase 5 pre-step — validates green baseline before any test-agent code lands
- v2.0: All *-apple-darwin release targets must run on macos-latest runners (rusqlite bundled requires macOS SDK)
- v2.0: README written last (Phase 8) — references features, binaries, and demo URL that exist only after Phases 5-7
- [Phase 05-test-agent-subcommand]: All GitHub Actions pinned to full commit SHAs (not version tags) per D-10: checkout v4, dtolnay/rust-toolchain master, Swatinem/rust-cache v2.9.1
- [Phase 05-02]: Pre-bind std::net::TcpListener before async runtime, then convert via from_std() — eliminates port-stealing race condition in test-agent
- [Phase 05-02]: Exit codes: 0=no callbacks (safe agent), 1=canaries triggered, 2=error (std::process::exit per D-05)
- [Phase 05-02]: detections_by_tier() returns [u32;3] per-tier counts from SQLite, excluding KnownCrawler sessions
- [Phase 05-03]: Scorecard::render_text() and render_json() are pure methods on Scorecard; main dispatch is the only call site
- [Phase 05-03]: Test module moved to end of file to satisfy clippy::items_after_test_module lint
- [Phase 06-release-infrastructure]: cargo install --git is primary source install path (crates.io publish deferred)
- [Phase 06-release-infrastructure]: Binary names use full Rust target triple per D-03 in README platform table
- [Phase 06-release-infrastructure]: taiki-e/upload-rust-binary-action auto-installs cross for Linux musl targets — no manual cross install step needed
- [Phase 06-release-infrastructure]: Release workflow: SHA pins for taiki-e actions — create-gh-release-action@c5baa0b5dc700cf06439d87935e130220a6882d9 (v1), upload-rust-binary-action@0e34102c043ded9f2ca39f7af5cd99a540c61aff (v1)
- [Phase 07-live-demo-deployment]: ExecStart uses /var/lib/honeyprompt/landing so DB at /var/lib/honeyprompt/landing/.honeyprompt/events.db stays within ReadWritePaths — satisfies D-04 persistent evidence store
- [Phase 07-live-demo-deployment]: KillSignal=SIGINT required in systemd unit: server shutdown_signal() listens for SIGINT (ctrl_c), not SIGTERM; without this systemctl stop bypasses graceful shutdown and risks losing in-flight mpsc channel events
- [Phase 08]: Project Status table rows use 'Phase N' prefix for clarity and grep-based verification
- [Phase 08-readme-and-public-launch]: X post drafting only — user posts manually per D-06; Google Search Console submission is human-only per D-07

### Pending Todos

- [deferred] Diversify payload instruction text across embedding locations — current catalog produces repetitive content when multiple locations render similar instruction text (user feedback from 01-03 checkpoint)

### Blockers/Concerns

- [Phase 8] crates.io publish workflow not researched — verify `cargo install honeyprompt` works before writing install instructions
- [Phase 5] Default 60s timeout is an assumption — validate against real agent behavior post-launch

## Session Continuity

Last session: 2026-03-31T17:22:00.055Z
Stopped at: Checkpoint: Task 2 of 08-02 — awaiting Google Search Console submission by user
Resume file: None
