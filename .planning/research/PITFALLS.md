# Pitfalls Research

**Domain:** AI agent honeypot / canary token security tool — v2 Ship & Learn features
**Project:** HoneyPrompt
**Researched:** 2026-03-29
**Confidence:** HIGH (cross-compilation, Docker) / MEDIUM (test-agent harness, launch)

---

> This file covers **v2 milestone pitfalls** for adding test-agent, CI/CD, Docker, and public
> launch to the existing v1 system. Domain/product pitfalls from the v1 milestone (payload
> embedding, nonce replay, SSRF, robots.txt, etc.) are preserved at the end of this file.

---

## Critical Pitfalls

### Pitfall 1: test-agent Spawns Its Own Server Without Waiting for Bind

**What goes wrong:**
`test-agent` starts an embedded Axum server in a background task, immediately fires the agent
under test at the listen address, and gets `Connection refused` because the TCP listener has not
finished `bind()` yet. The command exits with a failure that looks like an agent compliance
failure rather than a startup race.

**Why it happens:**
`tokio::spawn(serve(...))` returns before `TcpListener::bind()` completes. The interval between
spawning the task and the socket actually accepting connections is typically 1–50 ms but is
non-deterministic. Developers add `tokio::time::sleep(Duration::from_millis(100))` as a bandaid,
which is flaky in CI.

**How to avoid:**
Use a one-shot channel to signal readiness. The serve task sends on the channel after
`axum::serve(listener, ...)` has been called but before `.await`. The test-agent orchestrator
`recv()`s the ready signal before firing the agent. This is the pattern used in the existing
`test_serve.rs` tests (they avoid the problem entirely by using `tower::ServiceExt::oneshot`
without a real listener — test-agent cannot use that approach because it needs a real network
port for the target agent to reach).

Concretely:
```rust
let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<SocketAddr>();
tokio::spawn(async move {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    ready_tx.send(addr).unwrap();
    axum::serve(listener, app).await.unwrap();
});
let bound_addr = ready_rx.await.expect("server failed to start");
```

**Warning signs:**
- test-agent fails immediately with a connection error on first run, passes on second
- CI fails intermittently with "Connection refused" but local runs pass
- Developer has added a `sleep(200ms)` before the first agent request

**Phase to address:** test-agent subcommand implementation (Phase 1 of v2)

---

### Pitfall 2: test-agent Timeout Does Not Cancel the Embedded Server

**What goes wrong:**
`test-agent --timeout 30` runs for 30 seconds waiting for a callback. When the timeout fires,
the Axum server task is still running and holding the TCP port open. On the next invocation of
`test-agent`, `bind()` fails with `Address already in use`.

**Why it happens:**
`tokio::time::timeout(duration, wait_for_callback()).await` cancels the future waiting for the
callback channel, but does NOT cancel the spawned Axum task. The task is detached; it lives
until the process exits. In tests that run multiple `test-agent` invocations (or integration
tests that call the function directly), the leftover listener blocks the next bind.

**How to avoid:**
Hold a `JoinHandle` for the server task and call `handle.abort()` explicitly when the timeout
fires. Use a `CancellationToken` (from `tokio-util`) so the server task self-terminates when
cancelled:
```rust
let cancel = CancellationToken::new();
let server_cancel = cancel.clone();
let handle = tokio::spawn(async move {
    tokio::select! {
        _ = axum::serve(listener, app) => {}
        _ = server_cancel.cancelled() => {}
    }
});
// On timeout or completion:
cancel.cancel();
handle.await.ok();
```
Always bind to port 0 (OS-assigned) in test-agent's `--listen` default so port conflicts are
impossible across parallel invocations.

**Warning signs:**
- "Address already in use" on second run of `honeyprompt test-agent`
- Integration tests that call test-agent logic fail when run with `--test-threads > 1`
- `lsof -i :PORT` shows honeyprompt holding the port after the command exits

**Phase to address:** test-agent subcommand implementation (Phase 1 of v2)

---

### Pitfall 3: test-agent Exit Code Is Not Wired to CI — Silent Failures

