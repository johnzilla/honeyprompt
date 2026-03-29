# Phase 2: Server and Detection - Research

**Researched:** 2026-03-28
**Domain:** Rust async HTTP server (Axum 0.8), event broker (tokio channels), agent fingerprinting, known-crawler suppression, session-based detection
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Event pipeline**
- D-01: Broadcast architecture: Axum callback handler → mpsc → Event Broker → broadcast → DB Writer + stdout logger. Each consumer receives every event independently.
- D-02: Fingerprint extraction: UA string, source IP (proxy-aware), all HTTP headers, ASN/provider lookup from bundled catalog, request timestamp + response latency.
- D-03: Callback endpoint returns 204 unconditionally — no body reflection, strict schema validation on path (nonce = alphanumeric, fixed length).

**Crawler catalog**
- D-04: Known-crawler catalog stored as embedded TOML via rust-embed, same pattern as payload catalog. Users can override with local file.
- D-05: UA-primary identification: User-Agent match triggers "known_crawler" label. IP/ASN match adds confidence annotation but doesn't override.
- D-06: Three-tier classification: "known_crawler" (GPTBot, ClaudeBot, Googlebot, etc.), "known_agent" (identifiable autonomous agent), "unknown" (unclassified).

**Session model**
- D-07: Session ID = hash(source IP + User-Agent) with time window expiry (e.g., 30 min gap = new session).
- D-08: Detection counting is per-session per-tier. Same agent firing Tier 1 and Tier 2 = 2 detections. Same agent firing Tier 1 ten times = 1 detection.

**Serve UX**
- D-09: Detailed startup output: bind address, loaded payloads count, nonce count, DB path, then "ready".
- D-10: Structured log lines by default (one line per event: timestamp, tier, classification, source IP, UA snippet). `--json` flag for JSON lines output.
- D-11: Graceful shutdown on Ctrl+C: finish in-flight requests, flush DB writes, print summary stats, then exit.

### Claude's Discretion
- Event broker task implementation details (tokio::spawn patterns)
- Exact ASN catalog contents and IP range data structure
- Session timeout duration (30 min suggested but flexible)
- Stdout log line format details
- Error handling for malformed callback paths

### Deferred Ideas (OUT OF SCOPE)
- TLS fingerprinting — user selected during scoping but explicitly v2 per PROJECT.md Out of Scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CLI-03 | User can run `honeyprompt serve` to start HTTP server for honeypot + callbacks | Axum 0.8 `serve()` + `with_graceful_shutdown()`, Clap subcommand enum extension |
| SRV-01 | Serve mode hosts honeypot page and callback listener on same port | `Router::fallback_service(ServeDir)` + explicit callback route at `/cb/v1/:nonce` on same Router |
| SRV-03 | Agent fingerprinting extracts UA, IP/ASN, and HTTP headers from callbacks | `axum-client-ip` 1.3.1 `ClientIp` extractor, `axum-extra::TypedHeader<headers::UserAgent>`, `HeaderMap` extraction, pure `fingerprint::extract()` function |
| SRV-04 | Known-agent lookup enriches captures with provider names (GPTBot → OpenAI, etc.) | Embedded TOML crawler catalog via `rust-embed`, UA-primary matching |
| SRV-05 | Known crawler suppression separates indexing traffic from compliance detections | Three-tier `AgentClass` enum: `KnownCrawler`, `KnownAgent`, `Unknown`; stored in events table |
| SRV-06 | Detection counting uses sessions (visits), not raw callback rows | Session ID = `sha256(ip + ua)` truncated to hex, 30-min time window expiry tracked in SQLite |
| SRV-07 | Metadata-only mode stores only path/query/headers/connection metadata (no body) | Request body is never read; body discarded at handler entry; metadata-only is default |
</phase_requirements>

---

## Summary

Phase 2 builds the async server layer on top of Phase 1's static generation pipeline. The core work is: (1) wiring a `serve` subcommand that starts a Tokio-backed Axum server serving the static honeypot files and callback endpoint on the same port, (2) implementing an event pipeline from the callback handler through an mpsc channel to an Event Broker task that fans out via broadcast to a DB writer and stdout logger, (3) agent fingerprinting as a pure function over request parts, and (4) known-crawler suppression and session-based detection counting stored in SQLite.

All Phase 1 infrastructure is reusable without modification: the `rusqlite` schema already has `session_id`, `is_replay`, `fire_count`, `remote_addr`, `user_agent`, and `extra_headers` columns. `Config` already has `bind_address` and `callback_base_url`. The `callback-map.json` produced by `generate` is the nonce lookup source — the server loads it at startup. The only missing infrastructure is tokio + axum + tokio-rusqlite in Cargo.toml and three new modules: `server`, `broker`, `fingerprint`.

