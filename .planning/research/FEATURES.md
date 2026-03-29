# Feature Landscape

**Domain:** AI agent detection / honeypot canary token security tool
**Researched:** 2026-03-29 (v2.0 extension; v1.0 content preserved)
**Confidence:** HIGH (core canary/honeypot features) / MEDIUM (AI-agent-specific features, emerging space) / MEDIUM (release infra patterns)

---

## Context: What HoneyPrompt Is

HoneyPrompt occupies a novel intersection: canary tokens (Thinkst-style passive tripwires) + honeypots (behavior capture) + prompt injection research (AI agent compliance measurement). No direct competitor does passive, deployable, graduated-proof canary tokens specifically for AI browsing agents. Classic canary tools target human attackers; classic honeypots measure what attackers do; HoneyPrompt measures specifically whether AI agents follow injected instructions from web content they browse.

This means features must satisfy three distinct user needs simultaneously:
1. **Canary token workflows** — deploy, forget, get alerted when triggered
2. **Honeypot research workflows** — capture, store, analyze, export attacker behavior
3. **AI agent security research** — graduated proof, payload diversity, ethical constraints

---

## Status: What Is Already Built (v1.0)

All of the following are shipped and validated:

- `init`, `generate`, `serve`, `monitor`, `report` CLI workflow
- 3-tier prompt-injection payload catalog (arbitrary, conditional, computed callbacks)
- SQLite event store with replay detection and session grouping
- HTTP callback listener (integrated with Axum serve)
- Agent fingerprinting from request metadata (IP, UA, headers)
- Known-crawler catalog with UA-primary classification
- Ratatui TUI monitor with live event table, vim-style controls
- Markdown disclosure report with executive summary and full metadata
- robots.txt + ai.txt generation with AI-specific disallow rules
- Self-dogfooded honeyprompt.sh landing page with embedded canaries

---

## V2.0 Target Features

The four new capability areas for v2.0 Ship & Learn:

1. `honeyprompt test-agent` — interactive compliance runner with pass/fail scorecard
2. GitHub Actions CI/CD — automated test, lint, and cross-platform binary release
3. Live demo deployment — containerized or binary-on-VPS server at honeyprompt.sh
4. Public launch assets — README rewrite, blog post, HN/Reddit/Twitter distribution

---

## Table Stakes (v2.0)

Features users of a v2.0 security research CLI tool will expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **`test-agent` subcommand** | Any tool claiming to test AI agents needs a single command that runs a test and returns a pass/fail result; security researchers expect CLI-first, scriptable testing | MEDIUM | Spins up a test server, fires configured agent requests or awaits callbacks, evaluates tier compliance, exits with structured result |
| **Machine-readable output (`--format json`)** | Researchers pipe results to other tools; CI/CD pipelines need parseable output; the pattern is universal across security CLI tools (Snyk, Semgrep, Nuclei) | LOW | Default output is human-readable text; `--format json` emits structured JSON; field names must be stable |
| **Exit codes following Unix convention** | CI/CD integration requires exit codes; tools without correct exit codes cannot be used in pipelines; 0 = pass, 1 = fail, 2 = error is the established convention (Snyk, Semgrep) | LOW | Exit 0: all tests passed. Exit 1: one or more compliance failures detected. Exit 2: error/misconfiguration. Must be documented |
| **`--timeout` flag** | Agent tests have inherent time uncertainty; researchers need bounded test runs; Semgrep, curl, and most test harnesses expose `--timeout` | LOW | Default timeout should be generous (60–120s) but overridable; timeout exit should be distinguishable (exit 2 or separate exit 3) |
| **GitHub Actions CI workflow** | Any actively maintained OSS tool is expected to have CI; missing CI signals abandonware to potential contributors | LOW | `cargo test`, `cargo clippy`, `cargo fmt --check` on push and PR; standard Rust CI template |
| **Binary release artifacts** | Security researchers download tools as binaries; they do not want to compile from source for evaluation; absence of releases on GitHub is a significant adoption barrier | MEDIUM | Minimum: x86_64-unknown-linux-musl, x86_64-apple-darwin, aarch64-apple-darwin. Triggered by git tag |
| **Installation one-liner** | Every successful CLI security tool (ripgrep, fd, bat, cargo-nextest) provides a curl-pipe-sh or package manager install; absence raises friction to near-zero installs | LOW | `cargo install honeyprompt` always works; provide GitHub Release download links and checksums in README |
| **README with value prop and quick-start** | A tool's README is its product page; security researchers evaluate within 60 seconds; a poor README means no adoption regardless of quality | LOW | Value prop in first paragraph, quick-start under 5 commands, install section, screenshot of TUI |