**What goes wrong:**
`test-agent` returns `Ok(())` from main regardless of whether the agent triggered any callbacks.
CI passes. The compliance scorecard is printed to stdout but nobody notices the agent scored 0/3
tiers because the exit code was 0.

**Why it happens:**
Rust `fn main() -> anyhow::Result<()>` returns success unless it returns `Err(...)`. The
test-agent naturally returns `Ok(())` after printing results, whether the agent passed or failed.
Exit codes for CLI tools are easy to forget.

**How to avoid:**
Define explicit exit codes and use `std::process::exit()`:
- `0` — all expected tier callbacks received within timeout
- `1` — partial compliance (some but not all tiers received)
- `2` — no callbacks received (zero compliance)
- `3` — server or network error (tool fault, not agent fault)

Document these exit codes in the `--help` output and README. Test the exit codes explicitly in
integration tests:
```rust
assert_eq!(result.exit_code, 2, "no-callback run must exit 2");
```

**Warning signs:**
- `echo $?` after a failed run prints `0`
- CI pipeline shows test-agent step as green even when agent scored 0
- `--format json` output shows `"tier_hits": 0` but pipeline passed

**Phase to address:** test-agent subcommand implementation (Phase 1 of v2)

---

### Pitfall 4: Cross-Compiling rusqlite (bundled) for aarch64-apple-darwin Fails Without macOS SDK

**What goes wrong:**
`rusqlite` with `features = ["bundled"]` compiles SQLite from C source using the host's C
compiler. When cross-compiling from a Linux x86_64 runner to `aarch64-apple-darwin`, the C
compilation step fails because the macOS SDK headers (`TargetConditionals.h`, etc.) are not
present on the Linux runner. The error is a C compiler error, not a Rust error, which is
confusing.

**Why it happens:**
The `bundled` feature vendors SQLite C source and compiles it via `cc-rs`. `cc-rs` uses the
system C compiler, which on Linux has no macOS sysroot. Apple's SDK is not freely redistributable,
so `cross` (which uses Docker images) cannot include it.

**How to avoid:**
Use a GitHub Actions `macos-latest` runner for the `aarch64-apple-darwin` target. Apple Silicon
macOS runners (`macos-14` or `macos-latest` as of 2024) natively support building for both
`x86_64-apple-darwin` and `aarch64-apple-darwin` without cross-compilation. The workflow matrix
should route mac targets to mac runners, not Linux runners:

```yaml
matrix:
  include:
    - target: x86_64-unknown-linux-musl
      os: ubuntu-latest
      use_cross: true
    - target: aarch64-unknown-linux-musl
      os: ubuntu-latest
      use_cross: true
    - target: x86_64-apple-darwin
      os: macos-latest
      use_cross: false
    - target: aarch64-apple-darwin
      os: macos-latest
      use_cross: false
```

Native mac runners are the only reliable approach for mac targets when C dependencies are
involved. osxcross workarounds exist but require Apple SDK files with legally dubious
redistribution status.

**Warning signs:**
- `error: failed to run custom build command for 'libsqlite3-sys'`
- C compiler errors mentioning `TargetConditionals.h` in a Linux CI log
- `cross` Docker image build succeeding but the binary failing at link time

**Phase to address:** GitHub Actions release workflow (Phase 2 of v2)

---

### Pitfall 5: Using actions-rs/toolchain and actions-rs/cargo (Archived, Unmaintained)

**What goes wrong:**
A workflow that uses `actions-rs/toolchain@v1`, `actions-rs/cargo@v1`, or
`actions-rs/clippy-check@v1` will begin emitting deprecation warnings and eventually fail. The
entire `actions-rs` GitHub organization was archived on 2023-10-13. All actions use the
deprecated `node12` runtime and the deprecated `set-output` workflow command.

**Why it happens:**
Many tutorials and Stack Overflow answers from 2020–2022 use `actions-rs`. Developers copy-paste
without checking the maintenance status.

