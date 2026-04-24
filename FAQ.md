## FAQ

<details>
<summary>What is HoneyPrompt?</summary>

HoneyPrompt is a terminal-first security tool that helps detect unsafe AI browsing agent behavior. It generates static honeypot web pages containing hidden prompt-injection canaries. When an AI agent visits the page and follows the injected instructions, it triggers harmless HTTP callbacks that provide auditable evidence of compliance.

Built in Rust as a single static binary. No runtime dependencies.

</details>

<details>
<summary>How does it work?</summary>

1. You run `honeyprompt serve` (or `generate` + serve) on your domain.
2. It creates a honeypot page (`index.html`) with visible human warnings + hidden prompt-injection payloads embedded in multiple locations (HTML comments, meta tags, invisible elements, JSON-LD, natural language, etc.).
3. Each payload contains a unique cryptographic nonce and tier identifier.
4. If an AI agent obeys the injection, it makes a callback to your server.
5. The server records the callback with agent fingerprinting and stores it in SQLite.

All callbacks are non-destructive and contain no secrets.

</details>

<details>
<summary>What are the Proof Tiers?</summary>

HoneyPrompt uses a graduated 5-tier model:

- **Tier 1**: Arbitrary callback (basic outbound request)
- **Tier 2**: Conditional callback (evaluated a condition)
- **Tier 3**: Computed callback (performed a simple computation)
- **Tier 4**: Capability introspection (agent self-reports a sorted, base64-encoded list of its tools or scopes — no secrets)
- **Tier 5**: Multi-step compliance chain (agent extracts a page-visible seed, applies a deterministic formula, submits a 3-digit proof that the server re-verifies)

Higher tiers provide stronger evidence of agentic behavior.

</details>

<details>
<summary>Is it safe? Does it collect sensitive data?</summary>

No.  
- Payloads never request or transmit secrets, credentials, files, or private data.
- Every honeypot page displays a clear visible warning for human visitors.
- All prompt payloads are open-source and auditable.
- Callbacks contain only a nonce, prompt ID, and tier — nothing sensitive.

It is designed strictly for defensive security research and risk measurement.

</details>

<details>
<summary>Who should use HoneyPrompt?</summary>

- Security and product teams building agentic AI features
- AI safety / red team researchers
- Platform defenders monitoring for prompt-injection abuse
- Teams that want measurable evidence for AI risk registers and responsible disclosure

</details>

<details>
<summary>Can I run it on my own domain?</summary>

Yes — easily.  
Use `honeyprompt setup` (interactive wizard) or `honeyprompt serve --domain your-domain.com` (zero-config).  

It works behind reverse proxies (Caddy, nginx, Traefik), in Docker, or as a systemd service. Full deployment templates are included.

</details>

<details>
<summary>What does the TUI monitor show?</summary>

The `honeyprompt monitor` command opens a live terminal interface showing:
- Incoming callbacks in real time
- Agent fingerprints
- Tier breakdown per session
- Session grouping
- Statistics

Perfect for watching agents interact with your honeypot live.

</details>

<details>
<summary>Is HoneyPrompt production-ready?</summary>

Yes. Version 4.0 is stable with complete generation, serving, monitoring, reporting, and deployment tooling. It is already running live at [honeyprompt.sh](https://honeyprompt.sh) with additional info at [honeyprompt.dev](https://honeyprompt.dev).

</details>

<details>
<summary>How can I help?</summary>

- Star the repo
- Deploy it and share feedback
- Open issues or PRs
- Use it in your AI agent testing / red teaming workflows
- Review the ethics and safety section

</details>
