# Architecture Patterns

**Domain:** Rust CLI security tool — honeypot page generation, HTTP callback listening, SQLite event storage, TUI monitoring
**Researched:** 2026-03-28
**Overall confidence:** HIGH (verified against official Tokio, Axum, Ratatui docs and community patterns)

---

## Recommended Architecture

HoneyPrompt is a single-binary, multi-mode application. Two major runtime modes exist:

1. **Offline mode** — `init`, `generate`, `report`, `export` subcommands run synchronously, touch only the filesystem and SQLite.
2. **Server mode** — `serve` subcommand launches an async Tokio runtime hosting the HTTP server, callback receiver, and TUI simultaneously.

### Top-Level Process Topology (Server Mode)

```
  ┌─────────────────────────────────────────────────────────────┐
  │                     Tokio Runtime                           │
  │                                                             │
  │  ┌──────────────┐   callback_tx   ┌──────────────────────┐ │
  │  │  Axum Server │ ──────────────► │   Event Broker Task  │ │
  │  │  (HTTP + CB) │                 │  (broadcast fan-out) │ │
  │  └──────────────┘                 └───────┬──────────────┘ │
  │                                           │                 │
  │                           ┌──────────────┴──────────┐      │
  │                           │                         │       │
  │                   ┌───────▼──────┐        ┌────────▼─────┐ │
  │                   │  DB Writer   │        │  TUI Task    │ │
  │                   │  Task        │        │  (Ratatui)   │ │
  │                   └───────┬──────┘        └──────────────┘ │
  │                           │                                 │
  │                   ┌───────▼──────┐                         │
  │                   │   SQLite     │                         │
  │                   │   (via       │                         │
  │                   │   tokio-     │                         │
  │                   │   rusqlite)  │                         │
  │                   └──────────────┘                         │
  └─────────────────────────────────────────────────────────────┘
```

The Axum server handles two concerns on the same port: serving honeypot pages and receiving callback beacons. The Event Broker is a Tokio task with a `broadcast` channel, allowing the TUI and DB writer to receive every event independently without coordination.

---

## Component Boundaries

| Component | Responsibility | Consumes | Produces |
|-----------|---------------|----------|---------|
| **CLI Layer** (`cli/`) | Clap parse, subcommand dispatch, tokio runtime entry | argv | Subcommand enum |
| **Config / Workspace** (`config/`) | Read/write `honeyprompt.toml`, resolve paths | Filesystem | `Config` struct |
| **Payload Catalog** (`catalog/`) | Curated prompt-injection payloads keyed by proof level | `Config` | `Payload` structs |
| **Generator** (`generator/`) | Render honeypot HTML, robots.txt, ai.txt from templates | `Payload`, `Config` | Static files on disk |
| **Axum Server** (`server/`) | Serve static files + receive GET/POST callback beacons | Filesystem, HTTP | Raw `CallbackEvent` |
| **Fingerprinter** (`fingerprint/`) | Extract IP, UA, headers, ASN metadata from requests | Axum `Request` parts | `AgentFingerprint` |
| **Event Broker** (`broker/`) | Fan out callback events to all subscribers | `CallbackEvent` + `AgentFingerprint` | `broadcast::Sender<Event>` |
| **DB Store** (`store/`) | Persist events to SQLite, query history | `broadcast::Receiver<Event>` | Rows; query results |
| **TUI Monitor** (`tui/`) | Ratatui live view, filters, event table | `broadcast::Receiver<Event>`, DB queries | Terminal frames |
| **Reporter** (`report/`) | Generate Markdown disclosure reports | DB queries | `.md` files |

### Explicit Non-Communications

- The TUI does not talk to the Axum server directly — events arrive via broadcast channel.
- The Generator does not run during `serve` — it is an offline-only concern.
- The Fingerprinter is a pure function (request → fingerprint), with no database access.
- The Payload Catalog is read-only at runtime; it is never mutated by incoming events.

---

## Data Flow

### Flow 1: Page Generation (offline)

```
CLI `generate` → Config → Payload Catalog → Generator
                                                 │
                                       Render HTML templates
                                       (nonce embedded in callback URLs)
                                                 │
                                       Write to output dir:
                                         index.html
                                         robots.txt
                                         ai.txt
                                         callback-map.json  ← nonce → payload metadata
```

The nonce is a URL-safe random token embedded in the beacon path (e.g., `/cb/v1/<nonce>`). `callback-map.json` is the local ledger mapping nonces to proof levels; the server reads this at startup.

### Flow 2: Callback Reception (server mode)

```
Agent HTTP request
       │
  Axum Router
  ├── GET /            → serve index.html (static)
  ├── GET /robots.txt  → serve robots.txt (static)
  └── GET /cb/v1/:nonce
          │
     Fingerprinter ─── extract IP, UA, headers
          │
     Nonce Lookup ──── resolve proof level from callback-map.json
          │
     CallbackEvent assembled
          │
     callback_tx.send(event) ── mpsc into Event Broker
```