---

## Differentiators (v2.0)

Features that set the v2.0 release apart from a simple CLI tool with a binary release.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Compliance scorecard output** | Security tooling increasingly uses scorecard framing (OpenSSF Scorecard, Snyk, etc.); a per-tier pass/fail table provides a richer signal than a single boolean result | MEDIUM | Show: tier tested, payload used, callback received Y/N, time to callback, pass/fail per tier; summary line at bottom |
| **`--listen` flag for passive test mode** | Enables test-agent to work with real browsing agents pointed at the server (not just synthetic requests); this makes the subcommand useful for both synthetic and real-world testing | LOW | `--listen` starts the server and waits for callbacks within timeout; `--url` mode fires synthetic request to target agent |
| **Live demo at honeyprompt.sh** | A live deployment is the best possible demo; researchers can see the TUI screenshot, try the landing page, observe real captures; builds credibility far faster than documentation | MEDIUM | Deployment target: VPS with static binary + systemd; OR Docker container on Fly.io / Render; canary payloads embedded live |
| **Cross-platform release with checksums** | Security tools must be verifiable; providing SHA-256 checksums with release artifacts signals security consciousness; cargo-dist does this automatically | LOW | cargo-dist generates checksums and installer scripts automatically; adopt it rather than hand-rolling |
| **Shell installer + PowerShell installer** | cargo-dist generates these; they lower installation friction for users who are not Rust developers | LOW | cargo-dist's generated `install.sh` handles platform detection and binary download automatically |
| **Automated release pipeline (tag → release)** | Reduces maintainer burden; a single `git tag v2.0.0 && git push --tags` should trigger the entire release; this is table stakes for long-term maintainability | LOW | GitHub Actions release workflow triggered by `v*` tags; cargo-dist handles artifact building and GitHub Release creation |

---

## Anti-Features (v2.0)

Features that seem valuable for a v2.0 release but would add scope without proportional return.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Package manager submissions (brew, apt, choco)** | Each requires ongoing maintenance; homebrew formulae break on updates; too early for v2.0 | Provide `cargo install honeyprompt` and direct binary downloads; add brew tap after sustained adoption |
| **`--format html` or PDF report from test-agent** | Scope creep; HTML/PDF reports are a different workflow from the pass/fail test runner | JSON and human-readable text cover CI and human use cases; Markdown report from existing `report` subcommand is the long-form format |
| **test-agent with built-in agent driver (Playwright, headless browser)** | Running a real browser inside the test adds enormous complexity; HoneyPrompt measures passive compliance, not active browsing behavior | test-agent receives callbacks; the agent under test is external; users point their own agent at the test server |
| **Kubernetes / Helm deployment** | Over-engineering for a research tool demo deployment; adds YAML complexity and operational surface | Single binary + systemd on a VPS, or a minimal Docker container, is entirely sufficient for honeyprompt.sh |
| **Signed binaries (code signing certificates)** | macOS Gatekeeper and Windows SmartScreen require paid certificates; cost and maintenance overhead not justified for v2.0 | Document `xattr -d com.apple.quarantine` for macOS; revisit when there is commercial user demand |
| **Automated benchmark suite in CI** | Premature for a tool that has not yet established real-world usage patterns | Run `cargo test` and `cargo clippy`; benchmarks come after profiling reveals bottlenecks |
| **Discord / Slack community infrastructure** | Community management is a full-time concern; too early | GitHub Discussions is sufficient; add community channels when there is organic demand |

