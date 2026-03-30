# Phase 6: Release Infrastructure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-30
**Phase:** 06-release-infrastructure
**Areas discussed:** Release trigger & naming, Build matrix

---

## Release Trigger

| Option | Description | Selected |
|--------|-------------|----------|
| v* tags only | Push v2.0.0 → builds + creates GitHub Release. No pre-releases. | ✓ |
| v* tags + pre-releases | v2.0.0-rc1 creates pre-release on GitHub | |
| Manual dispatch | workflow_dispatch button in Actions UI | |

**User's choice:** v* tags only
**Notes:** None

---

## Binary Naming

| Option | Description | Selected |
|--------|-------------|----------|
| honeyprompt-{target} | Full Rust triple. e.g., honeyprompt-x86_64-unknown-linux-musl. Unambiguous. | ✓ |
| honeyprompt-{os}-{arch} | Shorter. e.g., honeyprompt-linux-amd64. Requires mapping. | |
| You decide | Claude picks | |

**User's choice:** honeyprompt-{target}
**Notes:** Standard for Rust projects

---

## Static Linking

| Option | Description | Selected |
|--------|-------------|----------|
| musl static | No glibc dependency. Works on any Linux. rusqlite bundled. Slightly larger. | ✓ |
| glibc dynamic | Standard linking. Requires matching glibc. Smaller. | |
| You decide | Claude picks | |

**User's choice:** musl static
**Notes:** None

---

## Cross-compilation

| Option | Description | Selected |
|--------|-------------|----------|
| cross + native split | cross (Docker) for Linux, native cargo for macOS. Most reliable for rusqlite bundled. | ✓ |
| actions-rust-cross for all | Simpler YAML but less control. Third-party dep. | |
| You decide | Claude picks | |

**User's choice:** cross + native split
**Notes:** None

---

## Claude's Discretion

- SHA256 checksums alongside binaries
- Compression format (tar.gz vs raw)
- Workflow YAML structure

## Deferred Ideas

- cargo-dist (future)
- crates.io publish (future)
- Windows binaries (out of scope)