The key architectural constraint is using `tokio-rusqlite` (not raw `rusqlite`) for all async database writes. The existing `src/store/mod.rs` uses synchronous `rusqlite::Connection` — the DB writer task wraps it via `tokio_rusqlite::Connection` or bridges with `spawn_blocking`. New insertion functions must be async-compatible.

**Primary recommendation:** Use Axum 0.8 `Router` with an explicit `/cb/v1/:nonce` GET route + `fallback_service(ServeDir::new(...))` for static files. Wire the event pipeline as three `tokio::spawn`-ed tasks (broker, db_writer, stdout_logger) sharing a `broadcast` sender. The Fingerprinter is a synchronous pure function called inside the Axum handler before sending to the channel.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.50.0 | Async runtime (multi-thread) | Required for concurrent HTTP + DB + channels; `#[tokio::main]` entry point |
| axum | 0.8.8 | HTTP framework | Current stable (Dec 2025); ergonomic extractors; tower/tower-http ecosystem |
| tower-http | 0.6.8 | Static file serving via `ServeDir` | Official Axum companion; no extra crate needed for file serving |
| tokio-rusqlite | 0.7.0 | Async SQLite bridge | Only correct way to use rusqlite in async context; background thread + channels |
| axum-client-ip | 1.3.1 | Proxy-aware IP extraction | `ClientIp` extractor handles X-Forwarded-For, CF-Connecting-IP etc. |
| axum-extra | 0.12.5 | `TypedHeader` for UA extraction | Official axum-extra; typed access to User-Agent and other headers |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio::sync::broadcast | (in tokio) | Event fan-out to DB writer + stdout logger | DB writer and logger are independent consumers needing every event |
| tokio::sync::mpsc | (in tokio) | Axum handler → Event Broker handoff | Single producer (handler) to single consumer (broker) |
| serde_json | 1.0.149 | JSON log output (`--json` flag), callback-map.json parsing | Already in Cargo.toml |
| rust-embed | 8.11 | Embed crawler catalog TOML into binary | Already used for payload catalog; same pattern |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| axum-client-ip `ClientIp` | Manual `X-Forwarded-For` header parse | axum-client-ip handles edge cases (rightmost-trustworthy IP logic); manual parse has known pitfalls |
| tokio-rusqlite | deadpool-sqlite | tokio-rusqlite is simpler for single-writer; deadpool adds connection pool complexity not needed at this scale |
| broadcast channel | Two mpsc channels | broadcast is cleaner for N independent consumers; two mpsc channels require duplication |

**Installation (additions to Cargo.toml):**
```toml
tokio = { version = "1.50", features = ["full"] }
axum = { version = "0.8", features = ["tokio"] }
tower-http = { version = "0.6", features = ["fs"] }
tokio-rusqlite = "0.7"
axum-client-ip = "1.3"
axum-extra = { version = "0.12", features = ["typed-header"] }
```

**Version verification:** Confirmed against crates.io registry on 2026-03-28:
- `axum`: 0.8.8 (updated 2025-12-20)
- `tokio`: 1.50.0 (updated 2026-03-03)
- `tokio-rusqlite`: 0.7.0 (updated 2025-11-16)
- `axum-client-ip`: 1.3.1 (updated 2026-01-22, 105K downloads)
- `tower-http`: 0.6.8 (updated 2025-12-08)
- `axum-extra`: 0.12.5 (updated 2025-12-27)

---

## Architecture Patterns

### Recommended Module Structure
```
src/
├── server/
│   └── mod.rs       # Axum router, serve() entry point, graceful shutdown
├── broker/
│   └── mod.rs       # Event Broker task, broadcast fan-out, AppEvent type
├── fingerprint/
│   └── mod.rs       # Pure fn extract(parts) -> AgentFingerprint
├── crawler_catalog/
│   └── mod.rs       # Embedded TOML catalog, UA matching, classify()
├── store/
│   └── mod.rs       # EXISTING — add insert_callback_event() async fn
├── cli/
│   └── mod.rs       # EXISTING — add Serve subcommand + ServeArgs
├── types.rs         # EXISTING — add AgentClass, CallbackEvent, AgentFingerprint
└── lib.rs           # EXISTING — add mod server; mod broker; mod fingerprint; mod crawler_catalog;
```

