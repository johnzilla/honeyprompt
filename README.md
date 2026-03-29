# HoneyPrompt

Terminal-first security tool that detects unsafe AI browsing agent behavior using honeypot pages and prompt-injection canaries.

## What This Is

HoneyPrompt generates static honeypot web pages with visible human warnings and hidden prompt-injection payloads embedded across multiple locations (HTML comments, meta tags, invisible elements, JSON-LD, semantic prose). When an AI browsing agent visits the page and follows the injected instructions, it makes an HTTP callback that proves a specific level of compliance. Built in Rust, ships as a single binary with no runtime dependencies.

Security researchers and platform defenders deploy HoneyPrompt pages to gather evidence of agentic web abuse without collecting secrets, credentials, or user data.

## Proof Levels

HoneyPrompt uses a five-tier graduated evidence model:

- **Tier 1: Arbitrary callback** — Agent executed an injected outbound request
- **Tier 2: Conditional callback** — Agent evaluated a condition and selected the correct branch
- **Tier 3: Computed callback** — Agent performed a non-sensitive computation and returned the result
- **Tier 4: Capability introspection** — Agent inspected its own tools or permissions and encoded non-sensitive metadata (planned)
- **Tier 5: Multi-step compliance chain** — Agent completed a sequence of dependent harmless actions (planned)

Each tier's callback URL carries only a unique cryptographic nonce, the prompt ID, and the tier level — no secrets or sensitive data.

## Installation

Requires Rust toolchain (stable). Install from [rustup.rs](https://rustup.rs/) if not present.

```sh
git clone https://github.com/honeyprompt/honeyprompt
cd honeyprompt
cargo build --release
```

The binary is at `target/release/honeyprompt`. Copy it to your `$PATH` or run it directly.

## Usage

### Initialize a project

```sh
honeyprompt init
honeyprompt init --dir /path/to/project
```

Creates a project directory containing `honeyprompt.toml` with default configuration:

- `project_name` — Name embedded in generated page content
- `callback_url` — Base URL where callback beacons will be sent (e.g., `https://your-server.example.com/cb`)
- `output_dir` — Where generated files are written (default: `site/`)

Edit `honeyprompt.toml` to set your callback URL before generating.

### Generate the honeypot site

```sh
honeyprompt generate
honeyprompt generate --dir /path/to/project
```

Reads the project config and writes static files to the output directory:

- `index.html` — Honeypot page with visible human warning and payloads for all active tiers embedded across multiple locations
- `robots.txt` — Disallow rules for known AI crawlers
- `ai.txt` — Companion disallow file

Each payload contains a unique cryptographic nonce in its callback URL, so individual visits and tiers can be correlated on the server side.

### Planned commands

The following commands are planned for future phases:

- `honeyprompt serve` — Serve the honeypot and listen for callbacks on the same port (Phase 2)
- `honeyprompt monitor` — Live TUI event viewer showing callbacks as they arrive (Phase 3)
- `honeyprompt report` — Generate a Markdown disclosure report from captured events (Phase 4)

## How It Works

1. `init` creates a project directory with a `honeyprompt.toml` config file containing your callback URL and project name.
2. `generate` reads the config, loads the payload catalog (Tiers 1-3), and assigns a unique cryptographic nonce to each payload instance.
3. The generator renders `index.html` using a built-in template that embeds payloads in: HTML comments, `<meta>` tags, invisible `<span>` elements, JSON-LD structured data, and natural-language prose.
4. Every generated page includes a visible human warning so real users know the page is a security research instrument.
5. `robots.txt` and `ai.txt` are generated with disallow rules, creating a detectable signal when compliant crawlers respect them but non-compliant agents do not.
6. When an agent triggers a callback, the URL encodes the prompt ID, nonce, and tier — enough to identify what happened without transmitting any sensitive data.

## Project Status

| Phase | Capability | Status |
|-------|-----------|--------|
| 1 | Generation Pipeline — `init`, `generate`, payload catalog (Tiers 1-3), SQLite schema | Complete |
| 2 | Server and Detection — `serve`, callback listener, agent fingerprinting, event storage | Planned |
| 3 | TUI Monitor — live event display, filters, session-based counts, replay flagging | Planned |
| 4 | Report and Landing — Markdown disclosure report, instrumented landing page | Planned |

Phase 1 is complete. The tool builds and runs. Phases 2-4 are sequential: each phase depends on the one before it.

## Safety Model

HoneyPrompt is designed to produce evidence, not to exploit or surveil:

- Payloads carry only prompt ID, nonce, tier, and derived non-sensitive values — no API keys, session tokens, file contents, or environment variables
- Every generated page displays a visible warning for human visitors
- All payload instructions are auditable in the open-source catalog
- Custom payload authoring is not supported in v1 — the curated catalog ensures the safety guarantee holds
- No harmful actions are requested of agents at any tier

## License

See LICENSE.
