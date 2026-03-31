use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{ConnectInfo, Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::get;
use axum::Router;
use tokio::sync::mpsc;
use tower_http::services::ServeDir;

use crate::config::Config;
use crate::crawler_catalog::CrawlerCatalog;
use crate::types::{NonceMapping, RawCallbackEvent};

/// Metadata stored in memory for each nonce, derived from callback-map.json at startup.
pub struct NonceMeta {
    pub tier: u8,
    pub payload_id: String,
    pub embedding_loc: String,
}

/// Shared application state passed to every Axum handler via Arc<AppState>.
pub struct AppState {
    pub callback_tx: mpsc::Sender<RawCallbackEvent>,
    pub nonce_map: HashMap<String, NonceMeta>,
    pub crawler_catalog: CrawlerCatalog,
}

/// Axum handler for GET /cb/v1/{nonce}.
///
/// Always returns 204 No Content (D-03: never reveal validation status).
/// Valid nonces trigger the event pipeline via the mpsc channel.
///
/// SRV-07: No body extractors — only Path, State, ConnectInfo, and HeaderMap are used.
pub async fn callback_handler(
    AxumPath(nonce): AxumPath<String>,
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> StatusCode {
    // Validate nonce format: exactly 16 lowercase hex chars (D-03: fail silently)
    let valid_format = nonce.len() == 16
        && nonce
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase());
    if !valid_format {
        return StatusCode::NO_CONTENT; // D-03: never reveal validation
    }

    // Look up nonce in the in-memory map
    let meta = match state.nonce_map.get(&nonce) {
        Some(m) => m,
        None => return StatusCode::NO_CONTENT, // D-03: unknown nonce, silent 204
    };

    // Extract fingerprint (pure function — no I/O)
    let fingerprint = crate::fingerprint::extract(peer_addr.ip(), &headers);

    // Classify the user agent
    let classification = state.crawler_catalog.classify(&fingerprint.user_agent);

    // Assemble the raw callback event
    let event = RawCallbackEvent {
        nonce,
        tier: meta.tier,
        payload_id: meta.payload_id.clone(),
        embedding_loc: meta.embedding_loc.clone(),
        fingerprint,
        classification,
        received_at: now_unix_secs(),
    };

    // Non-blocking send — drop if channel is full (best-effort delivery)
    let _ = state.callback_tx.try_send(event);

    StatusCode::NO_CONTENT
}

/// Build the Axum router given an AppState and output directory.
///
/// Extracted as a public function so integration tests can use it without binding a port.
pub fn build_router(state: Arc<AppState>, output_dir: PathBuf) -> Router {
    Router::new()
        .route("/cb/v1/{nonce}", get(callback_handler))
        .fallback_service(ServeDir::new(output_dir))
        .with_state(state)
}

/// Start the honeyprompt HTTP server.
///
/// Loads callback-map.json from `{project_path}/output/callback-map.json`, opens the
/// SQLite DB, wires up the event pipeline (broker + DB writer + stdout logger), and
/// serves on the configured bind address.
pub async fn serve(config: &Config, project_path: &Path, json_mode: bool) -> anyhow::Result<()> {
    let output_dir = project_path.join("output");
    let db_path = project_path.join(".honeyprompt").join("events.db");

    // Load callback-map.json and build in-memory nonce lookup
    let callback_map_path = output_dir.join("callback-map.json");
    let json_str = std::fs::read_to_string(&callback_map_path)
        .map_err(|e| anyhow::anyhow!("Failed to read callback-map.json: {}", e))?;
    let mappings: Vec<NonceMapping> = serde_json::from_str(&json_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse callback-map.json: {}", e))?;

    let mut nonce_map: HashMap<String, NonceMeta> = HashMap::new();
    for m in &mappings {
        nonce_map.insert(
            m.nonce.clone(),
            NonceMeta {
                tier: u8::from(m.tier),
                payload_id: m.payload_id.clone(),
                embedding_loc: m.embedding_location.to_string(),
            },
        );
    }
    let nonce_count = nonce_map.len();

    // Load crawler catalog
    let crawler_catalog = CrawlerCatalog::load()?;

    // Open tokio-rusqlite connection and run migrations
    let conn = tokio_rusqlite::Connection::open(&db_path).await?;
    conn.call(|c| crate::store::run_migrations(c).map_err(tokio_rusqlite::Error::from))
        .await?;

    // Create event pipeline channels
    let (callback_tx, callback_rx) = mpsc::channel::<RawCallbackEvent>(256);
    let (event_tx, _) = tokio::sync::broadcast::channel(1024);

    // Subscribe consumers before spawning broker (so no events are missed)
    let db_rx = event_tx.subscribe();
    let log_rx = event_tx.subscribe();

    // Spawn pipeline tasks
    tokio::spawn(crate::broker::broker_task(callback_rx, event_tx));
    tokio::spawn(crate::broker::db_writer_task(db_rx, conn.clone()));
    tokio::spawn(crate::broker::stdout_logger_task(log_rx, json_mode));

    // Build router
    let app_state = AppState {
        callback_tx,
        nonce_map,
        crawler_catalog,
    };
    let app = build_router(Arc::new(app_state), output_dir);

    // Print startup info (D-09)
    println!("honeyprompt serve");
    println!("  bind:   {}", config.bind_address);
    println!("  nonces: {}", nonce_count);
    println!("  db:     {}", db_path.display());
    println!("  ready");

    // Bind and serve with graceful shutdown (D-11, Pitfall 1: must use with_connect_info)
    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    // Post-shutdown summary stats (D-11)
    let detections = conn
        .call(|c| crate::store::count_detections(c).map_err(tokio_rusqlite::Error::from))
        .await?;
    println!("\nShutdown complete. {} detection(s) recorded.", detections);

    Ok(())
}

/// Wait for Ctrl+C and print a shutdown notice to stderr.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    eprintln!("\nShutting down -- flushing writes...");
}

/// Return current Unix epoch in whole seconds.
fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