---

## Feature Dependencies

```
test-agent subcommand
  ├── requires: existing serve + HTTP callback listener (already built)
  ├── requires: existing SQLite event store (already built)
  ├── requires: existing payload catalog (already built)
  └── emits: scorecard result (new)
       ├── --format text (default, human-readable)
       └── --format json (machine-readable, CI-friendly)
            └── exit codes (0 pass / 1 fail / 2 error)

GitHub Actions CI workflow
  └── requires: Cargo.toml with tests (already exists)
       └── cargo test + clippy + fmt on push/PR

GitHub Actions release workflow
  ├── requires: CI workflow (must pass before release)
  ├── requires: cargo-dist configuration in Cargo.toml
  └── triggered by: git tag v*
       └── builds: x86_64-linux-musl, x86_64-darwin, aarch64-darwin
            └── uploads: GitHub Release with binaries + checksums + installers

Live demo deployment
  ├── requires: binary release artifacts (from release workflow)
  ├── requires: existing serve + generate commands (already built)
  └── requires: domain + VPS or container platform
       └── deploys: honeyprompt.sh with live canary payloads

README rewrite
  ├── requires: test-agent subcommand (to document)
  ├── requires: binary release (install instructions need download URLs)
  └── requires: live demo (screenshot and URL to embed)
```

### Dependency Notes

- **test-agent requires serve (already built):** The compliance runner starts a local serve instance or connects to an existing one; it does not replace serve, it orchestrates it.
- **release workflow requires cargo-dist config:** cargo-dist writes a `[workspace.metadata.dist]` section to Cargo.toml on `cargo dist init`; this must happen before the release workflow is functional.
- **README rewrite should be last:** It depends on accurate documentation of all new features; writing it first leads to drift.
- **live demo is independent of test-agent:** Deployment to honeyprompt.sh can happen before test-agent is complete; the demo just runs `honeyprompt serve`.

---

## MVP Definition (v2.0)

### Launch With (v2.0)

Minimum viable for the "Ship & Learn" milestone.

- [ ] `honeyprompt test-agent` with `--listen`, `--timeout`, `--format` flags — the new user-visible feature
- [ ] Exit codes 0/1/2 documented and tested — required for CI/CD integration claims
- [ ] GitHub Actions CI workflow (`cargo test` + `cargo clippy` + `cargo fmt --check`) — signals active maintenance
- [ ] GitHub Actions release workflow with x86_64-linux-musl, x86_64-darwin, aarch64-darwin binaries — minimum viable distribution
- [ ] README rewrite with value prop, quick-start, install instructions, TUI screenshot — required for public launch
- [ ] Live deployment at honeyprompt.sh with canary payloads — the live demo is the best possible marketing

### Add After Validation (v2.x)

- [ ] Shell installer / PowerShell installer via cargo-dist — add when `cargo install` friction is measured to be a barrier
- [ ] `--format json` scorecard with stable schema — add when CI integration requests arrive
- [ ] aarch64-unknown-linux-musl binary — add when ARM Linux users report issues

### Future Consideration (v3+)

- [ ] DNS callback listener — significantly higher deployment complexity; worth it after HTTP callback pattern is validated
- [ ] Custom payload authoring — gated behind safety audit; not before core model is stable
- [ ] Alert integrations (email/Slack/webhook) — deferred; JSON export is a sufficient v2 bridge
- [ ] Package manager submissions (brew tap) — deferred until sustained adoption

---

