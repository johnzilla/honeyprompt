<!-- GSD:project-start source:PROJECT.md -->
## Project

**HoneyPrompt**

HoneyPrompt is a terminal-first security tool that detects and measures unsafe behavior by AI browsing agents. It generates honeypot web pages containing visible human warnings and hidden prompt-injection canaries, then records HTTP callbacks that prove varying levels of agent compliance with injected instructions. Built in Rust for security researchers, defenders, and platform teams who want evidence of agentic web abuse without collecting secrets or performing harmful actions.

**Core Value:** Provide graduated, verifiable proof that AI agents follow prompt-injection instructions from untrusted web content — without requiring secrets or causing harm.

### Constraints

- **Language**: Rust — single-binary distribution, performance, security community credibility
- **CLI**: Clap for argument parsing
- **TUI**: Ratatui for terminal UI
- **HTTP**: Axum or equivalent lightweight async stack
- **Storage**: SQLite via rusqlite or similar
- **Templates**: Built-in for site generation and reports
- **Platform**: Linux and macOS first
- **Performance**: Fast startup, low memory footprint
- **Ethics**: All generated content must include visible warnings for humans; payloads must be auditable
<!-- GSD:project-end -->

<!-- GSD:stack-start source:STACK.md -->
## Technology Stack

- **Rust** (stable) with Clap derive CLI, Axum HTTP, Ratatui TUI, dialoguer interactive prompts
- **SQLite** via rusqlite (sync) + tokio-rusqlite (async)
- **rust-embed** for bundled assets (templates, payload catalog)
- **tokio** async runtime with broadcast channels for event pipeline
- **GitHub Actions** CI (test/clippy/fmt) + release workflow (4-target cross-platform binaries)
- **Docker** + Caddy for deployment (ghcr.io/johnzilla/honeyprompt)
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

- All async entry points use `tokio::runtime::Runtime::new()` in main.rs (not `#[tokio::main]`)
- Event pipeline: mpsc(256) for raw callbacks → broadcast(1024) for processed events
- `build_router()` is the reusable Axum router constructor
- `ConnectInfo<SocketAddr>` + `into_make_service_with_connect_info` for peer address extraction
- Tests: unit tests in `#[cfg(test)] mod tests` within each module, integration tests in `tests/`
- CI: all GitHub Actions SHA-pinned with version comments
- Config precedence: CLI flags > config file > built-in defaults (`config_with_overrides`)
- `--domain` with existing project regenerates output if callback_base_url changed
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

- **CLI** (`cli/mod.rs`, `main.rs`): Clap derive with Commands enum (Init, Generate, Serve, Monitor, Report, Setup, TestAgent). Serve accepts `--domain`/`--bind`/`--tiers` flags for zero-config mode.
- **Setup** (`setup/mod.rs`): Interactive wizard using `dialoguer` crate. Prompts for domain, bind, tiers, page title. Writes honeyprompt.toml with DNS validation warning.
- **Generator** (`generator/mod.rs`): Loads payload catalog via rust-embed, assigns nonces, renders templates (index.html, robots.txt, ai.txt, security.txt)
- **Server** (`server/mod.rs`): Axum router serving static honeypot + `/cb/v1/{nonce}` callback handler + `/stats` JSON endpoint with CORS
- **Broker** (`broker/mod.rs`): Receives raw callbacks via mpsc, enriches with fingerprint/classification, fans out via broadcast to DB writer + stdout logger
- **Store** (`store/mod.rs`): SQLite with WAL mode, replay detection, session grouping, per-tier queries
- **Monitor** (`monitor/mod.rs`): Ratatui TUI with integrated server mode or DB attach mode
- **TestAgent** (`test_agent/mod.rs`): Ephemeral tempdir pipeline with CancellationToken timeout and per-tier scorecard
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->

## Skill routing

When the user's request matches an available skill, ALWAYS invoke it using the Skill
tool as your FIRST action. Do NOT answer directly, do NOT use other tools first.
The skill has specialized workflows that produce better results than ad-hoc answers.

Key routing rules:
- Product ideas, "is this worth building", brainstorming → invoke office-hours
- Bugs, errors, "why is this broken", 500 errors → invoke investigate
- Ship, deploy, push, create PR → invoke ship
- QA, test the site, find bugs → invoke qa
- Code review, check my diff → invoke review
- Update docs after shipping → invoke document-release
- Weekly retro → invoke retro
- Design system, brand → invoke design-consultation
- Visual audit, design polish → invoke design-review
- Architecture review → invoke plan-eng-review