**How to avoid:**
Use the maintained alternatives:
- `dtolnay/rust-toolchain@stable` replaces `actions-rs/toolchain`
- `houseabsolute/actions-rust-cross@v0` replaces `actions-rs/cargo` for cross builds
- Run `cargo clippy`, `cargo fmt --check`, `cargo test` directly as `run:` steps; no action wrapper needed
- `actions-rust-lang/audit@v1` replaces `actions-rs/audit-check`

Pin action versions to a specific SHA (not a floating tag like `@v1`) for security and
reproducibility.

**Warning signs:**
- `Node.js 12 actions are deprecated` in CI logs
- `::set-output` deprecation warning in CI logs
- Any step using `actions-rs/` prefix

**Phase to address:** GitHub Actions CI and release workflow setup (Phase 2 of v2)

---

### Pitfall 6: Docker FROM scratch Fails at Runtime — libdl and libgcc Missing

**What goes wrong:**
A `FROM scratch` Docker image with the Rust binary compiled for `x86_64-unknown-linux-musl`
crashes at runtime with:
```
error while loading shared libraries: libdl.so.2: cannot open shared object file
```
or the container exits silently with code 127 (dynamic linker not found).

**Why it happens:**
Even with `--target x86_64-unknown-linux-musl`, Tokio (via `mio`) and Rusqlite (via `libsqlite3-sys`
with `bundled`) can still pull in dynamic dependencies depending on compiler flags. `libdl` is
required by `dlopen()` calls in some async runtimes. A `FROM scratch` image has literally nothing
— no shell, no libc, no dynamic linker (`/lib/ld-musl-x86_64.so.1`).

**How to avoid:**
Verify the binary is fully static before building the Docker image:
```bash
ldd target/x86_64-unknown-linux-musl/release/honeyprompt
# Must output: "not a dynamic executable" or "statically linked"
```
If not fully static, either:
1. Set `RUSTFLAGS="-C target-feature=+crt-static"` during compilation
2. Switch from `FROM scratch` to `FROM gcr.io/distroless/static-debian12` (contains musl libc,
   `/etc/ssl/certs`, and `ca-certificates` but no shell — small, secure, and correct)
3. Switch to `FROM alpine:3` if debugging access is needed during development

For the honeyprompt live demo deployment, `distroless/static` is the right choice: smaller
than `alpine`, no shell attack surface, and handles edge cases in musl static linking.

**Warning signs:**
- Container exits immediately with code 127 or 1
- `docker logs` shows no output at all (dynamic linker crash before any Rust code runs)
- `ldd` on the binary shows any line other than "statically linked"

**Phase to address:** Docker deployment configuration (Phase 3 of v2)

---

### Pitfall 7: Docker Image Bloat — rusqlite Bundled + Build Stage Not Discarded

**What goes wrong:**
A naive Dockerfile that compiles the Rust binary and then copies the entire `target/` directory
into the final image produces a 2–4 GB image. Even a "release build only" single-stage Dockerfile
produces a ~600 MB image because the `rust:latest` base image includes the full Rust toolchain.

**Why it happens:**
`rusqlite` with `features = ["bundled"]` compiles SQLite from C source, which requires a C
compiler at build time. Developers either (a) forget to use multi-stage builds, or (b) use
multi-stage but copy the wrong artifact (e.g., copy `target/debug/` instead of
`target/release/`).

**How to avoid:**
Use a two-stage Dockerfile:
```dockerfile
FROM rust:1.87-slim AS builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl
STRIP target/x86_64-unknown-linux-musl/release/honeyprompt

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/honeyprompt /usr/local/bin/honeyprompt
ENTRYPOINT ["/usr/local/bin/honeyprompt"]
```
Strip the binary (`strip` or `cargo-strip`) before the copy. Final image target: under 20 MB.
The `bundled` SQLite feature adds ~1.2 MB to the binary; acceptable.

**Warning signs:**
- `docker images honeyprompt` shows size > 100 MB
- `docker history honeyprompt` shows a `rust:latest` layer
- CI takes > 10 minutes to push the image

**Phase to address:** Docker deployment configuration (Phase 3 of v2)

---

### Pitfall 8: Public Launch Without Install Verification on a Fresh Machine