### Pattern 1: Axum Router — Static Files + Callback Route on Same Port
**What:** One Router with an explicit `/cb/v1/:nonce` GET handler and `fallback_service(ServeDir)` for all other paths.
**When to use:** All serve mode. This gives callback priority over static files while still serving honeypot HTML, robots.txt, etc.
**Example:**
```rust
// Source: https://docs.rs/axum/0.8.8/axum/, official static-file-server example
use axum::{Router, routing::get, extract::{Path, State}};
use tower_http::services::ServeDir;
use std::sync::Arc;

async fn callback_handler(
    Path(nonce): Path<String>,
    State(state): State<Arc<AppState>>,
    ClientIp(ip): ClientIp,
    // headers extracted inline via HeaderMap extractor
) -> impl IntoResponse {
    // validate nonce, assemble CallbackEvent, send to broker
    StatusCode::NO_CONTENT
}

let app = Router::new()
    .route("/cb/v1/:nonce", get(callback_handler))
    .fallback_service(ServeDir::new(&output_dir));
```

### Pattern 2: ConnectInfo for Direct Connection IP
**What:** `ConnectInfo<SocketAddr>` extracts the raw TCP peer address. Used as fallback when no proxy headers are present.
**When to use:** When `axum-client-ip` finds no trusted proxy header.
**Critical:** Must use `into_make_service_with_connect_info::<SocketAddr>()` not `into_make_service()`.
```rust
// Source: https://docs.rs/axum/0.8.8/axum/extract/struct.ConnectInfo.html
axum::serve(
    listener,
    app.into_make_service_with_connect_info::<SocketAddr>()
).await?;
```

### Pattern 3: Event Broker with mpsc → broadcast Fan-Out
**What:** Axum handler sends to `mpsc::Sender<CallbackEvent>`. Broker task receives, enriches with fingerprint, broadcasts `AppEvent` to all consumers.
**When to use:** Phase 2 event pipeline (DB writer + stdout logger). Phase 3 TUI will add a third broadcast receiver.

```rust
// Source: https://tokio.rs/tokio/tutorial/channels (official Tokio docs)
use tokio::sync::{mpsc, broadcast};

// In AppState:
pub struct AppState {
    pub callback_tx: mpsc::Sender<RawCallbackEvent>,
    // broadcast_tx held by broker task, receivers given to consumers
}

// Broker task (tokio::spawn):
async fn broker_task(
    mut callback_rx: mpsc::Receiver<RawCallbackEvent>,
    event_tx: broadcast::Sender<AppEvent>,
) {
    while let Some(raw) = callback_rx.recv().await {
        let fingerprint = fingerprint::extract(&raw);
        let classification = crawler_catalog::classify(&fingerprint);
        let event = AppEvent { raw, fingerprint, classification };
        let _ = event_tx.send(event); // ignore lagged receiver errors
    }
}
```

### Pattern 4: tokio-rusqlite Async DB Write
**What:** DB writer task wraps the existing synchronous SQLite write inside `conn.call(|conn| { ... })`.
**When to use:** Any async context that needs to write to SQLite.
```rust
// Source: https://docs.rs/tokio-rusqlite/0.7.0/tokio_rusqlite/
use tokio_rusqlite::Connection;

let conn = Connection::open(&db_path).await?;
conn.call(|conn| {
    conn.execute(
        "INSERT OR REPLACE INTO events (...) VALUES (?1, ?2, ?3, ...)",
        params![nonce, tier, session_id, ...],
    )?;
    Ok(())
}).await?;
```

### Pattern 5: Graceful Shutdown with Ctrl+C
**What:** `axum::serve(...).with_graceful_shutdown(shutdown_signal())` where `shutdown_signal()` awaits `tokio::signal::ctrl_c()`.
**When to use:** Server startup in `serve` subcommand.
```rust
// Source: https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
}

axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
    .with_graceful_shutdown(shutdown_signal())
    .await?;
```

### Pattern 6: Session ID Computation
**What:** Session ID is a deterministic hash of source IP + User-Agent, used for deduplication and session-based counting.
**When to use:** Every callback event before DB insert.
```rust
// Claude's discretion: use std sha256 or siphasher; no new crate needed
// sha2 crate (already usable) or std hash
fn compute_session_id(ip: &str, ua: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    ip.hash(&mut h);
    ua.hash(&mut h);
    format!("{:016x}", h.finish())
}
```
Note: For research tool security, using a stable hash (sha256 truncated) is better than DefaultHasher. `sha2` crate or manual SHA-256 via `getrandom`-produced salt to make session IDs non-guessable is preferred.

