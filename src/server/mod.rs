use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{ConnectInfo, Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use base64::{engine::general_purpose, Engine as _};
use tokio::sync::mpsc;
use tower_http::services::ServeDir;

use crate::config::Config;
use crate::crawler_catalog::CrawlerCatalog;
use crate::types::{NonceMapping, RawCallbackEvent, T5Formula, Tier};

/// Metadata stored in memory for each nonce, derived from callback-map.json at startup.
pub struct NonceMeta {
    pub tier: u8,
    pub payload_id: String,
    pub embedding_loc: String,
    /// Some only for tier-5 nonces; carries the formula constants from the catalog.
    /// Loaded once at serve() startup by joining catalog payloads against callback-map.json
    /// by payload_id (RESEARCH Q3 — avoids a callback-map schema change).
    pub t5_formula: Option<T5Formula>,
}

/// Shared application state passed to every Axum handler via Arc<AppState>.
pub struct AppState {
    pub conn: tokio_rusqlite::Connection,
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
        t4_capability: None,
        t5_proof: None,
        t5_proof_valid: None,
        // Phase 14: T1 handler does not know about T5 formulas.
        t5_formula: None,
    };

    // Non-blocking send — drop if channel is full (best-effort delivery)
    let _ = state.callback_tx.try_send(event);

    StatusCode::NO_CONTENT
}

