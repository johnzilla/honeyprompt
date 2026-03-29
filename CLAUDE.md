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

Technology stack not yet documented. Will populate after codebase mapping or first phase.
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
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
