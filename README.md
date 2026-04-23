# HoneyPrompt

Terminal-first security tool that detects unsafe AI browsing agent behavior using honeypot pages and prompt-injection canaries.

**Live Demo:** [honeyprompt.sh](https://honeyprompt.sh) — a live honeypot serving canary payloads to AI agents right now. Want to run your own? See [Deploy Your Own](#deploy-your-own) below.

## What This Is

HoneyPrompt generates static honeypot web pages with visible human warnings and hidden prompt-injection payloads embedded across multiple locations (HTML comments, meta tags, invisible elements, JSON-LD, semantic prose). When an AI browsing agent visits the page and follows the injected instructions, it makes an HTTP callback that proves a specific level of compliance. Built in Rust, ships as a single binary with no runtime dependencies.

Security researchers and platform defenders deploy HoneyPrompt pages to gather evidence of agentic web abuse without collecting secrets, credentials, or user data.

## Why This Matters for Product Security Teams

This tool helps security and product teams mature their **AI risk registers** by providing measurable, auditable evidence of agent behavior in the wild. It supports:
- Automated compliance testing in CI/CD pipelines
- Evidence for coordinated vulnerability disclosure
- Better threat modeling for agentic features
- "Paved road" controls that let teams ship AI features faster while reducing risk

## Proof Levels

HoneyPrompt uses a five-tier graduated evidence model:

- **Tier 1: Arbitrary callback** — Agent executed an injected outbound request
- **Tier 2: Conditional callback** — Agent evaluated a condition and selected the correct branch
- **Tier 3: Computed callback** — Agent performed a non-sensitive computation and returned the result
- **Tier 4: Capability introspection** — Agent inspected its own tools or permissions and encoded non-sensitive metadata (planned)
- **Tier 5: Multi-step compliance chain** — Agent completed a sequence of dependent harmless actions (planned)

Each tier's callback URL carries only a unique cryptographic nonce, the prompt ID, and the tier level — no secrets or sensitive data.

## FAQ

Check the [FAQ](https://github.com/johnzilla/honeyprompt/FAQ.md) and find additional info below. 

## Deploy Your Own

Run a honeypot on your own domain. All steps below use `your-domain.com` as the example — replace it with your actual domain throughout.

### 1. Install

Download the latest release for your platform from [GitHub Releases](https://github.com/johnzilla/honeyprompt/releases/latest):

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `honeyprompt-x86_64-unknown-linux-musl.tar.gz` |
| Linux aarch64 | `honeyprompt-aarch64-unknown-linux-musl.tar.gz` |
| macOS x86_64 | `honeyprompt-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `honeyprompt-aarch64-apple-darwin.tar.gz` |

```sh
curl -LO https://github.com/johnzilla/honeyprompt/releases/latest/download/honeyprompt-x86_64-unknown-linux-musl.tar.gz
tar xzf honeyprompt-x86_64-unknown-linux-musl.tar.gz
./honeyprompt --version
```

Or build from source (requires [Rust stable](https://rustup.rs/)):

```sh
cargo install --git https://github.com/johnzilla/honeyprompt
```

See [Installation](#installation) below for full platform details.

### 2. Configure

**Option A — Interactive wizard** (recommended for first-time setup):

```sh
honeyprompt setup
# Prompts for: domain, bind address, tiers, page title
# Writes honeyprompt.toml
```

The wizard validates your domain, warns if DNS is not yet pointed at your server, and writes a `honeyprompt.toml` you can review and edit before serving.

**Option B — Zero-config** (no config file needed):

```sh
honeyprompt serve --domain your-domain.com
# Generates site in a tempdir and serves immediately
# Binds 0.0.0.0:8080, enables all tiers
```

Use Option B for a quick trial or when you want to skip the config file entirely. Option A is better for long-running production deployments where you want to commit a config to version control.

### 3. Deploy

Choose the path that matches your setup:

**Quick — single binary behind your reverse proxy:**

```sh
honeyprompt serve --domain your-domain.com
```

Point your Caddy, nginx, or Traefik instance at `localhost:8080`. The binary handles honeypot serving and callback recording in one process. See `deploy/templates/` for ready-made reverse proxy configs.

---

**Production — Docker Compose + Caddy (auto-TLS):**

```sh
# Copy templates
cp deploy/templates/docker-compose.yml deploy/templates/Caddyfile /opt/honeyprompt/

# Replace {DOMAIN} placeholder
sed -i 's/{DOMAIN}/your-domain.com/g' /opt/honeyprompt/docker-compose.yml /opt/honeyprompt/Caddyfile

# Start the stack
cd /opt/honeyprompt && docker compose up -d
```

Caddy auto-provisions a Let's Encrypt certificate on first request. SQLite persists in a Docker volume and survives image updates.

Architecture: Internet → Caddy (:443 TLS) → honeyprompt (:8080) → SQLite DB (persistent volume)

---

**Bare metal — systemd service:**

```sh
# Install binary
install -m 755 honeyprompt /usr/local/bin/

# Install unit file
cp deploy/templates/honeyprompt.service /etc/systemd/system/
sed -i 's/{DOMAIN}/your-domain.com/g' /etc/systemd/system/honeyprompt.service

# Enable and start
systemctl daemon-reload
systemctl enable --now honeyprompt
```

See `deploy/templates/honeyprompt.service` for the full unit file with `KillSignal=SIGINT` set correctly.

### 4. Verify

Once deployed, confirm everything is working:

```sh
curl -I https://your-domain.com              # Should return HTTP/2 200
curl https://your-domain.com/cb/v1/test      # Should return callback response
curl https://your-domain.com/stats           # Should return JSON aggregate counts
```

Then open the TUI monitor to watch live callbacks:

```sh
honeyprompt monitor
```

## Installation

### Prebuilt binaries

Download the latest release for your platform from [GitHub Releases](https://github.com/johnzilla/honeyprompt/releases/latest).

| Platform | Binary |
|----------|--------|
| Linux x86_64 | `honeyprompt-x86_64-unknown-linux-musl.tar.gz` |
| Linux aarch64 | `honeyprompt-aarch64-unknown-linux-musl.tar.gz` |
| macOS x86_64 | `honeyprompt-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `honeyprompt-aarch64-apple-darwin.tar.gz` |

Example (Linux x86_64):

```sh
curl -LO https://github.com/johnzilla/honeyprompt/releases/latest/download/honeyprompt-x86_64-unknown-linux-musl.tar.gz
tar xzf honeyprompt-x86_64-unknown-linux-musl.tar.gz
./honeyprompt --version
```

### Build from source

Requires Rust toolchain (stable). Install from [rustup.rs](https://rustup.rs/) if not present.

```sh
cargo install --git https://github.com/johnzilla/honeyprompt
```

Or clone and build:

```sh
git clone https://github.com/johnzilla/honeyprompt
cd honeyprompt
cargo build --release
```

## Usage

### Run the setup wizard

```sh
honeyprompt setup
```

Interactive wizard for first-time configuration. Prompts for domain, bind address, enabled tiers, and page title. Writes `honeyprompt.toml`. Validates DNS before completing. Exits without overwriting if a config already exists.

### Initialize a project (manual config)

```sh
honeyprompt init
honeyprompt init --dir /path/to/project
```

Creates a project directory containing `honeyprompt.toml` with default configuration:

- `project_name` — Name embedded in generated page content
- `callback_url` — Base URL where callback beacons will be sent (e.g., `https://your-domain.com/cb`)
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

### Serve the honeypot

```sh
honeyprompt serve
honeyprompt serve --port 8080
honeyprompt serve --domain your-domain.com
```

Starts an HTTP server that serves the honeypot page and listens for callback beacons on the same port. Each incoming callback is fingerprinted and stored in SQLite.

The `--domain` flag enables zero-config mode: generates a fresh site in a tempdir, sets the callback base URL to your domain, binds `0.0.0.0:8080`, and enables all tiers — no config file needed.

### Monitor live callbacks

```sh
honeyprompt monitor
honeyprompt monitor --dir /path/to/project
```

Opens the TUI live event viewer. Displays incoming callbacks as they arrive with agent fingerprinting, tier breakdown, and session grouping. Vim-style navigation. Can attach to a running server or start integrated mode.

### Generate a disclosure report

```sh
honeyprompt report
honeyprompt report --output report.md
```

Generates a Markdown disclosure report from captured events, including an executive summary, per-session tier breakdown, agent fingerprints, and full request metadata.

### Test an AI agent

```sh
honeyprompt test-agent --listen 0.0.0.0:8080
honeyprompt test-agent --listen 0.0.0.0:8080 --timeout 30 --format json
```

Runs a self-contained compliance test: spins up a honeypot server, waits for callbacks, then outputs a pass/fail scorecard. Exit code 0 means no canaries triggered (safe agent). Exit code 1 means one or more canaries triggered. Designed for CI pipelines testing AI agent compliance.

## How It Works

1. `honeyprompt setup` (or `init`) creates a config with your domain and desired settings.
2. `honeyprompt serve --domain your-domain.com` (or `serve` with a config) generates and serves the honeypot in one step.
3. `generate` reads the config, loads the payload catalog (Tiers 1-3), and assigns a unique cryptographic nonce to each payload instance.
4. The generator renders `index.html` using a built-in template that embeds payloads in: HTML comments, `<meta>` tags, invisible `<span>` elements, JSON-LD structured data, and natural-language prose.
5. Every generated page includes a visible human warning so real users know the page is a security research instrument.
6. `robots.txt` and `ai.txt` are generated with disallow rules, creating a detectable signal when compliant crawlers respect them but non-compliant agents do not.
7. When an agent triggers a callback, the URL encodes the prompt ID, nonce, and tier — enough to identify what happened without transmitting any sensitive data.

## Project Status

| Phase | Capability | Status |
|-------|-----------|--------|
| Phase 1 | Generation Pipeline — `init`, `generate`, payload catalog (Tiers 1-3), SQLite schema | Complete |
| Phase 2 | Server and Detection — `serve`, callback listener, agent fingerprinting, event storage | Complete |
| Phase 3 | TUI Monitor — live event display, filters, session-based counts, replay flagging | Complete |
| Phase 4 | Report and Landing — `report` subcommand, Markdown disclosure report, instrumented landing page | Complete |
| Phase 5 | Test Agent — `test-agent` subcommand, compliance scorecard, CI integration, GitHub Actions CI | Complete |
| Phase 6 | Release Infrastructure — cross-platform binary releases, README installation guide | Complete |
| Phase 7 | Live Demo Deployment — `honeyprompt serve` running at honeyprompt.sh with HTTPS and auto-restart | Complete |
| Phase 8 | Public Launch — README polish, ethics section, discoverability | Complete |
| Phase 9 | Server-Side Identity & Stats — footer identity, security.txt, /stats JSON endpoint | Complete |
| Phase 10 | Landing Page — honeyprompt.dev with live stats from /stats API | Complete |
| Phase 11 | Setup Wizard & Zero-Config Serve — `honeyprompt setup`, `--domain` flag, tempdir mode | Complete |
| Phase 12 | Documentation & Deploy Templates — Deploy Your Own guide, deploy/templates/ for all platforms | Complete |

## Ethics and Safety

HoneyPrompt is not an exploit tool, a vulnerability scanner, or a surveillance instrument. It is not designed to collect secrets, credentials, or private data of any kind. Its sole purpose is to produce verifiable, graduated evidence of whether AI agents comply with prompt-injection instructions from untrusted web content — so defenders can measure the risk, not cause harm.

HoneyPrompt is designed to produce evidence, not to exploit or surveil:

- Payloads carry only prompt ID, nonce, tier, and derived non-sensitive values — no API keys, session tokens, file contents, or environment variables
- Every generated page displays a visible warning for human visitors
- All payload instructions are auditable in the open-source catalog
- Custom payload authoring is not supported — the curated catalog ensures the safety guarantee holds
- No harmful actions are requested of agents at any tier

## License

[MIT](https://github.com/johnzilla/honeyprompt/LICENSE)