**What goes wrong:**
The README install instructions reference a GitHub release binary that works on the developer's
machine but fails on a fresh Ubuntu 22.04 because the binary was compiled with GNU libc (glibc)
and the target machine has a different glibc version, or the binary was compiled for
`x86_64-unknown-linux-gnu` and the user is on `aarch64`.

This is the #1 cause of "your tool doesn't work" issue reports in the first 48 hours of a launch.

**Why it happens:**
- The developer compiles on their machine, uploads the binary, and tests the download on the
  same machine (or a similar one). Fresh-machine testing is skipped.
- glibc version mismatch: `GLIBC_2.38 not found` on older LTS distributions.
- musl-compiled binaries are portable but developers forget to label them clearly
  (`honeyprompt-linux-x86_64-musl` vs `honeyprompt-linux-x86_64`).

**How to avoid:**
- Use `x86_64-unknown-linux-musl` (static, portable) for all Linux release binaries.
  Never release a dynamically linked glibc binary unless explicitly labeled.
- Name release artifacts with full target triple: `honeyprompt-x86_64-unknown-linux-musl`,
  `honeyprompt-aarch64-apple-darwin`, etc. — not just `honeyprompt-linux` or `honeyprompt-mac`.
- Before launch: run the downloaded binary in a Docker container from `ubuntu:22.04` (clean
  environment) and from `ubuntu:20.04` (older glibc baseline).
- Add a `cargo install honeyprompt` path via crates.io as a fallback for users who have Rust
  installed. This sidesteps binary compatibility entirely.

**Warning signs:**
- First GitHub issue after launch is "GLIBC_2.XX not found"
- README says "download from releases" but has no note about which binary to pick
- `file honeyprompt-linux` shows `dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2`

**Phase to address:** README/launch preparation (Phase 4 of v2)

---

### Pitfall 9: Security Tool Launch Without Clear Scope Statement Draws Abuse Reports

**What goes wrong:**
HoneyPrompt generates honeypot pages that inject instructions into AI agents visiting the page.
Without explicit, prominent scope documentation, two problems emerge:

1. AI platform operators (OpenAI, Anthropic, Google) receive abuse reports claiming HoneyPrompt
   "attacks" their agents, causing friction before evidence is even collected.
2. Researchers use the tool against agents they do not control or have no authorization to test,
   exposing themselves to potential ToS violations.

**Why it happens:**
Security tools that interact with third-party systems are always dual-use. The line between
"authorized security research" and "unauthorized testing" depends on scope that is not visible
in the tool itself. Journalists and defenders who encounter the tool's output — a page with
visible warning banners — may still interpret the callback mechanism as an attack tool.

**How to avoid:**
- The README must have a prominent "Scope and Ethics" section that explicitly states:
  - HoneyPrompt tests agents that visit pages YOU deploy on infrastructure YOU control
  - It does not actively probe third-party systems; agents must visit the honeypot voluntarily
  - All generated content includes visible human-readable warnings (already enforced)
  - Payloads contain no harmful instructions; all payloads are auditable and open source
- Add a `--ack-ethics` flag or a first-run disclaimer that is required before the server starts
  (good UX and legal CYA)
- If deploying the live demo at honeyprompt.sh: include a public disclosure notice explaining
  that the site is a research honeypot and visitors are consenting to being studied

**Warning signs:**
- First 24h after HN launch includes "is this legal?" top comment
- README has no ethics or scope section
- No mention of who has authorization to use the tool against what

