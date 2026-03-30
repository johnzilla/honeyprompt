# Roadmap: HoneyPrompt

## Milestones

- ✅ **v1.0 MVP** — Phases 1-4 (shipped 2026-03-29)
- 🚧 **v2.0 Ship & Learn** — Phases 5-8 (in progress)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-4) — SHIPPED 2026-03-29</summary>

- [x] Phase 1: Generation Pipeline (3/3 plans) — completed 2026-03-29
- [x] Phase 2: Server and Detection (3/3 plans) — completed 2026-03-29
- [x] Phase 3: TUI Monitor (2/2 plans) — completed 2026-03-29
- [x] Phase 4: Report and Landing (2/2 plans) — completed 2026-03-29

</details>

### 🚧 v2.0 Ship & Learn (In Progress)

**Milestone Goal:** Get HoneyPrompt in front of real users and collect evidence of whether AI agents trigger canary payloads in the wild.

- [ ] **Phase 5: test-agent Subcommand** - Add `honeyprompt test-agent` with scorecard, exit codes, and CI gating all new code
- [ ] **Phase 6: Release Infrastructure** - Cross-platform binary releases triggered by version tag
- [ ] **Phase 7: Live Demo Deployment** - `honeyprompt serve` running persistently at honeyprompt.sh with HTTPS
- [ ] **Phase 8: README and Public Launch** - Rewrite README with value prop, install instructions, and demo link; launch publicly

## Phase Details

### Phase 5: test-agent Subcommand
**Goal**: Users can run a bounded compliance test against any AI agent and get a verifiable pass/fail scorecard with process exit codes suitable for CI
**Depends on**: Phase 4
**Requirements**: TEST-01, TEST-02, TEST-03, TEST-04, TEST-05, REL-01
**Success Criteria** (what must be TRUE):
  1. User can run `honeyprompt test-agent` and receive a per-tier (1/2/3) compliance scorecard after the test window closes
  2. User can configure listen address, timeout duration, and output format via `--listen`, `--timeout`, and `--format` flags
  3. Process exits with code 0 (no canaries triggered), 1 (one or more triggered), or 2 (error/no data) — verifiable with `echo $?`
  4. User can pass `--format json` and receive structured JSON output for use in CI pipelines
  5. Every push and PR to main triggers a green CI badge (cargo test, clippy, fmt) visible on the repository
**Plans**: TBD

### Phase 6: Release Infrastructure
**Goal**: Pre-built binaries for all four target platforms are produced automatically on every version tag and downloadable from GitHub Releases
**Depends on**: Phase 5
**Requirements**: REL-02, REL-03
**Success Criteria** (what must be TRUE):
  1. Pushing a `v*` tag causes GitHub Actions to build and upload binaries for x86_64-linux, aarch64-linux, x86_64-darwin, and aarch64-darwin
  2. A security researcher can download a pre-built binary and run `honeyprompt --version` without compiling from source
  3. README includes both `cargo install honeyprompt` and pre-built binary download paths
**Plans**: TBD

### Phase 7: Live Demo Deployment
**Goal**: honeyprompt.sh serves a live honeypot with canary payloads over HTTPS and stays up without manual intervention
**Depends on**: Phase 6
**Requirements**: DEPLOY-01, DEPLOY-02, DEPLOY-03
**Success Criteria** (what must be TRUE):
  1. A Dockerfile or systemd unit file in the repository lets any user deploy `honeyprompt serve` as a persistent process with a single command
  2. honeyprompt.sh responds over HTTPS with a live honeypot page containing canary payloads
  3. The live demo process auto-restarts on failure and is monitored for uptime
**Plans**: TBD
**UI hint**: yes

### Phase 8: README and Public Launch
**Goal**: The README communicates HoneyPrompt's value to a security researcher landing cold, provides a complete quick-start, and the project is publicly announced
**Depends on**: Phase 7
**Requirements**: LAUNCH-01, LAUNCH-02, LAUNCH-03
**Success Criteria** (what must be TRUE):
  1. A security researcher arriving at the repository cold can understand the value proposition, run `honeyprompt test-agent`, and install the binary — all within the README, in under 5 commands
  2. The README links to the live demo at honeyprompt.sh and honeyprompt.sh is submitted to Google Search Console for crawler discoverability
  3. The README contains an ethics/scope section that explains what HoneyPrompt does and does not do, and the project is publicly announced (HN, Reddit r/netsec, Twitter/X)
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Generation Pipeline | v1.0 | 3/3 | Complete | 2026-03-29 |
| 2. Server and Detection | v1.0 | 3/3 | Complete | 2026-03-29 |
| 3. TUI Monitor | v1.0 | 2/2 | Complete | 2026-03-29 |
| 4. Report and Landing | v1.0 | 2/2 | Complete | 2026-03-29 |
| 5. test-agent Subcommand | v2.0 | 0/? | Not started | - |
| 6. Release Infrastructure | v2.0 | 0/? | Not started | - |
| 7. Live Demo Deployment | v2.0 | 0/? | Not started | - |
| 8. README and Public Launch | v2.0 | 0/? | Not started | - |
