# Project Research Summary

**Project:** HoneyPrompt v2.0 — Ship & Learn
**Domain:** Rust CLI security tool — AI agent compliance testing, release infrastructure, live deployment
**Researched:** 2026-03-29
**Confidence:** HIGH (existing stack, architecture patterns, pitfall mitigations) / MEDIUM (test-agent harness specifics, launch strategy)

## Executive Summary

HoneyPrompt v2.0 is a well-scoped extension of a fully built v1.0 product. The existing Rust stack (Axum, Ratatui, rusqlite, tokio, minijinja, rust-embed, Clap) requires exactly one new explicit dependency: `tokio-util@0.7` for `CancellationToken`. Everything else — the `test-agent` subcommand, release infrastructure, and deployment — is achievable by composing what is already in the codebase or adding GitHub Actions workflow files. The core architecture insight is that `test-agent` is a pure orchestration layer that reuses `server::build_router`, `generator::generate`, `broker_task`, and `db_writer_task` directly, following the same pattern already proven in the existing `monitor` subcommand (monitor/mod.rs lines 854–895). No existing module requires modification.

The recommended delivery order follows a strict dependency chain: CI workflow first (validates a green baseline before adding new code), then `test-agent` subcommand (the primary user-visible v2 feature), then release workflow and cross-platform binaries (distribution requires working code), then live deployment and README (launch assets require a releasable binary). These four phases are independent in implementation but ordered by what must exist before the next phase can be validated.

The primary risks are technical and well-understood. Three pitfalls in `test-agent` cause silent failures in CI unless deliberately addressed at implementation time: the server-not-ready race (use an oneshot ready channel, not a sleep), the port leak after timeout (use `CancellationToken` + `handle.abort()`), and the missing exit code wiring (call `std::process::exit()` explicitly — `Ok(())` from main always exits 0). For distribution, the only non-obvious risk is routing `*-apple-darwin` targets to macOS runners: `rusqlite` with `bundled` requires macOS SDK headers that `cross` on Linux cannot supply. For the public launch, a missing ethics/scope statement in the README is the highest-probability friction point and must be written before any HN or Reddit post.

## Key Findings

### Recommended Stack

The v1.0 stack is locked and appropriate. For v2.0, only `tokio-util@0.7` needs to be made an explicit `Cargo.toml` dependency — it is currently an implicit transitive dep of `tokio`, and making it explicit stabilizes the `CancellationToken` API surface for the `test-agent` shutdown pattern.

GitHub Actions infrastructure uses two established, actively maintained action pairs: `dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2` for CI, and `houseabsolute/actions-rust-cross@v0` + `softprops/action-gh-release@v2` for cross-platform releases. The entire `actions-rs` organization was archived on 2023-10-13 and must not appear in any workflow file.

For containerized deployment, a two-stage Dockerfile (builder: `rust:slim` with musl target; runtime: `gcr.io/distroless/static-debian12`) produces images under 20 MB. Fly.io is the recommended host at ~$1.94/month for a persistent `shared-cpu-1x 256 MB` machine. Systemd on a plain VPS is equally valid and operationally simpler — both are viable.

**Core technologies (new or changed in v2.0):**
- `tokio-util@0.7`: CancellationToken for coordinated server shutdown in test-agent — make explicit in Cargo.toml
- `dtolnay/rust-toolchain@stable`: CI toolchain action replacing deprecated `actions-rs` — current standard
- `Swatinem/rust-cache@v2`: Cargo/target cache cutting CI cold-build from ~3 min to ~30 sec
- `houseabsolute/actions-rust-cross@v0`: Cross-compilation for Linux ARM targets using Docker; native cargo for macOS
- `softprops/action-gh-release@v2`: Canonical multi-file GitHub Release upload action
- `gcr.io/distroless/static-debian12`: Runtime Docker base — CA certs included, no shell, ~15 MB final image

### Expected Features

v2.0 adds four capability areas to the shipped v1.0 product. Research confirms all are table stakes for public launch.

