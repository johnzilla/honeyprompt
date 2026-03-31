# Phase 6: Release Infrastructure - Context

**Gathered:** 2026-03-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a GitHub Actions release workflow that builds cross-platform binaries on `v*` tag push and uploads them to GitHub Releases. Update README with `cargo install` and prebuilt binary download instructions.

</domain>

<decisions>
## Implementation Decisions

### Release Trigger
- **D-01:** Release triggered by `v*` tag push only (e.g., `v2.0.0`). No pre-release support. No manual dispatch.
- **D-02:** Workflow creates a GitHub Release and attaches all binaries as assets.

### Binary Naming
- **D-03:** Use full Rust target triple: `honeyprompt-{target}` (e.g., `honeyprompt-x86_64-unknown-linux-musl`, `honeyprompt-aarch64-apple-darwin`). Unambiguous, standard for Rust projects.

### Build Matrix
- **D-04:** Four targets: x86_64-unknown-linux-musl, aarch64-unknown-linux-musl, x86_64-apple-darwin, aarch64-apple-darwin.
- **D-05:** Linux targets use musl static linking — single static binary, no glibc dependency, maximum portability across distros. `rusqlite` bundled feature compiles SQLite from source for musl compatibility.
- **D-06:** Cross-compilation split by OS: Linux targets use `cross` (Docker-based) on ubuntu runners, macOS targets use native `cargo` on `macos-latest` runners. Matrix routes by `runs-on` based on target.

### Carried Forward from Phase 5
- **D-07 (from D-09):** Use dtolnay/rust-toolchain + Swatinem/rust-cache. Rust stable channel.
- **D-08 (from D-10):** All third-party actions SHA-pinned with version comments.

### Claude's Discretion
- Whether to include SHA256 checksums alongside binaries (standard practice but adds complexity)
- Whether to compress binaries in .tar.gz or ship raw executables
- Exact workflow YAML structure (single job with matrix vs separate jobs)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing CI
- `.github/workflows/ci.yml` — Existing CI workflow from Phase 5. Release workflow should follow same SHA-pinning pattern and action choices.

### Research
- `.planning/research/STACK.md` — taiki-e actions, cross-compilation details, Fly.io deployment
- `.planning/research/FEATURES.md` — cargo-dist analysis (deferred), binary release patterns
- `.planning/research/ARCHITECTURE.md` — Build order, release workflow structure
- `.planning/research/PITFALLS.md` — macOS cross-compile must use macos runners, actions-rs deprecated

### Prior Phase Context
- `.planning/phases/05-test-agent-subcommand/05-CONTEXT.md` — D-08 through D-10 (CI decisions that carry forward)

### Design Doc
- `~/.gstack/projects/johnzilla-honeyprompt/john-main-design-20260329-180748.md` — Distribution Plan section

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `.github/workflows/ci.yml` — SHA-pinned action pattern to replicate in release workflow
- `Cargo.toml` — Already has `rusqlite` with bundled feature, `tokio-util`, all deps needed

### Established Patterns
- SHA pinning: `actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5  # v4`
- Rust stable channel via dtolnay/rust-toolchain

### Integration Points
- `.github/workflows/release.yml` — New file
- `README.md` — Add installation section with cargo install + binary download
- `Cargo.toml` — Version field used as release version source

</code_context>

<specifics>
## Specific Ideas

- Research recommends `taiki-e/upload-rust-binary-action` and `taiki-e/create-gh-release-action` for the upload step — evaluate during planning
- `cross` tool needs to be installed in the workflow for Linux ARM targets
- Binary names should be tar.gz'd per platform convention or shipped raw — Claude's discretion

</specifics>

<deferred>
## Deferred Ideas

- cargo-dist — useful later for Homebrew tap and installer scripts, not needed for v2
- crates.io publish — deferred per design doc (binary releases + cargo install from git sufficient)
- Windows binaries — out of scope per PROJECT.md

</deferred>

---

*Phase: 06-release-infrastructure*
*Context gathered: 2026-03-30*
