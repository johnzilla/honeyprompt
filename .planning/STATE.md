---
gsd_state_version: 1.0
milestone: v4.0
milestone_name: Self-Hosted UX
status: verifying
stopped_at: Completed 12-02-PLAN.md
last_updated: "2026-04-01T23:16:58.607Z"
last_activity: 2026-04-01
progress:
  total_phases: 2
  completed_phases: 2
  total_plans: 4
  completed_plans: 4
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-01)

**Core value:** Provide graduated, verifiable proof that AI agents follow prompt-injection instructions from untrusted web content — without requiring secrets or causing harm.
**Current focus:** Phase 11 — setup-wizard-zero-config-serve

## Current Position

Phase: 12
Plan: Not started
Status: Phase complete — ready for verification
Last activity: 2026-04-01

Progress: [░░░░░░░░░░] 0%

## Performance Metrics

**Velocity:**

- Total plans completed: 0 (v4.0) / 17 (v1.0–v3.0 combined)
- Average duration: ~25 min/plan (v2.0 baseline)
- Total execution time: —

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [v4.0 design]: `dialoguer` crate for interactive CLI prompts in setup wizard
- [v4.0 design]: tempdir serve mode reuses existing generate pipeline (same pattern as test-agent)
- [v4.0 design]: --domain flag sets callback_base_url, bind 0.0.0.0:8080, all payloads enabled
- [v4.0 design]: flag > config file > built-in defaults precedence chain
- [v4.0 design]: deploy/templates/ with {DOMAIN} placeholder pattern for docker-compose, systemd, Caddyfile
- [Phase 11-01]: dialoguer crate for interactive CLI prompts; check_dns returns Ok(bool) for non-blocking warning semantics; Setup guard exits with process::exit(1) on existing config
- [Phase 11-02]: config_with_overrides precedence: flag > domain-defaults > base-config, with domain implying bind=0.0.0.0:8080 and tiers=[1,2,3]
- [Phase 11-02]: Tempdir mode triggers when --domain set, path=='.', and no honeyprompt.toml present; explicit --path always uses standard mode
- [Phase 12-02]: Templates use {DOMAIN} placeholder that users globally-replace — no templating engine required
- [Phase 12-02]: honeyprompt.service ExecStart updated to use --domain flag matching v4.0 zero-config UX

### Pending Todos

- [deferred] Diversify payload instruction text across embedding locations — current catalog produces repetitive content when multiple locations render similar instruction text (user feedback from 01-03 checkpoint)

### Blockers/Concerns

None for current phase.

## Session Continuity

Last session: 2026-04-01T23:11:55.305Z
Last activity: 2026-04-01 — Roadmap written for v4.0 Self-Hosted UX (phases 11-12)
Stopped at: Completed 12-02-PLAN.md
Resume file: None