**Phase to address:** README/launch preparation (Phase 4 of v2)

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `sleep(100ms)` in test-agent to wait for server ready | Simple, works locally | Flaky in CI (slow runners, resource contention) | Never — use oneshot channel instead |
| Hardcode `127.0.0.1:8080` as test-agent listen address | No flag to implement | Port conflict on second run or parallel tests | Never — bind to port 0 |
| Single `cargo build --release` in CI for all platforms on one runner | Simple workflow | Fails for mac targets (no SDK), slow serial builds | Never for multi-platform releases |
| `actions-rs/toolchain` copy-pasted from old tutorial | Works today | Node12 deprecation warning, eventual failure | Never on new projects |
| `FROM rust:latest` final Docker image | No multi-stage complexity | 600 MB+ image, Rust toolchain exposed in prod | Never — use multi-stage |
| `x86_64-unknown-linux-gnu` release binary | No musl setup required | GLIBC version mismatch on old LTS | Only if clearly labeled; prefer musl |
| Floating action version `@v1` | Simple | Supply chain attack vector, silent breakage on new release | Acceptable in dev; pin SHA for releases |

---

## Integration Gotchas

Common mistakes when connecting these new features to the existing system.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| test-agent + existing `serve()` | Calling `serve()` directly in test-agent — it blocks forever | Extract `build_router()` + `TcpListener::bind()` into a testable `serve_ephemeral()` function; `serve()` stays as the long-running entrypoint |
| test-agent + existing event pipeline | Bypassing the mpsc channel and polling SQLite directly for callback detection | Subscribe to the broadcast channel (`event_tx.subscribe()`) before firing the agent; channel delivery is faster than polling and consistent with the existing architecture |
| test-agent + `--timeout` flag | Using wall-clock `sleep` to simulate timeout | Use `tokio::time::timeout()` wrapping the channel `recv()` — testable, cancellable, and correct |
| GitHub Actions + rusqlite bundled | Running `cross` for Apple targets | Route `*-apple-darwin` targets to `macos-*` runners; only Linux/Windows need `cross` |
| GitHub Actions + Cargo cache | Using `actions/cache` with `key: ${{ runner.os }}-cargo` for a matrix of targets | Cache key must include target triple: `${{ runner.os }}-${{ matrix.target }}-cargo`; shared cache across targets causes cache poisoning |
| Docker + SQLite WAL | Bind-mounting a SQLite WAL database into a container using `overlay2` storage | Use a named volume, not bind mount, for the `.honeyprompt/` directory in Docker; overlay2 has known WAL-mode issues |
| Docker + `serve` command | Passing project path as `/app` but not mounting it | The entrypoint must mount the project directory: `docker run -v $(pwd):/data honeyprompt serve /data` |

---

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| test-agent polling SQLite for callbacks every 100ms | DB lock contention if serve and test-agent share one connection | Use broadcast channel subscription, not DB polling | Immediately if connection is shared |
| CI matrix builds on a single sequential workflow | Release takes 45+ minutes | Use `strategy: matrix` with `fail-fast: false` to parallelize across runners | When matrix has > 3 targets |
| Docker image rebuilt from scratch on every commit | Slow CI feedback | Cache the builder stage layers using `--cache-from` / `--cache-to` | When Cargo.lock hasn't changed but full rebuild is triggered |

---

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Shipping release binary built from uncommitted changes | Binary doesn't match source; security auditors cannot reproduce | CI release workflow must checkout tagged commit; never build from local workdir |
| Docker image runs as root | Container escape risk; attacker gets root on host | Add `USER nonroot` and run as a dedicated non-root user in the Dockerfile |
| GitHub Actions workflow reads secrets in a matrix job triggered by fork PRs | Secret exfiltration via malicious fork PR | Never put secrets in matrix jobs triggered by `pull_request`; use `pull_request_target` carefully or restrict secret access to `push` and `release` triggers |
| honeyprompt.sh deployment reuses the same nonce set indefinitely | Nonces scraped from source become stale replay payloads | Rotate nonces on each `generate` run; live deployment should re-generate periodically |

---

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| test-agent output format differs between `--format text` and `--format json` | Scripts that parse the output break | Define a stable JSON schema for `--format json`; text format is a rendered view of the same struct |
| No indication of which tier callbacks were received before timeout | User cannot tell if partial compliance happened | Print a running tally as callbacks arrive; don't wait until timeout to print results |
| `--timeout` default too short for real-world agents | Agent hasn't finished processing before test-agent exits | Default to 60 seconds; document that real agents may take 10–30s to process a page and fire callbacks |
| Release binary named `honeyprompt` on all platforms | User downloads wrong binary | Name artifacts with full target triple in the release |
| No `--version` in the release binary | User cannot confirm they're running the expected version | Ensure `clap` version derives from `Cargo.toml`; verify `honeyprompt --version` works in CI before upload |