**Must have (table stakes for v2.0 launch):**
- `honeyprompt test-agent` with `--listen`, `--timeout`, `--format` flags — the primary user-visible v2 feature
- Exit codes 0/1/2 (no-compliance/compliance-detected/error) documented, tested, and verified with `echo $?`
- `--timeout <seconds>` flag with 60-second default — bounded test runs are expected by all security researchers
- GitHub Actions CI workflow (`cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`) — absence reads as abandonware
- Cross-platform binary release (x86_64-linux-musl, x86_64-darwin, aarch64-darwin) triggered by `v*` tag — security researchers will not compile from source for evaluation
- README rewrite with value prop, quick-start under 5 commands, install instructions, TUI screenshot — the README is the product page
- Live demo at honeyprompt.sh — the single most effective launch artifact

**Should have (add after P1 complete):**
- `--format json` output with stable schema — needed when CI integration requests arrive; `--format text` is sufficient for day-one launch
- Compliance scorecard table with per-tier pass/fail and latency — richer signal than a single boolean; low implementation cost
- Shell installer via cargo-dist — lower friction for non-Rust users; add when `cargo install` friction is confirmed as a barrier

**Defer to v2.x / v3+:**
- DNS callback listener — higher deployment complexity; validate HTTP callback pattern first
- Package manager submissions (brew, apt) — too early; add after sustained adoption
- Built-in agent driver (Playwright, headless browser) — out of scope; test-agent receives callbacks, it does not drive agents
- Kubernetes/Helm deployment — over-engineering for a research tool live demo
- Signed binaries (code signing certs) — cost not justified at this stage; document `xattr -d com.apple.quarantine` workaround

### Architecture Approach

The `test-agent` subcommand is a pure orchestration layer: it composes existing modules in a new sequence (generate ephemeral output → spin up temporary Axum server → wait for callbacks with timeout → emit scorecard → exit with code) without modifying any of them. The new module `src/test_agent/mod.rs` owns only the orchestration logic, `TestAgentArgs`, scorecard rendering (text + JSON), and the timeout/early-exit coordination via `tokio::select!`. Three existing files change (cli/mod.rs adds the enum variant, main.rs adds the dispatch arm, lib.rs exposes the new module); all other modules are unchanged.

**Major components for v2.0:**

1. `src/test_agent/mod.rs` (NEW) — orchestrates ephemeral generate→serve→wait→score pipeline; owns TestAgentArgs, scorecard struct, timeout/early-exit coordination
2. `src/cli/mod.rs` (MODIFIED) — add `TestAgent(TestAgentArgs)` variant to `Commands` enum; add `OutputFormat` enum
3. `src/main.rs` (MODIFIED) — add `Commands::TestAgent` match arm; call `std::process::exit(exit_code)` explicitly
4. `.github/workflows/ci.yml` (NEW) — fmt + clippy + test on ubuntu-latest and macos-latest; triggers on push/PR to main
5. `.github/workflows/release.yml` (NEW) — cross-platform matrix build + GitHub Release; triggers on `v*` tag push

**Key patterns to follow:**
- Oneshot-gated server with CancellationToken: `axum::serve(...).with_graceful_shutdown(token.cancelled())` + `tokio::select!` on timeout vs early-exit channel
- Module reuse through public function boundaries: call existing `server::build_router`, `generator::generate`, `broker_task`, `db_writer_task` directly — mirrors existing `monitor` pattern
- Exit code as test oracle: `std::process::exit(n)` from `Commands::TestAgent` arm — 0 = no compliance detected, 1 = compliance detected, 2 = tool error

### Critical Pitfalls

1. **Server-not-ready race in test-agent** — `tokio::spawn(serve(...))` returns before `TcpListener::bind()` completes; agent fired immediately gets "Connection refused." Avoid: use a oneshot channel to send `SocketAddr` after bind, receive it before firing any requests. Never use `sleep()` as a substitute.

2. **Port leak after timeout** — `tokio::time::timeout()` cancels the wait future but not the spawned Axum task; subsequent `test-agent` invocations fail with "Address already in use." Avoid: always bind to port 0 (OS-assigned); use `CancellationToken` + `handle.abort()` to tear down the server task explicitly on timeout.