### Pattern 7: Nonce Validation (strict schema)
**What:** Validate the nonce path segment before any DB lookup. Nonce must be exactly 16 lowercase hex characters. Anything else returns 400 (or 204 — per D-03 "204 unconditionally"). Decision D-03 says 204 always; this means even invalid nonces return 204 to avoid revealing the validation rule to probers.
**When to use:** Top of callback handler.
```rust
fn is_valid_nonce(s: &str) -> bool {
    s.len() == 16 && s.chars().all(|c| c.is_ascii_hexdigit() && c.is_lowercase())
}
// If !is_valid_nonce(&nonce) { return StatusCode::NO_CONTENT; }
```

### Anti-Patterns to Avoid
- **Writing to SQLite inside the Axum handler:** Blocks the handler on I/O; SQLite writes are serialized — creates bottleneck. Use mpsc send → broker → DB writer task.
- **Using `Arc<Mutex<rusqlite::Connection>>` in async context:** `rusqlite::Connection` is not `Sync`; this fails at runtime. Use `tokio-rusqlite::Connection`.
- **Calling `axum::serve(listener, app.into_make_service())` when using ConnectInfo:** ConnectInfo requires `into_make_service_with_connect_info::<SocketAddr>()` — using the wrong method causes a runtime panic.
- **Reading or storing the callback request body:** Violates metadata-only mode. Handler must never read the body. Drop the request at extraction layer.
- **Reflecting nonce or any request data in the 204 response:** Per D-03, return empty 204 with no body.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Proxy-aware IP extraction | Parse X-Forwarded-For manually | `axum-client-ip` 1.3.1 `ClientIp` | Rightmost-trustworthy IP logic has edge cases (IPv6, chained proxies, CF headers) |
| Static file serving | Custom file-read handler | `tower_http::services::ServeDir` | Handles mime types, range requests, ETag, directory index; already in tower-http |
| Async SQLite bridge | `Arc<Mutex<Connection>>` or `spawn_blocking` inline | `tokio-rusqlite` | Background thread + oneshot channels; the only correct pattern; has documented API |
| Graceful shutdown signal | Manual `ctrlc` crate | `tokio::signal::ctrl_c()` | Built into tokio; works on both Unix (SIGTERM too) and Windows |
| Event fan-out | Shared `Arc<Mutex<Vec<Event>>>` | `tokio::sync::broadcast` | No lock contention; each consumer owns its receiver; lagged receiver is explicit error |

**Key insight:** The async bridging problem for rusqlite is subtle — it's not just "use spawn_blocking". `tokio-rusqlite` provides a stable, idiomatic API with proper error types. Anything hand-rolled will either be unsound or require re-implementing the same background-thread pattern.

---

## Common Pitfalls

### Pitfall 1: Using `into_make_service()` instead of `into_make_service_with_connect_info()`
**What goes wrong:** `ConnectInfo<SocketAddr>` extractor panics at runtime with a confusing error about missing extension.
**Why it happens:** Axum's ConnectInfo requires the connection info to be injected by the service factory, which only happens with `into_make_service_with_connect_info`.
**How to avoid:** Always use `app.into_make_service_with_connect_info::<SocketAddr>()` when any handler uses `ConnectInfo`.
**Warning signs:** Runtime panic mentioning "missing extension" or "ConnectInfo" on first request.

### Pitfall 2: UA-Primary Classification Over-Trust (from PITFALLS.md Pitfall 3)
**What goes wrong:** GPTBot/ClaudeBot fire Tier 1 callbacks because they index the page. If classified as "agent_complied" instead of "known_crawler", reports show false detections.
**Why it happens:** UA string says "GPTBot" but the behavior (following a URL) is normal crawler behavior.
**How to avoid:** D-05 and D-06 are mandatory: UA match → "known_crawler" first, never "agent_complied". Only classify as "agent_complied" when proof tier >= 2 AND UA is not in the known-crawler list.
**Warning signs:** High Tier 1 hit rate from OpenAI/Anthropic/Google ASNs within hours of deployment.

### Pitfall 3: Session ID Based on IP+UA Without Timestamp Window
**What goes wrong:** If an agent returns days later it gets the same session_id as its prior visit. Detection count = 1 for two independent visits.
**Why it happens:** Hash(IP+UA) is deterministic; without a time window, all visits by the same agent are one "session".
**How to avoid:** D-07 requires time window expiry. Store `last_seen_at` per session. If `now - last_seen_at > 30min`, treat as new session (new session_id generation via timestamp-bucketed hash or UUID).
**Warning signs:** Single session accumulating `fire_count` > expected across many hours.