---

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **test-agent exit codes:** Binary prints scorecard — verify `echo $?` returns `2` on zero-callback run, not `0`
- [ ] **test-agent server teardown:** Server starts, test runs — verify port is released after command exits (run twice consecutively)
- [ ] **CI release triggers correctly:** Workflow exists — verify it triggers on `v*` tags, not on every push
- [ ] **Mac binaries run on mac:** Binary uploads to release — verify `aarch64-apple-darwin` binary actually runs on Apple Silicon (not just builds)
- [ ] **Docker binary is static:** Dockerfile produces image — verify `ldd` inside container shows "statically linked"
- [ ] **Docker image size:** Image builds — verify final image is under 30 MB, not the builder stage (~1 GB)
- [ ] **README install path works:** Install instructions written — verify on a fresh `ubuntu:22.04` container with no pre-installed tools
- [ ] **Cargo.lock committed:** Release builds consistently — verify `Cargo.lock` is committed and CI uses it (`--locked` flag)

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Server-not-ready race in test-agent | LOW | Add oneshot ready channel; no schema or data changes needed |
| Port leak after timeout | LOW | Add `CancellationToken` + `handle.abort()`; no user-visible behavior change |
| actions-rs deprecation breaks CI | LOW | Replace with `dtolnay/rust-toolchain` + direct `cargo` invocations; 30-minute fix |
| Mac cross-compile failure | MEDIUM | Move mac targets to `macos-*` runners; requires workflow restructure but no code changes |
| `FROM scratch` runtime crash | LOW | Switch base image to `distroless/static`; one-line Dockerfile change |
| Image size bloat | LOW | Add multi-stage build; existing binary unchanged |
| glibc version error on launch | MEDIUM | Re-build with musl target; upload corrected binary; update README; acknowledge in comments |
| Missing exit codes discovered post-launch | LOW | Add `std::process::exit()` calls; patch release; no breaking change to users who don't use exit codes |

---

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Server-not-ready race | Phase 1: test-agent subcommand | Run `honeyprompt test-agent` twice consecutively; no connection error |
| Port leak after timeout | Phase 1: test-agent subcommand | Run `honeyprompt test-agent --timeout 5` with no agent; verify port released via `lsof` |
| Exit code not wired | Phase 1: test-agent subcommand | `honeyprompt test-agent && echo "WRONG"` — should not print "WRONG" on zero callbacks |
| rusqlite C compile on Linux for macOS | Phase 2: release workflow | Check CI matrix routes `*-apple-darwin` to `macos-*` runners |
| actions-rs unmaintained | Phase 2: release workflow | No `actions-rs/` prefix in any workflow file |
| `FROM scratch` runtime crash | Phase 3: Docker deployment | `docker run honeyprompt --version` succeeds |
| Docker image bloat | Phase 3: Docker deployment | `docker images honeyprompt` shows < 30 MB |
| Glibc portability on launch | Phase 4: README + launch | Test download on `ubuntu:22.04` clean container |
| Missing ethics/scope statement | Phase 4: README + launch | README has "Scope and Ethics" section before HN post |

---

## Sources

