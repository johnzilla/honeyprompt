# Stack Research

**Domain:** Rust CLI security tool — new capabilities for v2.0
**Researched:** 2026-03-29
**Confidence:** HIGH (existing stack), MEDIUM (GitHub Actions patterns), HIGH (Fly.io/Docker)

## Scope

This file covers ONLY the stack additions for v2.0. The existing locked stack (Clap, Axum, Ratatui, rusqlite, tokio-rusqlite, rust-embed, minijinja, serde, tokio) is NOT re-examined here.

---

## New Stack Additions

### test-agent Subcommand

The `test-agent` subcommand needs to: spin up a temporary Axum server, drive an agent (or wait for agent traffic), wait for callbacks within a timeout, then produce a pass/fail scorecard and exit with a meaningful exit code.

No new HTTP or server crates are needed — Axum 0.8 is already in-tree. The missing piece is coordinated shutdown with timeout.

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tokio-util | 0.7 | `CancellationToken` for coordinated server shutdown | Already a transitive dep of tokio; provides the standard pattern for `axum::serve(...).with_graceful_shutdown(token.cancelled())`. The timeout branch fires `token.cancel()` after the deadline. |
| `std::process::ExitCode` | stdlib | Structured exit codes (0=pass, 1=fail, 2=timeout/error) | Rust stdlib since 1.61; no crate needed. Return `ExitCode` from `main` via the `Termination` trait. |

**Pattern:** `tokio::select!` on `tokio::time::timeout(duration, wait_for_callback_rx)` vs the cancellation token. When the timeout fires, cancel the token (stopping the server), then compute the scorecard from whatever callbacks arrived. Exit 0 if all expected tiers fired, 1 if partial/none, 2 if infrastructure error.

No new Cargo dependencies are required for the core `test-agent` logic. `tokio-util` should be made an explicit dependency (it is currently implicit) so the `CancellationToken` API surface is stable.

---

### GitHub Actions — CI Workflow (test + clippy + fmt)

| Tool | Version/Tag | Purpose | Why |
|------|------------|---------|-----|
| `dtolnay/rust-toolchain` | `@stable` | Install Rust toolchain + components | The de-facto standard; lighter than `actions-rust-lang/setup-rust-toolchain`, no extra abstraction layer, used by the Rust project itself. Specify `components: clippy, rustfmt` inline. |
| `Swatinem/rust-cache` | `@v2` | Cache `~/.cargo` and `target/` | Cuts cold-build time from ~3 min to ~30 sec on subsequent runs. Use `shared-key` per job to prevent cross-contamination. Must appear AFTER toolchain install (cache key includes rustc version). |

**Three jobs (not one):** `test`, `clippy`, `fmt` as separate jobs that run in parallel. This gives faster feedback than a sequential single job. Use `cargo clippy -- -D warnings` (deny warnings) and `cargo fmt --check` (fail on unformatted code).

No other CI-specific crates are needed.

---

### GitHub Actions — Release Workflow (cross-platform binaries)

| Tool | Version/Tag | Purpose | Why |
|------|------------|---------|-----|
| `taiki-e/upload-rust-binary-action` | `@v1` | Build and upload binaries to GitHub Releases | Best-in-class for this use case: handles cross-compilation via `cross` automatically for Linux targets, native cargo for macOS/Windows, produces `.tar.gz` archives, uploads to GitHub Releases. Maintained, widely adopted. |
| `taiki-e/create-gh-release-action` | `@v1` | Create the GitHub Release entry | Companion to `upload-rust-binary-action`; parses CHANGELOG.md for release notes automatically. |

**Targets to ship:**

| Target | Runner | Method |
|--------|--------|--------|
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `cross` (via action) |
| `aarch64-unknown-linux-gnu` | `ubuntu-latest` | `cross` (via action) |
| `x86_64-apple-darwin` | `macos-latest` | native cargo |
| `aarch64-apple-darwin` | `macos-latest` | native cargo |

**Not shipping Windows yet** — PROJECT.md marks Windows as v2+ out of scope.

**Trigger:** `push` to tags matching `v[0-9]+.*`. The CI workflow triggers on push/PR; the release workflow triggers on tag push only.

**Alternative considered — `cargo-dist`:** cargo-dist is opinionated end-to-end tooling (generates workflow YAML, handles installers, Homebrew tap, etc.). More powerful but also more invasive — it wants to own your entire release process. For a single binary with no installer requirement yet, `upload-rust-binary-action` gives full control with less ceremony. Choose `cargo-dist` if/when a Homebrew tap or shell installer is needed.

---

### Containerized Deployment (Fly.io live demo at honeyprompt.sh)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Docker multi-stage build | Dockerfile | Build minimal runtime image | Stage 1: `rust:slim` builder with `x86_64-unknown-linux-musl` target. Stage 2: `scratch` or `gcr.io/distroless/static-debian12`. Produces images under 15 MB. |
| `flyctl` / `fly.toml` | current | Deploy to Fly.io machines | Fly.io has native Rust support with auto-detected Dockerfile. Persistent machine at ~$2/month (shared-cpu-1x 256 MB), pay-as-you-go. Axum is already in-tree — zero new deps for deployment. |

**musl static binary approach:** Compile with `--target x86_64-unknown-linux-musl` in the build stage. This produces a fully static binary with no glibc dependency, enabling `scratch` or `distroless/static` as the runtime base. `rustls` is already used (Axum default with tokio), so no OpenSSL linking issue.

**Distroless over scratch:** `gcr.io/distroless/static-debian12` is preferred over bare `scratch` because it includes CA certificates (needed if honeyprompt.sh ever makes outbound HTTPS calls) and a non-root user. Attack surface is near-identical to scratch.