### Pitfall 4: broadcast::Receiver Lag — Events Dropped in DB Writer
**What goes wrong:** If the DB writer task falls behind, `broadcast::Receiver::recv()` returns `Err(RecvError::Lagged(n))`. Default behavior: events are silently lost.
**Why it happens:** SQLite writes can block briefly; if a burst of callbacks arrives, the channel fills (capacity 1024) and old messages are overwritten.
**How to avoid:** In the DB writer task, handle `Err(RecvError::Lagged(n))` explicitly — log the lag count. In testing, ensure the broadcast capacity (1024) exceeds realistic burst sizes. At research scale (< 100 callbacks/session) this is not a real risk.
**Warning signs:** `RecvError::Lagged` in DB writer logs.

### Pitfall 5: `axum-client-ip` API Changed in v1.0 (Breaking)
**What goes wrong:** Documentation or examples using `SecureClientIp` or `InsecureClientIp` fail to compile — these types were removed in v1.0.
**Why it happens:** axum-client-ip 1.x renamed `SecureClientIp` to `ClientIp` and removed `InsecureClientIp`.
**How to avoid:** Use `ClientIp` from axum-client-ip 1.3.1. Configure `ClientIpSource` in `AppState` for the appropriate header (or use specific extractors like `XRealIp`).
**Warning signs:** Compile error mentioning `SecureClientIp` or `InsecureClientIp` not found.

### Pitfall 6: Metadata-Only Mode Default Not Enforced
**What goes wrong:** Axum handlers implicitly buffer the request body if a `Bytes` or `String` extractor is present. Even without storing it, reading it violates the safety model.
**Why it happens:** Accidental body extraction.
**How to avoid:** Callback handler must use only: `Path<String>`, `ConnectInfo<SocketAddr>`, `ClientIp`, `HeaderMap`, `State`. Never extract `Bytes`, `String`, `Json<T>`, or `Form<T>` from the callback endpoint.
**Warning signs:** Compiler allows it; runtime no-op reading is invisible. Code review gate required.

---

## Code Examples

Verified patterns from official sources:

### Callback Handler (skeleton)
```rust
// Sources:
//   https://docs.rs/axum/0.8.8/axum/
//   https://docs.rs/axum-client-ip/1.3.1/axum_client_ip/
//   https://docs.rs/axum/0.8.8/axum/extract/struct.ConnectInfo.html
use axum::{
    extract::{Path, State, ConnectInfo},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use axum_client_ip::ClientIp;
use std::{net::SocketAddr, sync::Arc};

pub async fn callback_handler(
    Path(nonce): Path<String>,
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    ClientIp(client_ip): ClientIp,
    headers: HeaderMap,
) -> impl IntoResponse {
    // 1. Validate nonce format (strict: 16 lowercase hex chars)
    if !is_valid_nonce(&nonce) {
        return StatusCode::NO_CONTENT; // D-03: 204 always
    }

    // 2. Look up nonce in callback-map (loaded at startup)
    let Some(nonce_meta) = state.nonce_map.get(&nonce) else {
        return StatusCode::NO_CONTENT; // unknown nonce → 204
    };

    // 3. Fingerprint (pure function, no side effects)
    let fingerprint = fingerprint::extract(peer_addr, client_ip, &headers);

    // 4. Classify (pure function, no side effects)
    let classification = state.crawler_catalog.classify(&fingerprint.user_agent);

    // 5. Assemble event and send to broker (non-blocking)
    let event = RawCallbackEvent { nonce, nonce_meta, fingerprint, classification, received_at: now() };
    let _ = state.callback_tx.try_send(event); // drop if channel full (not block)

    StatusCode::NO_CONTENT // D-03: unconditional 204
}
```

### DB Write via tokio-rusqlite
```rust
// Source: https://docs.rs/tokio-rusqlite/0.7.0/tokio_rusqlite/
use tokio_rusqlite::Connection;
use rusqlite::params;

pub async fn insert_callback_event(
    conn: &Connection,
    event: &AppEvent,
) -> tokio_rusqlite::Result<()> {
    let nonce = event.raw.nonce.clone();
    let tier = event.raw.nonce_meta.tier;
    let session_id = event.session_id.clone();
    let remote_addr = event.raw.fingerprint.source_ip.to_string();
    let user_agent = event.raw.fingerprint.user_agent.clone();
    let headers_json = serde_json::to_string(&event.raw.fingerprint.headers).unwrap_or_default();
    let classification = format!("{:?}", event.classification);
    let now_ts = now_unix_secs();

    conn.call(move |conn| {
        // Upsert: if same nonce seen before, increment fire_count and set is_replay=1
        conn.execute(
            "INSERT INTO events (nonce, tier, payload_id, embedding_loc, first_seen_at, last_seen_at, fire_count, is_replay, session_id, remote_addr, user_agent, extra_headers)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5, 1, 0, ?6, ?7, ?8, ?9)
             ON CONFLICT(nonce) DO UPDATE SET
               last_seen_at = ?5,
               fire_count = fire_count + 1,
               is_replay = 1",
            params![
                nonce, tier as i64, "payload_id_here", "embedding_loc_here",
                now_ts.to_string(), session_id, remote_addr, user_agent, headers_json
            ],
        )?;
        Ok(())
    }).await
}
```