3. **Silent exit code failure** — `fn main() -> anyhow::Result<()>` returns `Ok(())` whether or not agents triggered callbacks; CI passes on compliance failures. Avoid: call `std::process::exit(code)` explicitly from `Commands::TestAgent`; verify with integration tests that assert `result.exit_code == 2` on zero-callback runs.

4. **rusqlite bundled C compile fails for Apple targets on Linux runners** — `cross` Docker images lack macOS SDK headers; `cc-rs` cannot compile SQLite C source for `*-apple-darwin` targets from a Linux host. Avoid: route all `*-apple-darwin` targets to `macos-latest` runners in the GitHub Actions matrix — never use `cross` for Mac targets.

5. **Launch without ethics/scope statement draws abuse reports** — HoneyPrompt injects instructions into agents; without prominent "authorized use only" language, platform operators and journalists interpret this as an attack tool. Avoid: add a "Scope and Ethics" section to the README before any public post.

## Implications for Roadmap

Based on research, the four v2.0 capability areas map naturally to four sequential phases. Each phase has a green validation checkpoint before the next begins.

### Phase 1: test-agent Subcommand

**Rationale:** The primary user-visible v2 feature; everything else (release, deployment, README) depends on having this working. The CI workflow (`ci.yml`) is built as a pre-step within this phase — it validates the green baseline before any new code is added, meaning the first commit to `test-agent` goes into a repo with a passing badge.

**Delivers:** `honeyprompt test-agent --listen --timeout --format` with exit codes 0/1/2, text scorecard, optional JSON output, and a green CI badge on the repository.

**Addresses:** test-agent (P1), exit codes (P1), `--timeout` flag (P1), compliance scorecard table (P2), `--format json` (P2).

**Avoids:**
- Server-not-ready race: use oneshot ready channel, not sleep
- Port leak: bind to port 0, use CancellationToken + handle.abort()
- Silent exit code: call std::process::exit() explicitly; verify with `echo $?`

**Build order within phase:**
1. Add `.github/workflows/ci.yml` (validates green baseline)
2. CLI extension: add `TestAgent(TestAgentArgs)` + stub main arm — `honeyprompt test-agent --help` works and compiles clean
3. Server orchestration: wire broker, server, store; implement timeout shutdown with CancellationToken
4. Scorecard: DB query post-shutdown, text + JSON rendering, exit codes verified

**Research flag:** STANDARD PATTERNS — module reuse via existing `monitor` pattern is proven in-codebase; Axum graceful shutdown is officially documented by Tokio.

### Phase 2: GitHub Actions Release Workflow

**Rationale:** Distribution requires working code (Phase 1 must be green). The release workflow triggers only on `v*` tags, so it is independent of Phase 1 feature commits. Binaries must exist before the live deployment can run the same artifact users download.

**Delivers:** Cross-platform binaries (x86_64-linux-musl, x86_64-darwin, aarch64-darwin, aarch64-linux) on GitHub Releases triggered by tag push; archives named `honeyprompt-{version}-{target}.tar.gz`.

**Addresses:** Binary release artifacts (P1), automated release pipeline.

**Avoids:**
- Do not use `actions-rs/*` — archived; use `dtolnay/rust-toolchain` + direct `cargo` steps
- Route `*-apple-darwin` targets to `macos-latest` runners exclusively — rusqlite bundled requires macOS SDK
- Cache key must include target triple: `${{ runner.os }}-${{ matrix.target }}-cargo` to prevent cross-target poisoning
- Verify binaries run on a fresh `ubuntu:22.04` container before Phase 4 launch announcement

**Research flag:** STANDARD PATTERNS — GitHub Actions matrix release for Rust is well-documented; specific action choices (`dtolnay`, `Swatinem`, `actions-rust-cross`, `softprops/action-gh-release`) are the current community standard with HIGH confidence.

### Phase 3: Live Demo Deployment

**Rationale:** Depends on a releasable binary (Phase 2) but is operationally independent of the test-agent feature. The live demo at honeyprompt.sh is the single highest-ROI launch asset — it should be running before the public announcement.

**Delivers:** `honeyprompt serve` running persistently at honeyprompt.sh with live canary payloads and HTTPS.

**Addresses:** Live demo deployment (P1).