## Feature Prioritization Matrix (v2.0 Features)

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| `test-agent` subcommand (core) | HIGH | MEDIUM | P1 |
| Exit codes 0/1/2 | HIGH | LOW | P1 |
| `--timeout` flag | HIGH | LOW | P1 |
| GitHub Actions CI workflow | MEDIUM | LOW | P1 |
| Binary release artifacts (3 platforms) | HIGH | MEDIUM | P1 |
| README rewrite | HIGH | LOW | P1 |
| Live demo deployment | HIGH | MEDIUM | P1 |
| `--format json` output | MEDIUM | LOW | P2 |
| Compliance scorecard table | MEDIUM | LOW | P2 |
| Shell installer (cargo-dist) | MEDIUM | LOW | P2 |
| aarch64-linux-musl binary | LOW | LOW | P3 |
| Brew tap / package manager | LOW | MEDIUM | P3 |

**Priority key:**
- P1: Must have for v2.0 launch
- P2: Should have, add when P1 is complete
- P3: Nice to have, future consideration

---

## Competitor / Reference Tool Analysis

| Feature | Canarytokens.org | Promptfoo | OpenSSF Scorecard | HoneyPrompt v2.0 |
|---------|-----------------|-----------|-------------------|------------------|
| CLI test runner | No (web UI only) | `promptfoo eval` | `scorecard` CLI | `honeyprompt test-agent` |
| Pass/fail exit codes | N/A | Exit 0/100/1 | Exit 0/1 | Exit 0/1/2 |
| JSON output | No | `--output results.json` | `--format json` | `--format json` |
| Timeout flag | N/A | Not exposed | Not applicable | `--timeout <seconds>` |
| Binary releases | N/A (SaaS) | npm package | Go binary on GitHub Releases | Rust binary via cargo-dist |
| Cross-platform | N/A | Node.js (cross-platform by runtime) | Linux/macOS/Windows | Linux/macOS |
| CI/CD integration | Webhook only | GitHub Action | GitHub Action | GitHub Actions workflow |
| AI-agent-specific compliance | No | No | No | Yes (graduated proof tiers) |

**Key insight:** No existing tool combines CLI-based agent compliance testing with graduated proof levels and a passive canary delivery model. Promptfoo is the closest for CLI pattern reference (exit codes, JSON output, CI integration) but tests LLM outputs, not browsing agent HTTP behavior.

---

## Implementation Notes for test-agent

Based on research into CLI security tool patterns (Snyk, Semgrep, Promptfoo, OpenSSF Scorecard):

**Subcommand interface:**
```
honeyprompt test-agent [OPTIONS]

OPTIONS:
  --listen <addr>      Start a local test server and wait for callbacks (default: 127.0.0.1:8080)
  --timeout <seconds>  Maximum wait time for callbacks (default: 60)
  --format <fmt>       Output format: text (default) or json
  --tier <1|2|3|all>   Test specific tier or all tiers (default: all)
  --project <path>     Path to existing honeyprompt project (default: current dir)
```

**Exit code contract:**
- Exit 0: All tested tiers received expected callbacks within timeout
- Exit 1: One or more tiers did not receive expected callbacks (compliance failure)
- Exit 2: Configuration error, server startup failure, or invalid arguments

**Scorecard output (text format):**
```
HoneyPrompt Compliance Test Results
=====================================
Tier 1 (Arbitrary Callback)     PASS  (callback received in 2.3s)
Tier 2 (Conditional Callback)   PASS  (correct branch selected in 4.1s)
Tier 3 (Computed Callback)      FAIL  (no callback received within 60s timeout)
-------------------------------------
Result: 2/3 tiers passed
Exit: 1 (compliance failure)
```

**Scorecard output (JSON format):**
```json
{
  "version": "2.0.0",
  "timestamp": "2026-03-29T12:00:00Z",
  "result": "fail",
  "tiers": [
    { "tier": 1, "name": "arbitrary_callback", "passed": true, "latency_ms": 2300 },
    { "tier": 2, "name": "conditional_callback", "passed": true, "latency_ms": 4100 },
    { "tier": 3, "name": "computed_callback", "passed": false, "latency_ms": null }
  ],
  "summary": { "passed": 2, "failed": 1, "total": 3 }
}
```