### Fingerprint Extraction (pure function)
```rust
// Pure function — no async, no I/O, no side effects
pub struct AgentFingerprint {
    pub source_ip: std::net::IpAddr,
    pub user_agent: String,     // ANSI-stripped
    pub headers: std::collections::HashMap<String, String>,
    pub received_at: u64,       // unix seconds
}

pub fn extract(
    peer_addr: std::net::SocketAddr,
    client_ip: std::net::IpAddr,
    headers: &axum::http::HeaderMap,
) -> AgentFingerprint {
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(strip_ansi)   // Pitfall 4 mitigation: strip ANSI sequences
        .unwrap_or_default()
        .to_string();

    let header_map: std::collections::HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| {
            let val = v.to_str().ok()?.to_string();
            Some((k.as_str().to_string(), strip_ansi(&val).to_string()))
        })
        .collect();

    AgentFingerprint {
        source_ip: client_ip, // proxy-aware (from ClientIp extractor)
        user_agent,
        headers: header_map,
        received_at: now_unix_secs(),
    }
}

fn strip_ansi(s: &str) -> &str {
    // Strip ESC sequences before storing any agent-supplied string
    // Simple implementation: reject/strip any string containing ESC (0x1b)
    // Full implementation uses regex or strip_ansi_escapes crate
    s // placeholder — planner should add strip_ansi_escapes crate or regex
}
```

### Known-Crawler Classification
```rust
// Embedded TOML catalog loaded at startup via rust-embed
// Same pattern as payload catalog in Phase 1

#[derive(Debug, Clone, PartialEq)]
pub enum AgentClass {
    KnownCrawler { provider: String },  // D-06: GPTBot → "OpenAI"
    KnownAgent { provider: String },    // identifiable autonomous agent
    Unknown,
}

pub fn classify(user_agent: &str, asn_hint: Option<&str>) -> AgentClass {
    // D-05: UA-primary. ASN adds confidence but doesn't override.
    if let Some(entry) = KNOWN_CRAWLERS.iter().find(|e| user_agent.contains(e.ua_fragment)) {
        return AgentClass::KnownCrawler { provider: entry.provider.to_string() };
    }
    if let Some(entry) = KNOWN_AGENTS.iter().find(|e| user_agent.contains(e.ua_fragment)) {
        return AgentClass::KnownAgent { provider: entry.provider.to_string() };
    }
    AgentClass::Unknown
}
```