**Deployment options (both valid):**
- Option A (recommended for simplicity): systemd service on a VPS with x86_64-linux-musl binary; no container overhead; one-command deploy after Phase 2 produces the binary
- Option B (recommended for reproducibility): two-stage Dockerfile → `distroless/static-debian12` runtime; Fly.io at ~$1.94/month with `auto_stop_machines = false`

**Avoids:**
- Verify `ldd honeyprompt` shows "statically linked" before building Docker image; if not, set `RUSTFLAGS="-C target-feature=+crt-static"` or switch to `distroless/static`
- Use multi-stage Dockerfile — final image must be under 30 MB, not the builder stage (~1 GB)
- Use `distroless/static-debian12` not bare `FROM scratch` — handles musl edge cases and includes CA certs
- Use a named Docker volume (not bind mount) for `.honeyprompt/` to avoid overlay2 WAL-mode issues
- Rotate nonces periodically on the live deployment — stale nonces become replay targets

**Research flag:** STANDARD PATTERNS — Rust musl + distroless Docker and Fly.io Rust are both officially documented with HIGH confidence.

### Phase 4: README Rewrite and Public Launch

**Rationale:** README must be written last — it documents features that exist (test-agent from Phase 1), install instructions that reference binaries that exist (Phase 2), and a live demo URL that works (Phase 3). Writing it first leads to drift.

**Delivers:** Polished README with value prop, quick-start (under 5 commands), install instructions with download links, TUI screenshot, and an ethics/scope section. Launch posts to HN, Reddit r/netsec, Twitter/X.

**Addresses:** README rewrite (P1), installation one-liner (P1), ethics/scope statement (critical for launch).

**Avoids:**
- Ethics/scope section is non-optional — "HoneyPrompt tests agents visiting pages YOU deploy on infrastructure YOU control"; absence draws immediate abuse reports within the first 24 hours
- Test `cargo install honeyprompt` works (crates.io publish) as a fallback for users without pre-built binaries — research did not cover the publish workflow; verify before Phase 4
- Test install instructions on a fresh `ubuntu:22.04` container before posting
- Verify `honeyprompt --version` returns the correct version in all release binaries

**Research flag:** NEEDS ATTENTION — exact ethics framing and launch messaging for the security research community is MEDIUM confidence; the scope statement should be reviewed before posting to HN/Reddit.

### Phase Ordering Rationale

- CI workflow is a Phase 1 pre-step (not its own phase) because it validates the baseline before new code is added — a green badge before the first `test-agent` commit prevents regressions from landing silently
- test-agent before release workflow because the release workflow should ship working features; a broken or stub `test-agent` should not be tagged v2.0
- Release workflow before deployment because the live demo should run the same binary users download — build once, ship consistently
- README last because it references features, binaries, and URLs that do not exist until Phases 1–3 complete

### Research Flags

Phases needing deeper research during planning:
- **Phase 4 (launch):** Ethics framing and exact messaging for the security research community is MEDIUM confidence; review with a security-community-experienced reviewer before posting to HN/Reddit