/// Axum handler for GET /cb/v4/{nonce}/{b64_payload}.
///
/// D-13-14, D-13-15: Always returns 204 No Content. Any validation failure is silent.
/// On happy path: decodes URL-safe base64, sanitizes (D-13-09), and sends a
/// RawCallbackEvent with `t4_capability = Some(sanitized)`.
pub async fn t4_callback_handler(
    AxumPath((nonce, b64_payload)): AxumPath<(String, String)>,
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> StatusCode {
    // D-13-16: reuse the same nonce validation as /cb/v1/
    if !crate::nonce::is_valid_nonce(&nonce) {
        return StatusCode::NO_CONTENT;
    }
    let meta = match state.nonce_map.get(&nonce) {
        Some(m) => m,
        None => return StatusCode::NO_CONTENT,
    };
    if meta.tier != 4 {
        // Wrong tier for this route — silent 204 (D-13-15)
        return StatusCode::NO_CONTENT;
    }
    let sanitized = match decode_t4_payload(&b64_payload) {
        Some(s) => s,
        None => return StatusCode::NO_CONTENT, // D-13-09/D-13-15: nothing stored
    };

    let fingerprint = crate::fingerprint::extract(peer_addr.ip(), &headers);
    let classification = state.crawler_catalog.classify(&fingerprint.user_agent);

    let event = RawCallbackEvent {
        nonce,
        tier: meta.tier,
        payload_id: meta.payload_id.clone(),
        embedding_loc: meta.embedding_loc.clone(),
        fingerprint,
        classification,
        received_at: now_unix_secs(),
        t4_capability: Some(sanitized),
        t5_proof: None,
        t5_proof_valid: None,
        // Phase 14: T4 handler does not know about T5 formulas.
        t5_formula: None,
    };
    let _ = state.callback_tx.try_send(event);
    StatusCode::NO_CONTENT
}

/// Axum handler for GET /cb/v5/{nonce}/{proof}.
///
/// D-13-02 / D-13-15: Always returns 204 No Content. On happy path: validates the
/// proof is exactly 3 ASCII digits, derives seed from nonce, computes expected proof
/// via `compute_expected_proof`, and sends RawCallbackEvent with `t5_proof_valid`.
pub async fn t5_callback_handler(
    AxumPath((nonce, proof_str)): AxumPath<(String, String)>,
    State(state): State<Arc<AppState>>,
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> StatusCode {
    if !crate::nonce::is_valid_nonce(&nonce) {
        return StatusCode::NO_CONTENT;
    }
    let meta = match state.nonce_map.get(&nonce) {
        Some(m) => m,
        None => return StatusCode::NO_CONTENT,
    };
    if meta.tier != 5 {
        return StatusCode::NO_CONTENT;
    }
    let formula = match meta.t5_formula.as_ref() {
        Some(f) => f,
        None => return StatusCode::NO_CONTENT, // misconfigured catalog — silent 204
    };
    // D-13-02: proof must be exactly 3 ASCII digits (zero-padded)
    if proof_str.len() != 3 || !proof_str.bytes().all(|b| b.is_ascii_digit()) {
        return StatusCode::NO_CONTENT;
    }
    let submitted = match proof_str.parse::<u32>() {
        Ok(p) => p,
        Err(_) => return StatusCode::NO_CONTENT,
    };
    let seed = match crate::nonce::derive_seed(&nonce) {
        Some(s) => s,
        None => return StatusCode::NO_CONTENT,
    };
    let expected = compute_expected_proof(seed, formula.a, formula.b, formula.modulus);
    let proof_valid = submitted == expected;

    let fingerprint = crate::fingerprint::extract(peer_addr.ip(), &headers);
    let classification = state.crawler_catalog.classify(&fingerprint.user_agent);

    let event = RawCallbackEvent {
        nonce,
        tier: meta.tier,
        payload_id: meta.payload_id.clone(),
        embedding_loc: meta.embedding_loc.clone(),
        fingerprint,
        classification,
        received_at: now_unix_secs(),
        t4_capability: None,
        t5_proof: Some(proof_str),
        t5_proof_valid: Some(proof_valid),
        // Phase 14: propagate the server-verified formula so the Monitor detail
        // pane can render `formula=(seed+A)*B % M` (D-14-02). T5Formula is Copy.
        t5_formula: Some(*formula),
    };
    let _ = state.callback_tx.try_send(event);
    StatusCode::NO_CONTENT
}

/// Axum handler for GET /stats.
///
/// Returns aggregate callback statistics as JSON with CORS header.
/// Returns 500 on database error.
pub async fn stats_handler(State(state): State<Arc<AppState>>) -> axum::response::Response {
    match state
        .conn
        .call(|c| crate::store::query_report_summary(c).map_err(tokio_rusqlite::Error::from))
        .await
    {
        Ok(summary) => {
            let mut response = axum::Json(summary).into_response();
            response.headers_mut().insert(
                "access-control-allow-origin",
                axum::http::HeaderValue::from_static("*"),
            );
            response
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Build the Axum router given an AppState and output directory.
///
/// Extracted as a public function so integration tests can use it without binding a port.
pub fn build_router(state: Arc<AppState>, output_dir: PathBuf) -> Router {
    Router::new()
        .route("/cb/v1/{nonce}", get(callback_handler)) // UNCHANGED (D-13-18)
        .route("/cb/v4/{nonce}/{b64_payload}", get(t4_callback_handler)) // NEW (D-13-14)
        .route("/cb/v5/{nonce}/{proof}", get(t5_callback_handler)) // NEW (D-13-14)
        .route("/stats", get(stats_handler))
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

    // Load catalog once for T5 formula lookup (RESEARCH Q3).
    // MUST happen before the nonce_map construction loop below, which depends on
    // this HashMap to populate NonceMeta.t5_formula for tier-5 entries.
    let all_payloads = crate::catalog::load_catalog()?;
    let t5_formulas_by_payload_id: HashMap<String, T5Formula> = all_payloads
        .iter()
        .filter_map(|p| p.t5_formula.map(|f| (p.id.clone(), f)))
        .collect();

    let mut nonce_map: HashMap<String, NonceMeta> = HashMap::new();
    for m in &mappings {
        let t5_formula = if m.tier == Tier::Tier5 {
            t5_formulas_by_payload_id.get(&m.payload_id).copied()
        } else {
            None
        };
        nonce_map.insert(
            m.nonce.clone(),
            NonceMeta {
                tier: u8::from(m.tier),
                payload_id: m.payload_id.clone(),
                embedding_loc: m.embedding_location.to_string(),
                t5_formula,
            },
        );
    }
    let nonce_count = nonce_map.len();

    // Load crawler catalog
    let crawler_catalog = CrawlerCatalog::load()?;

    // Ensure the DB parent directory exists before SQLite tries to open the file.
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            anyhow::anyhow!("Failed to create DB directory {}: {}", parent.display(), e)
        })?;
    }

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
        conn: conn.clone(),
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

/// Compute the expected T5 proof value in u64 space, then reduce to u32.
/// Returns a number in [0, modulus - 1]. D-13-02 arithmetic:
/// `proof = ((seed + formula_a) * formula_b) % formula_mod`.
///
/// u64 promotion is critical: `(seed + a) * b` overflows u32 when seed near
/// u32::MAX (RESEARCH Pitfall 3 / Risk 2). `wrapping_add` / `wrapping_mul`
/// on u64 cannot in practice overflow (max value is ~u32::MAX^2 + u32::MAX
/// < 2^64), but the wrapping versions are used as defense-in-depth.
///
/// PARENTHESIZATION INVARIANT: the formula is `((seed + a) * b) % m`. A
/// refactor that changes this to `(seed + (a * b)) % m` would silently
/// produce wrong proofs on ALL nonzero seeds. Guarded by
/// `test_compute_expected_proof_seed_zero_nontrivial_a_b`.
fn compute_expected_proof(seed: u32, formula_a: u32, formula_b: u32, formula_mod: u32) -> u32 {
    let s = seed as u64;
    let a = formula_a as u64;
    let b = formula_b as u64;
    let m = formula_mod as u64;
    ((s.wrapping_add(a).wrapping_mul(b)) % m) as u32
}

/// D-13-09 hand-rolled sanitizer: `^[a-z0-9_,.\-]{1,256}$`
/// No `regex` crate dependency -- auditable byte scan (RESEARCH "Don't Hand-Roll" line 461).
fn is_valid_t4_payload(s: &str) -> bool {
    let len = s.len();
    if len == 0 || len > 256 {
        return false;
    }
    s.bytes().all(|b| {
        b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b',' | b'.' | b'-')
    })
}

/// URL-safe base64 decode with D-13-09 sanitization. Returns None on any failure.
/// Oversize check BEFORE decode to avoid wasteful decoding (RESEARCH line 308).
fn decode_t4_payload(b64: &str) -> Option<String> {
    // URL-safe base64 of 256 bytes = ceil(256 * 4/3) ~= 344; allow slack to 400.
    if b64.len() > 400 {
        return None;
    }
    let decoded = general_purpose::URL_SAFE_NO_PAD.decode(b64).ok()?;
    let as_str = String::from_utf8(decoded).ok()?;
    let normalized: String = as_str
        .trim()
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if is_valid_t4_payload(&normalized) {
        Some(normalized)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_expected_proof_zero_seed() {
        // (0 + 0) * 1 % 1000 = 0
        assert_eq!(compute_expected_proof(0, 0, 1, 1000), 0);
    }

    #[test]
    fn test_compute_expected_proof_max_seed_no_overflow() {
        // RESEARCH Pitfall 3 critical test: MUST NOT panic in debug builds.
        // u32::MAX + 1_000_000 would overflow u32; u64 promotion prevents it.
        let result = compute_expected_proof(u32::MAX, 1_000_000, 1_000_000, 1000);
        // Don't assert specific value -- just that we didn't panic and got something in range.
        assert!(result < 1000);
    }

    #[test]
    fn test_compute_expected_proof_known_vector() {
        // seed = 0x12345678 = 305_419_896
        // a = 42, b = 17, mod = 1000
        // S = 305_419_896 + 42 = 305_419_938
        // proof = (305_419_938 * 17) % 1000 = 5_192_138_946 % 1000 = 946
        assert_eq!(compute_expected_proof(0x12345678, 42, 17, 1000), 946);
    }

    #[test]
    fn test_compute_expected_proof_seed_zero_nontrivial_a_b() {
        // PARENTHESIZATION TRAP: seed=0 does NOT distinguish `(seed + a) * b`
        // from `seed + (a * b)` because addition-identity collapses both forms
        // to `a * b` at seed=0. We must use a nonzero seed.
        //
        // With seed=1, a=42, b=17, mod=1000:
        //   Correct `((seed + a) * b) % mod`:
        //     ((1 + 42) * 17) % 1000 = (43 * 17) % 1000 = 731 % 1000 = 731
        //   WRONG  `(seed + (a * b)) % mod`:
        //     (1 + (42 * 17)) % 1000 = (1 + 714) % 1000 = 715
        //
        // Any future refactor that silently swaps the parenthesization will
        // produce 715 and fail this test -- the intended guard.
        assert_eq!(compute_expected_proof(1, 42, 17, 1000), 731);
    }

    #[test]
    fn test_t4_decode_valid_payload() {
        // "web_search,browse_page" URL-safe base64 no-pad
        let input = "web_search,browse_page";
        let encoded = general_purpose::URL_SAFE_NO_PAD.encode(input.as_bytes());
        assert_eq!(decode_t4_payload(&encoded), Some(input.to_string()));
    }

    #[test]
    fn test_t4_decode_rejects_oversize() {
        let oversize = "A".repeat(500);
        assert_eq!(decode_t4_payload(&oversize), None);
    }

    #[test]
    fn test_t4_decode_rejects_invalid_chars() {
        let bad = general_purpose::URL_SAFE_NO_PAD.encode(b"contains!punctuation@");
        assert_eq!(decode_t4_payload(&bad), None);
    }

    #[test]
    fn test_t4_decode_normalizes_case_and_whitespace() {
        let input = "Web_Search , Browse_Page";
        let encoded = general_purpose::URL_SAFE_NO_PAD.encode(input.as_bytes());
        assert_eq!(
            decode_t4_payload(&encoded),
            Some("web_search,browse_page".to_string())
        );
    }

    #[test]
    fn test_t4_decode_rejects_non_base64() {
        // Characters not in URL-safe alphabet (contains + and /)
        assert_eq!(decode_t4_payload("not+url/safe"), None);
    }

    #[test]
    fn test_is_valid_t4_payload_empty_and_oversize() {
        assert!(!is_valid_t4_payload(""), "empty string must be rejected");
        let s = "a".repeat(256);
        assert!(
            is_valid_t4_payload(&s),
            "256 chars of lowercase alpha must be accepted"
        );
        let s = "a".repeat(257);
        assert!(!is_valid_t4_payload(&s), "257 chars must be rejected");
    }
}