### Server Startup with Graceful Shutdown
```rust
// Source: https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
pub async fn serve(cfg: &Config, project_dir: &Path) -> anyhow::Result<()> {
    // Startup output (D-09)
    println!("honeyprompt serve");
    println!("  bind:    {}", cfg.bind_address);
    println!("  db:      {}", db_path.display());
    println!("  ready");

    let listener = tokio::net::TcpListener::bind(&cfg.bind_address).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>()
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    // Post-shutdown summary stats (D-11)
    print_summary_stats(&conn).await;
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    println!("\nShutting down — flushing writes...");
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `SecureClientIp` / `InsecureClientIp` | `ClientIp` (renamed) | axum-client-ip v1.0 | Old names don't exist; docs/examples using old names will fail to compile |
| `axum::Server::bind()` (axum 0.6) | `axum::serve(listener, app)` | axum 0.7+ | Old `Server::bind` API removed; must use `tokio::net::TcpListener` + `axum::serve()` |
| `rusqlite` directly in async handlers | `tokio-rusqlite` | Ongoing best practice | Direct use of `rusqlite::Connection` in async fails to compile (`!Sync`) |

**Deprecated/outdated:**
- `axum::Server::bind(...)`: Removed in axum 0.7; replaced by `TcpListener::bind(...).await?` + `axum::serve()`
- `InsecureClientIp` / `SecureClientIp`: Removed in axum-client-ip 1.0; replaced by `ClientIp`
- `chrono_now()` returning raw unix seconds: Phase 2 should consider ISO-8601 format for timestamps (the existing `chrono_now()` returns a raw integer string — adequate for Phase 2 but worth noting for Phase 3/4 compatibility)

---

## Open Questions

1. **Per-visitor nonce injection vs. static nonces**
   - What we know: Phase 1 generates static nonces at `generate` time. PITFALLS.md Pitfall 2 identifies per-visitor nonces as the correct approach. SUMMARY.md explicitly flags this as unresolved.
   - What's unclear: Does Phase 2 serve `index.html` statically (nonces fixed at generate time) or dynamically inject fresh nonces per request?
   - Recommendation: For Phase 2, serve statically. Static nonces are replay-detected by the `is_replay` flag already in the schema. Per-visitor nonce injection is a Phase 3 enhancement (requires replacing `ServeDir` with a dynamic handler for `index.html`). Document this limitation in the serve startup output as a warning: "Static nonces — replay detection active but per-visitor nonce injection not enabled."

2. **Session ID hash function**
   - What we know: D-07 says hash(IP + UA); `getrandom` and `hex` are already in Cargo.toml.
   - What's unclear: `DefaultHasher` is not stable across Rust versions. SHA-256 requires `sha2` crate or manual implementation.
   - Recommendation: Use `sha2` crate (industry standard, ~10KB) or the existing `hex` + `getrandom` to generate a keyed HMAC. Adding `sha2 = "0.10"` is the cleanest option and avoids `DefaultHasher` instability.

3. **ANSI escape stripping**
   - What we know: PITFALLS.md Pitfall 4 requires stripping ANSI escapes from all agent-supplied strings before TUI/log output.
   - What's unclear: Whether a new crate is needed or a simple regex/char filter is sufficient.
   - Recommendation: Add `strip-ansi-escapes = "0.2"` (lightweight, no dependencies) or implement a simple filter that rejects any byte 0x1b. The crate approach is more robust; add it to Cargo.toml in the same wave as fingerprint module.

4. **ASN catalog contents**
   - What we know: D-02 requires ASN/provider lookup. D-04 says embedded TOML.
   - What's unclear: The actual IP ranges for known AI providers. This data changes as providers add infrastructure.
   - Recommendation: Start with a minimal catalog of known CIDR prefixes for OpenAI (15.177.x.x, 23.102.x.x), Anthropic, Google (34.x.x.x common), Microsoft/Azure. Store as `Vec<(IpNet, provider)>` in embedded TOML. The `ipnet` crate (0.9) provides CIDR matching. Keep this as a data-only update path — no code change needed to add new CIDRs.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All compilation | ✓ | (existing project) | — |
| Cargo (internet) | `cargo add` for new crates | ✓ | Available | — |
| SQLite (bundled) | rusqlite with bundled feature | ✓ | bundled in rusqlite 0.39 | — |
| tokio | New dependency | Not yet in Cargo.toml | Need 1.50 | — |
| axum | New dependency | Not yet in Cargo.toml | Need 0.8.8 | — |
| tower-http | New dependency | Not yet in Cargo.toml | Need 0.6.8 | — |
| tokio-rusqlite | New dependency | Not yet in Cargo.toml | Need 0.7.0 | — |
| axum-client-ip | New dependency | Not yet in Cargo.toml | Need 1.3.1 | Fall back to raw `ConnectInfo<SocketAddr>` only (no proxy support) |
| axum-extra | New dependency | Not yet in Cargo.toml | Need 0.12.5 | Use `headers` map directly |

**Missing dependencies with no fallback:**
- tokio, axum, tower-http, tokio-rusqlite — must be added to Cargo.toml in Wave 0

**Missing dependencies with fallback:**
- axum-client-ip — fallback is `ConnectInfo<SocketAddr>` (direct peer address only, no proxy header support). Acceptable for local testing, insufficient for production deployment.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | none (uses `#[test]` attributes, integration tests in `tests/`) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test --all` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLI-03 | `honeyprompt serve` starts and responds to requests | integration | `cargo test --test test_serve` | ❌ Wave 0 |
| SRV-01 | Same port serves honeypot page GET / and callback GET /cb/v1/:nonce | integration | `cargo test --test test_serve::test_serve_static_and_callback` | ❌ Wave 0 |
| SRV-03 | Fingerprint extraction captures UA, IP, headers | unit | `cargo test --lib fingerprint` | ❌ Wave 0 |
| SRV-04 | Known-agent lookup returns correct provider name | unit | `cargo test --lib crawler_catalog` | ❌ Wave 0 |
| SRV-05 | GPTBot UA classified as KnownCrawler, not Unknown | unit | `cargo test --lib crawler_catalog::test_known_crawler_classification` | ❌ Wave 0 |
| SRV-06 | Session-based counting: same nonce fires 10x → fire_count=10, is_replay=1, 1 session | unit | `cargo test --lib store::test_session_counting` | ❌ Wave 0 |
| SRV-07 | Callback handler never reads request body | unit | `cargo test --lib server::test_no_body_read` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test` (unit tests only, < 5s)
- **Per wave merge:** `cargo test --all` (all unit + integration tests)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/test_serve.rs` — integration tests for SRV-01, CLI-03 (requires tokio in dev-deps)
- [ ] `src/fingerprint/mod.rs` — unit tests for SRV-03
- [ ] `src/crawler_catalog/mod.rs` — unit tests for SRV-04, SRV-05
- [ ] `src/store/mod.rs` additions — unit tests for SRV-06 (insert_callback_event, session counting)
- [ ] `src/server/mod.rs` — unit test for no-body-read assertion (SRV-07)
- [ ] Add `tokio` to `[dev-dependencies]` for async integration tests: `tokio = { version = "1", features = ["macros", "rt-multi-thread"] }`