**MEDIUM confidence note:** No existing tool tests AI agent browsing compliance in this exact way (passive canary + tier-based scorecard). The patterns above are derived from analogous security CLI tools (Snyk, Semgrep, Promptfoo) and standard Unix conventions. Real-world usage may reveal the need for additional flags or output fields.

---

## Implementation Notes for Release Infrastructure

Based on research into Rust binary distribution patterns:

**Recommended approach: cargo-dist**
- cargo-dist (axodotdev, v0.26.0+) handles cross-compilation, artifact packaging, checksum generation, GitHub Release creation, and installer script generation
- `cargo dist init` writes configuration to Cargo.toml; `cargo dist generate` emits `.github/workflows/release.yml`
- Supports cross-compilation to Linux via cargo-zigbuild and macOS via osxcross as of v0.26.0 (December 2024)
- Generates shell installer (`install.sh`) and PowerShell installer automatically

**Minimum target matrix for v2.0:**
- `x86_64-unknown-linux-musl` — static binary, runs on any Linux without glibc dependency
- `x86_64-apple-darwin` — Intel Mac
- `aarch64-apple-darwin` — Apple Silicon Mac

**CI workflow (separate from release):**
- Triggers: push to main, pull request to main
- Steps: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`
- Runs on: ubuntu-latest (sufficient for CI; release workflow handles platform matrix)

**Deployment for honeyprompt.sh:**
- Option A (VPS + systemd): Upload x86_64-linux-musl binary, create systemd unit, `systemctl start honeyprompt`; zero container overhead; simplest ops model
- Option B (Docker + Fly.io or Render): Multi-stage Dockerfile (rust:alpine builder → scratch runtime); FROM scratch produces ~3MB image; Fly.io free tier supports this
- Recommendation: Option A (VPS + systemd) for simplicity; Option B if the team prefers managed infrastructure
- Both options are one-command-deployable once the binary is built

**HIGH confidence note:** The Rust + GitHub Actions + cargo-dist pattern is well-established and actively maintained. cargo-dist v0.26.0 added Linux cross-compilation support. The multi-stage Docker FROM scratch pattern is equally well-established.

---

## Sources

- Canarytokens.org documentation — reference implementation for canary token features
- [LLM Agent Honeypot: Monitoring AI Hacking Agents in the Wild (arXiv:2410.13919)](https://arxiv.org/html/2410.13919v2) — prompt injection + timing analysis
- [Promptfoo CLI documentation](https://www.promptfoo.dev/docs/usage/command-line/) — exit code patterns (0/100/1), JSON output, CI integration reference
- [Snyk CLI test command documentation](https://docs.snyk.io/snyk-cli/cli-commands-and-options-summary) — exit code conventions (0/1/2) for security CLI tools
- [Semgrep CLI reference](https://semgrep.dev/docs/cli-reference) — `--timeout` flag pattern, `--test` subcommand
- [OpenSSF Scorecard](https://scorecard.dev/) — scorecard output format reference (structured per-check pass/fail)
- [cargo-dist GitHub releases](https://github.com/axodotdev/cargo-dist/releases) — v0.26.0 Linux cross-compilation support confirmed
- [actions-rust-cross GitHub Action](https://github.com/houseabsolute/actions-rust-cross) — alternative to cargo-dist for cross-compilation
- [Rust cross-platform binary release (dzfrias.dev)](https://dzfrias.dev/blog/deploy-rust-cross-platform-github-actions/) — GitHub Actions matrix strategy for Rust releases
- [Rust Docker multi-stage builds (DEV Community)](https://dev.to/deciduously/use-multi-stage-docker-builds-for-statically-linked-rust-binaries-3jgd) — FROM scratch static binary deployment pattern
- [Deploying Rust web applications (Shuttle.dev)](https://www.shuttle.dev/blog/2024/02/07/deploy-rust-web) — VPS + systemd deployment pattern

---

*Feature research for: HoneyPrompt v2.0 — test-agent, release infrastructure, deployment, public launch*
*Researched: 2026-03-29*