**Cross-compilation and GitHub Actions:**
- [Cross Compilation in Rust (2025)](https://fpira.com/blog/2025/01/cross-compilation-in-rust)
- [Cross Compiling Rust Projects in GitHub Actions](https://blog.urth.org/2023/03/05/cross-compiling-rust-projects-in-github-actions/)
- [houseabsolute/actions-rust-cross](https://github.com/houseabsolute/actions-rust-cross)
- [actions-rs archived — Issue #216](https://github.com/actions-rs/toolchain/issues/216)
- [rusqlite cross-compile to musl — Issue #914](https://github.com/rusqlite/rusqlite/issues/914)
- [rusqlite aarch64-apple-darwin failure — Issue #1615](https://github.com/rusqlite/rusqlite/issues/1615)
- [Rust Cross-Compilation with GitHub Actions](https://reemus.dev/tldr/rust-cross-compilation-github-actions)

**Docker:**
- [Rust Docker Image Optimization with Multi-Stage Builds](https://dev.to/mattdark/rust-docker-image-optimization-with-multi-stage-builds-4b6c)
- [Building Standalone Rust Binary for Scratch Container](https://bxbranden.github.io/)
- [Missing libdl.so and distroless discussion](https://github.com/dotnet/dotnet-docker/discussions/4938)
- [min-sized-rust reference](https://github.com/johnthagen/min-sized-rust)

**Axum testing patterns:**
- [Axum testing discussion #1701 — wait until server started](https://github.com/tokio-rs/axum/discussions/1701)
- [Axum testing discussion #555 — how to write tests](https://github.com/tokio-rs/axum/discussions/555)
- [Axum testing examples](https://github.com/tokio-rs/axum/blob/main/examples/testing/src/main.rs)

---

---

## v1 Domain Pitfalls (Preserved from v1.0 Research)

*The following pitfalls cover the core product domain — payload design, nonce security, SSRF,
and evidence quality. They remain relevant for any modifications to the serve, generate, or
report subsystems during v2.*

---

### v1 Pitfall 1: Payload Embedding Locations Agents Actually Ignore

**What goes wrong:** Payloads embedded only in HTML comments or `display:none` CSS elements produce no callbacks — not because agents are evasion-aware, but because many modern AI browsing agents render-filter their DOM view before the LLM sees the page.

**Prevention:** Distribute payloads across ALL embedding locations. Include at least one semantically embedded visible-prose payload per page.

**Phase:** Payload catalog design (Phase 1/2 of v1).

---

### v1 Pitfall 2: Nonce/Callback Integrity — No Replay Prevention

**What goes wrong:** A statically generated nonce can be replayed by link-checkers, human inspectors, or the agent itself. Replay events look like independent detections.

**Prevention:** Per-visitor nonce injection, replay detection flag in schema (`is_replay`, `fire_count` columns — already implemented), replay events excluded from detection counts.

**Phase:** Event store schema (Phase 1 of v1). Already addressed in the existing schema.

---

### v1 Pitfall 3: Over-Trusting User-Agent and IP for Agent Attribution

**What goes wrong:** Attributing "AI agent" status based on UA/IP alone produces false positives (GPTBot indexing) and false negatives (agents spoofing Chrome UA).

**Prevention:** UA/IP is metadata, not classification basis. Proof level (behavioral signal) is the primary classification criterion.

**Phase:** Fingerprinting module (Phase 1/2 of v1). Already addressed in the existing crawler catalog.

---

### v1 Pitfall 4: Callback Listener Accepts Arbitrary Payloads — SSRF and Log Injection Risk

**What goes wrong:** An open callback endpoint that reflects request data into the DB or TUI is vulnerable to SQL injection, SSRF, and terminal injection.

**Prevention:** Parameterized queries (already enforced with `params![]`), strict schema validation, 204 unconditionally, no body extractors (SRV-07 already enforced).

**Phase:** HTTP server and event store (Phase 1 of v1). Already addressed in current implementation.

---

### v1 Pitfall 5: robots.txt Compliance Treated as a Detection Signal

**What goes wrong:** robots.txt violations are treated as proof of agentic behavior when they are actually normal for both legitimate crawlers and AI agents.

**Prevention:** robots.txt = friction only. Not a proof level. Report shows it as metadata.

**Phase:** Site generator and report generator (Phase 2/3 of v1).

---

*For the full v1 pitfall research including Pitfalls 6–12 (static page indexing, timing fragility,
tier inflation, legitimate scanners, SQLite WAL on exotic filesystems, TUI blocking reads,
Markdown injection), see git history for this file prior to 2026-03-29.*

---

*Pitfalls research for: HoneyPrompt v2 Ship & Learn — test-agent, CI/CD, Docker, public launch*
*Researched: 2026-03-29*