---

## Project Constraints (from CLAUDE.md)

All actionable directives from `CLAUDE.md`:
- **Language**: Rust — do not introduce non-Rust components
- **CLI**: Clap for argument parsing — `serve` subcommand must use Clap derive pattern
- **HTTP**: Axum or equivalent — Axum 0.8 confirmed
- **Storage**: SQLite via rusqlite — `tokio-rusqlite` wraps rusqlite, satisfies this constraint
- **Platform**: Linux and macOS first — no Windows-specific code
- **Performance**: Fast startup, low memory footprint — no unnecessary allocations in handler hot path
- **Ethics**: All generated content must include visible warnings — callback endpoint must NOT reflect or log content that could help an adversary enumerate the system
- **GSD workflow**: Do not make direct repo edits outside a GSD workflow — planning artifacts must be in sync

---

## Sources

### Primary (HIGH confidence)
- [axum 0.8.8 — docs.rs](https://docs.rs/axum/0.8.8/axum/) — Router, ServeDir fallback, ConnectInfo, State, handler patterns
- [axum ConnectInfo — docs.rs](https://docs.rs/axum/0.8.8/axum/extract/struct.ConnectInfo.html) — `into_make_service_with_connect_info` requirement confirmed
- [axum graceful shutdown example — github.com/tokio-rs/axum](https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs) — `with_graceful_shutdown` + `tokio::signal::ctrl_c()`
- [axum static-file-server example — github.com/tokio-rs/axum](https://github.com/tokio-rs/axum/blob/main/examples/static-file-server/src/main.rs) — `fallback_service(ServeDir)` + API routes on same Router
- [tokio-rusqlite 0.7.0 — docs.rs](https://docs.rs/tokio-rusqlite/0.7.0/tokio_rusqlite/) — `conn.call(|conn| { ... })` async bridge
- [tower-http ServeDir — docs.rs](https://docs.rs/tower-http/0.6.8/tower_http/services/struct.ServeDir.html) — static file serving configuration
- [Tokio channels tutorial — tokio.rs](https://tokio.rs/tokio/tutorial/channels) — broadcast vs. mpsc selection rationale
- [axum-client-ip 1.3.1 — docs.rs](https://docs.rs/axum-client-ip/1.3.1/axum_client_ip/) — `ClientIp` extractor, v1.0 breaking changes confirmed
- [Crates.io version verification — 2026-03-28] — all version numbers confirmed against registry

### Secondary (MEDIUM confidence)
- [.planning/research/ARCHITECTURE.md] — Event Broker architecture, component build layers (HIGH confidence from prior research)
- [.planning/research/PITFALLS.md] — Pitfalls 3, 4, 5, 9 directly applicable to Phase 2
- [.planning/research/SUMMARY.md] — axum-client-ip maintenance status note resolved (1.3.1 confirmed active)
- [Axum graceful shutdown discussion — github.com/tokio-rs/axum/discussions/2565](https://github.com/tokio-rs/axum/discussions/2565) — `with_graceful_shutdown` CancellationToken integration patterns

### Tertiary (LOW confidence)
- [strip-ansi-escapes crate recommendation] — inferred from domain knowledge; not verified against official docs; planner should confirm `strip-ansi-escapes = "0.2"` is current

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions confirmed against crates.io registry 2026-03-28
- Architecture: HIGH — verified against official Axum, Tokio, and tokio-rusqlite docs
- Pitfalls: HIGH — sourced from Phase 1 research PITFALLS.md + confirmed against Axum docs (ConnectInfo panic case, axum-client-ip API change)
- Crawler catalog: MEDIUM — catalog contents (which UA strings, which ASN ranges) are data that needs ongoing maintenance; patterns for embedding/lookup are HIGH

**Research date:** 2026-03-28
**Valid until:** 2026-04-28 (stable libraries; axum-client-ip and tower-http track axum releases so verify if axum is upgraded)