Phases with standard patterns (skip research-phase):
- **Phase 1 (test-agent):** Module reuse pattern is proven in the existing `monitor` codebase; Axum graceful shutdown patterns are officially documented by Tokio and Axum
- **Phase 2 (release):** Rust GitHub Actions release matrix is well-documented with HIGH-confidence action choices
- **Phase 3 (deployment):** Rust musl + distroless Docker and Fly.io Rust are both officially documented with HIGH confidence

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Existing stack locked; tokio-util is a transitive dep being made explicit; all new Actions choices verified against active, maintained projects |
| Features | HIGH (core) / MEDIUM (test-agent UX specifics) | Table stakes features for a v2.0 CLI security tool are well-established via Snyk/Semgrep/Promptfoo reference. AI-agent compliance testing interface is novel; real-world usage may reveal need for additional flags or output fields |
| Architecture | HIGH | test-agent pattern is directly demonstrated by existing `monitor` subcommand in the same codebase; Axum + oneshot shutdown is officially documented |
| Pitfalls | HIGH (cross-compile, Docker) / MEDIUM (test-agent harness) | Cross-compile issues sourced from rusqlite issue tracker (direct reports from #914, #1615); Docker pitfalls from official min-sized-rust reference; test-agent harness pitfalls derived from Axum testing discussions |

**Overall confidence:** HIGH for implementation decisions; MEDIUM for launch strategy and test-agent UX assumptions.

### Gaps to Address

- **crates.io publish workflow:** `cargo install honeyprompt` as an install path requires publishing to crates.io. Research did not cover the publish workflow (crate name availability, metadata requirements). Verify during Phase 4 planning before writing install instructions.

- **Default timeout for real-world agents:** Research recommends 60s default. Real-world agents may take 10–30s to process a page and fire callbacks. Validate this assumption against actual agent behavior post-launch and adjust default before v2.1 if needed.

- **JSON schema stability for `--format json`:** Defer marking the JSON output schema as "stable" until after the first real-world CI integration request. Declaring it stable in v2.0 means supporting it indefinitely — ship it without a stability guarantee initially.

- **Live demo nonce rotation:** No automated nonce rotation strategy was designed. The honeyprompt.sh live deployment will need a cron job or periodic manual `generate` + restart to keep nonces fresh. Address in Phase 3 planning before the live demo goes up.

- **Docker overlay2 + SQLite WAL:** Pitfalls research identifies a known issue with SQLite WAL mode and Docker `overlay2` storage driver when using bind mounts. Address with a named volume in the Docker deployment configuration.

## Sources

### Primary (HIGH confidence)
- Existing `src/monitor/mod.rs` lines 823–960 — oneshot shutdown pattern proven in this codebase
- Existing `src/server/mod.rs` `build_router` pub API
- [Tokio graceful shutdown documentation](https://tokio.rs/tokio/topics/shutdown) — shutdown coordination patterns
- [tokio-util CancellationToken docs](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release) — canonical release action
- [Fly.io Rust documentation](https://fly.io/docs/rust/) — Dockerfile auto-detection, pricing (~$1.94/month)
- [std::process::ExitCode](https://doc.rust-lang.org/beta/std/process/struct.ExitCode.html) — stdlib exit code API
- [rusqlite cross-compile issue #914](https://github.com/rusqlite/rusqlite/issues/914) and [#1615](https://github.com/rusqlite/rusqlite/issues/1615) — aarch64-apple-darwin C compile failure confirmed
- [min-sized-rust reference](https://github.com/johnthagen/min-sized-rust) — Docker image optimization patterns

### Secondary (MEDIUM confidence)
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) — replacement for archived actions-rs; widely recommended
- [Swatinem/rust-cache@v2](https://github.com/Swatinem/rust-cache) — current community standard for Rust CI caching
- [houseabsolute/actions-rust-cross@v0](https://github.com/houseabsolute/actions-rust-cross) — active cross-compilation action on GitHub Marketplace
- [Promptfoo CLI documentation](https://www.promptfoo.dev/docs/usage/command-line/) — exit code patterns (0/100/1) and JSON output as reference for test-agent interface
- [Snyk CLI documentation](https://docs.snyk.io/snyk-cli/cli-commands-and-options-summary) — exit code conventions for security CLI tools
- [Cross-platform Rust CI/CD pipeline (December 2025)](https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/) — recent workflow patterns
- [Minimal Docker Images for Rust Binaries (January 2026)](https://oneuptime.com/blog/post/2026-01-07-rust-minimal-docker-images/view) — distroless vs scratch trade-offs

### Tertiary (informational)
- [cargo-dist introduction](https://axodotdev.github.io/cargo-dist/book/introduction.html) — considered and deferred; relevant when Homebrew tap or shell installer is needed
- [Cross Compiling Rust Projects in GitHub Actions (houseabsolute blog)](https://blog.urth.org/2023/03/05/cross-compiling-rust-projects-in-github-actions/) — design rationale for actions-rust-cross
- [LLM Agent Honeypot: Monitoring AI Hacking Agents in the Wild (arXiv:2410.13919)](https://arxiv.org/html/2410.13919v2) — prompt injection + timing analysis background

---
*Research completed: 2026-03-29*
*Ready for roadmap: yes*