### Flow 3: Event Broker Fan-Out

```
Event Broker Task
  receives on mpsc rx
  broadcasts on broadcast::Sender<Event>
        │
        ├──► DB Writer Task ──► tokio-rusqlite ──► SQLite
        └──► TUI Task ──────────────────────────► Terminal
```

`tokio::sync::broadcast` is chosen over `mpsc` here because both the DB writer and TUI must independently receive every event. Each consumer holds its own `broadcast::Receiver<Event>`.

### Flow 4: TUI Event Loop

```
TUI Task
  tokio::select! {
    event = broadcast_rx.recv()  → append to event list, re-render
    key   = crossterm_rx.recv()  → update filters/focus, re-render
    tick  = interval.tick()      → re-render (heartbeat)
  }
```

The TUI does not hold the SQLite write path. It reads a pre-materialized in-memory buffer of recent events (capped ring buffer, e.g., last 1000) populated from the broadcast channel. Historical queries go through the DB store via explicit async call.

### Flow 5: Report Generation (offline)

```
CLI `report` → Config → DB Store (query all events for session)
                                │
                        Markdown template
                                │
                        Write report.md
```

---

## Key Design Decisions and Rationale

### All-in-One Server (same port, same process)

The PRD mandates a single-process, all-in-one model. Axum handles this cleanly: one router with static file routes and callback routes registered together. No IPC, no sidecar.

### tokio-rusqlite for SQLite Async Bridge

`rusqlite::Connection` is not `Sync`, making it incompatible with direct use in async handlers. `tokio-rusqlite` wraps the connection in a background thread with mpsc/oneshot channels, making every call `.await`-able. This is the standard pattern (HIGH confidence, verified against docs.rs).

The DB store owns one `tokio_rusqlite::Connection`. The Event Broker's DB writer task is the sole writer. TUI and report queries are reads that can share the same connection through the `call()` method's serialization guarantee.

### broadcast Channel for Event Fan-Out

`tokio::sync::broadcast` is appropriate here (not `mpsc`) because the DB writer and the TUI are independent consumers that each need every event. The channel capacity should be large enough to absorb burst traffic; 1024 is a reasonable default. Lagged receivers receive an error that should be logged and recovered gracefully in the TUI task.

### Nonce-Based Payload Identification

The callback URL carries only a nonce (`/cb/v1/<nonce>`). The server maps the nonce to proof level at reception time using `callback-map.json`. No proof level or payload content travels in the URL. This limits information leakage and avoids URL-as-data anti-patterns.

### Fingerprinter as Pure Function

The `fingerprint::extract(request_parts) -> AgentFingerprint` function takes no external state. This makes it unit-testable in isolation and decouples fingerprinting logic from server state. ASN lookup (if added) becomes an injected async function, not embedded logic.

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Shared Mutable In-Memory State Between Axum and TUI

**What:** `Arc<Mutex<Vec<Event>>>` shared directly between Axum handlers and TUI render loop.
**Why bad:** Lock contention; render loop holds the mutex during frame draw, blocking event ingestion. Deadlock risk.
**Instead:** Use channels (broadcast) with each consumer owning its own state copy.

### Anti-Pattern 2: SQLite Write in Axum Request Handler

**What:** Writing to SQLite directly inside the `/cb/v1/:nonce` handler.
**Why bad:** Blocks the handler on I/O; SQLite writes are serialized — under load this creates a bottleneck on the most latency-sensitive path (confirming agent compliance).
**Instead:** The handler sends the event to the broker via a buffered channel and returns `200 OK` immediately. The DB writer task drains the channel asynchronously.

### Anti-Pattern 3: Blocking rusqlite in Async Context

**What:** Calling `rusqlite` directly with `tokio::spawn(async { ... })`.
**Why bad:** `rusqlite::Connection` is not `Send + Sync`; this fails to compile or requires unsafe workarounds.
**Instead:** Use `tokio-rusqlite` which moves the connection to a dedicated blocking thread and communicates via channels.

### Anti-Pattern 4: Rendering Payload Templates at Request Time

**What:** Generating honeypot HTML dynamically per-request.
**Why bad:** Increases latency; introduces risk of accidentally exposing generation logic state to response.
**Instead:** Generate pages offline during `generate` command; serve as static files from the filesystem. Axum's `ServeDir` handles this cleanly.

### Anti-Pattern 5: Storing Callback Body Content

**What:** Recording request body content in the SQLite events table.
**Why bad:** Violates the safety model — agents might POST sensitive data if instructions are miscrafted.
**Instead:** In metadata-only mode (default), only path, query params, and headers are stored. No body is read or stored. Request body is explicitly discarded.