**fly.toml key settings:**

```toml
[http_service]
  internal_port = 8080   # matches honeyprompt serve --port
  force_https = true
  auto_stop_machines = false   # always-on for live demo
  auto_start_machines = true
  min_machines_running = 1

[[vm]]
  size = "shared-cpu-1x"
  memory = "256mb"
```

**Fly.io pricing note:** Free tier was removed for accounts after October 2024. A single always-on `shared-cpu-1x 256 MB` machine costs ~$1.94/month. This is fine for a live demo — document in README.

---

## Supporting Libraries — Explicit Cargo.toml Additions

| Crate | Version | Reason for Adding Explicitly |
|-------|---------|------------------------------|
| `tokio-util` | `0.7` | Currently implicit transitive dep; make explicit to use `CancellationToken` for `test-agent` shutdown coordination |

Everything else for new features (Axum graceful shutdown, exit codes, timeout) is already available through existing dependencies or stdlib.

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `tokio-util` CancellationToken | `tokio::sync::oneshot` for shutdown signal | oneshot works for simpler cases; CancellationToken is preferred when multiple tasks need to observe the same shutdown signal (server + monitor loop) |
| `taiki-e/upload-rust-binary-action` | `cargo-dist` | cargo-dist if you want generated installers, Homebrew tap, or shell installer scripts — premature for v2.0 |
| `taiki-e/upload-rust-binary-action` | `houseabsolute/actions-rust-cross` | actions-rust-cross if you need more control over cross invocation (e.g., custom cross config); upload-rust-binary-action is simpler for the common case |
| Fly.io | Railway / Render / VPS | Railway/Render if managed HTTPS + zero-config DB is needed; VPS if you want persistent SQLite volumes without volume management; Fly.io wins for single-binary Rust with no managed DB |
| `distroless/static` Docker base | `scratch` | scratch if CA certs are provably never needed; distroless costs nothing extra and avoids certificate debugging surprises |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `actions-rs/*` (actions-rs org) | Unmaintained — last commit 2022, several actions deprecated | `dtolnay/rust-toolchain` + `Swatinem/rust-cache` |
| `wasm-pack` / `cargo-web` | Irrelevant to CLI distribution | Native cargo builds |
| `openssl-sys` with vendored feature | Breaks musl static builds; complex to cross-compile | Use `rustls` (Axum's default; already in-tree) |
| Alpine Linux as Docker base | `glibc` vs `musl` mismatch when mixing pre-built binaries; fiddly | musl static binary + `distroless/static` |
| Windows cross-compilation via `cross` | `cross` uses Linux Docker containers — MSVC toolchain requires native Windows runner | Use `windows-latest` runner with native cargo for Windows targets (deferred to v2+) |

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `tokio-util@0.7` | `tokio@1` | Same major version family; `tokio@1` is already in Cargo.toml |
| `axum@0.8` graceful shutdown | `tokio-util@0.7` CancellationToken | `axum::serve(...).with_graceful_shutdown(token.cancelled_owned())` is the documented pattern |
| musl target | `rusqlite` bundled feature | `rusqlite` with `features = ["bundled"]` compiles SQLite from source — works with musl; no dynamic linking issue |
| Fly.io Dockerfile | multi-stage musl build | Fly.io's buildkit supports multi-stage; first build is slow (~5 min), subsequent deploys are fast |

---

## Installation

```toml
# Cargo.toml — only new explicit addition for v2.0
[dependencies]
tokio-util = { version = "0.7", features = ["rt"] }
```

```yaml
# .github/workflows/ci.yml (new file)
- uses: dtolnay/rust-toolchain@stable
  with:
    components: clippy, rustfmt
- uses: Swatinem/rust-cache@v2
```

```yaml
# .github/workflows/release.yml (new file — triggered on tag push)
- uses: taiki-e/create-gh-release-action@v1
- uses: taiki-e/upload-rust-binary-action@v1
  with:
    bin: honeyprompt
    tar: unix
    token: ${{ secrets.GITHUB_TOKEN }}
```

---

## Sources

- [taiki-e/upload-rust-binary-action](https://github.com/taiki-e/upload-rust-binary-action) — binary release action, targets, token interface
- [dtolnay/rust-toolchain](https://github.com/dtolnay/rust-toolchain) — toolchain action
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) — caching best practices, key ordering requirement
- [tokio-util CancellationToken docs](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html) — shutdown coordination API
- [Axum graceful shutdown discussion](https://github.com/tokio-rs/axum/discussions/2565) — CancellationToken + axum::serve pattern
- [Tokio shutdown guide](https://tokio.rs/tokio/topics/shutdown) — official guidance on graceful shutdown with select/oneshot/CancellationToken
- [Fly.io Rust docs](https://fly.io/docs/rust/) — Dockerfile auto-detection, deployment basics
- [Fly.io pricing](https://fly.io/docs/about/pricing/) — always-on machine cost ~$1.94/month
- [Cross-platform Rust CI/CD pipeline](https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/) — December 2025 workflow patterns, MEDIUM confidence
- [cargo-dist introduction](https://axodotdev.github.io/cargo-dist/book/introduction.html) — why deferred: installer/tap features premature for v2.0
- [How to Create Minimal Docker Images for Rust Binaries](https://oneuptime.com/blog/post/2026-01-07-rust-minimal-docker-images/view) — distroless vs scratch trade-offs, January 2026
- [std::process::ExitCode](https://doc.rust-lang.org/beta/std/process/struct.ExitCode.html) — stdlib exit code API

---
*Stack research for: HoneyPrompt v2.0 — test-agent, CI/CD, deployment*
*Researched: 2026-03-29*