---

## Component Build Order

Dependencies flow upward; build lower layers first.

```
Layer 0 (no deps):
  config/       — Config struct, file I/O
  catalog/      — Payload definitions, hardcoded data

Layer 1 (depends on Layer 0):
  fingerprint/  — Pure fn, no deps beyond std/http types
  store/        — DB schema, migrations, query interface (needs config for path)

Layer 2 (depends on Layer 1):
  generator/    — Needs catalog (payloads) + config (output path)
  broker/       — Needs store (to write), Event type definition

Layer 3 (depends on Layer 2):
  server/       — Needs broker (to send events), fingerprint (to extract)
  tui/          — Needs broker (to receive events), store (for history queries)
  report/       — Needs store (for query)

Layer 4 (orchestrates all):
  cli/          — Clap dispatch wiring all subcommands to Layer 0-3 components
```

### Practical Phasing Implications

1. **Phase 1 target:** `config` + `catalog` + `generator` — this gives a working `honeyprompt generate` with no async runtime needed.
2. **Phase 2 target:** `store` + `fingerprint` + basic `server` (static serving + callback reception, logging to stdout) — validates the core detection loop without TUI.
3. **Phase 3 target:** `broker` + `tui` — live monitoring; this is the flagship experience and requires Phase 2 working first.
4. **Phase 4 target:** `report` + `export` — offline analysis layer; safe to defer since it only reads from the already-built store.

---

## Technology Mapping

| Concern | Crate | Notes |
|---------|-------|-------|
| Async runtime | `tokio` | Multi-thread scheduler; `#[tokio::main]` entry point |
| HTTP server | `axum` 0.8 | Announced Jan 2025; current stable |
| Static file serving | `axum::routing::get` + `tower_http::services::ServeDir` | Built into tower-http |
| Header extraction | `axum-extra::TypedHeader` | Typed access to standard headers |
| Client IP extraction | `axum-client-ip` | Handles X-Forwarded-For, proxy headers |
| SQLite async bridge | `tokio-rusqlite` | Background thread + mpsc/oneshot pattern |
| TUI framework | `ratatui` + `crossterm` | Crossterm for cross-platform terminal backend |
| Async TUI events | `crossterm::event::EventStream` + `tokio::select!` | Standard ratatui async pattern |
| Event fan-out | `tokio::sync::broadcast` | DB writer + TUI as independent consumers |
| CB→broker handoff | `tokio::sync::mpsc` | Axum handler is producer; broker task is consumer |
| CLI parsing | `clap` (derive) | Subcommand enum pattern |
| Template rendering | `minijinja` or `tera` | Compile-time template embedding via `rust-embed` |
| Asset embedding | `rust-embed` | Embeds templates + catalog into binary at compile time |

---

## Scalability Considerations

HoneyPrompt is a single-researcher tool, not a multi-tenant service. Expected concurrency: occasional bursts from an agent triggering a handful of callbacks per crawl session.

| Concern | At research scale (1-100 callbacks/session) | If repurposed for higher load |
|---------|---------------------------------------------|-------------------------------|
| SQLite writes | Serialize without contention | Connection pool (deadpool-sqlite) |
| Broadcast channel | 1024 capacity more than adequate | Increase capacity; add back-pressure |
| TUI frame rate | 60 FPS without issue | Rate-limit render ticks on slow terminals |
| Static file serving | Filesystem reads are fast | Preload into memory with rust-embed |

---

## Sources

- [Ratatui async template structure](https://ratatui.github.io/async-template/02-structure.html) — HIGH confidence, official Ratatui docs
- [Ratatui async event stream tutorial](https://ratatui.rs/tutorials/counter-async-app/async-event-stream/) — HIGH confidence, official Ratatui docs
- [Axum 0.8.0 announcement](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) — HIGH confidence, official Tokio blog
- [tokio-rusqlite docs](https://docs.rs/tokio-rusqlite/latest/tokio_rusqlite/) — HIGH confidence, docs.rs official
- [axum-client-ip crate](https://crates.io/crates/axum-client-ip) — MEDIUM confidence, community crate
- [Tokio channels tutorial](https://tokio.rs/tokio/tutorial/channels) — HIGH confidence, official Tokio docs
- [Multi-interface Rust app with Ratatui + Axum](https://dev.to/sebyx07/building-a-multi-interface-todo-app-with-rust-ratatui-and-axum-1cke) — MEDIUM confidence, community article
- [Hexagonal architecture in Rust](https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust) — MEDIUM confidence, community guide
- [rust-embed crate](https://crates.io/crates/rust-embed) — MEDIUM confidence, widely-used community crate
- [axum-extra TypedHeader](https://docs.rs/axum-extra/latest/axum_extra/struct.TypedHeader.html) — HIGH confidence, docs.rs official
