/// Integration tests for the honeyprompt serve command.
///
/// Uses tower::ServiceExt::oneshot to test the Axum router in-process without binding a port.
/// ConnectInfo is satisfied via axum::extract::connect_info::MockConnectInfo layer.
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::StatusCode;
use honeyprompt::{config, generator, server, store};
use http::Request;
use tempfile::tempdir;
use tokio::sync::mpsc;
use tower::ServiceExt;

/// Helper: run init + generate in a tempdir, return the dir (kept alive).
fn init_and_generate() -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir must create");
    let path = dir.path();

    std::fs::create_dir_all(path.join("output")).expect("output dir must create");
    std::fs::create_dir_all(path.join(".honeyprompt").join("overrides"))
        .expect("overrides dir must create");
    let config_path = path.join("honeyprompt.toml");
    config::write_default_config(&config_path).expect("write_default_config must succeed");
    let db_path = path.join(".honeyprompt").join("events.db");
    let conn = store::open_or_create_db(&db_path).expect("open_or_create_db must succeed");

    let cfg = config::load_config(&config_path).expect("load_config must succeed");
    generator::generate(&cfg, &conn, path).expect("generate must succeed");

    dir
}

/// Helper: build AppState loaded from the generated output directory.
async fn build_test_state(
    dir: &tempfile::TempDir,
    callback_tx: mpsc::Sender<honeyprompt::types::RawCallbackEvent>,
) -> (
    Arc<server::AppState>,
    std::path::PathBuf,
    Vec<honeyprompt::types::NonceMapping>,
) {
    let path = dir.path();
    let output_dir = path.join("output");
    let db_path = path.join(".honeyprompt").join("events.db");

    // Open async connection to the same DB that init_and_generate created
    let conn = tokio_rusqlite::Connection::open(&db_path)
        .await
        .expect("tokio-rusqlite connection must open");

    // Load callback-map.json
    let json_str = std::fs::read_to_string(output_dir.join("callback-map.json"))
        .expect("callback-map.json must exist after generate");
    let mappings: Vec<honeyprompt::types::NonceMapping> =
        serde_json::from_str(&json_str).expect("callback-map.json must be valid JSON");

    // Build nonce_map
    let mut nonce_map = HashMap::new();
    for m in &mappings {
        nonce_map.insert(
            m.nonce.clone(),
            server::NonceMeta {
                tier: u8::from(m.tier),
                payload_id: m.payload_id.clone(),
                embedding_loc: m.embedding_location.to_string(),
                t5_formula: None,
            },
        );
    }

    let crawler_catalog =
        honeyprompt::crawler_catalog::CrawlerCatalog::load().expect("catalog must load");

    let app_state = Arc::new(server::AppState {
        conn,
        callback_tx,
        nonce_map,
        crawler_catalog,
    });

    (app_state, output_dir, mappings)
}

/// Build a router with MockConnectInfo layer for in-process testing.
fn test_router(
    state: Arc<server::AppState>,
    output_dir: std::path::PathBuf,
) -> impl tower::Service<
    http::Request<Body>,
    Response = axum::response::Response,
    Error = std::convert::Infallible,
    Future = impl std::future::Future<
        Output = Result<axum::response::Response, std::convert::Infallible>,
    >,
> + Clone {
    let mock_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
    server::build_router(state, output_dir).layer(MockConnectInfo(mock_addr))
}

/// A valid callback request with a known nonce returns 204 No Content.
#[tokio::test]
async fn test_callback_valid_nonce_returns_204() {
    let dir = init_and_generate();
    let (callback_tx, _callback_rx) = mpsc::channel(16);
    let (state, output_dir, mappings) = build_test_state(&dir, callback_tx).await;

    // Pick the first nonce from the generated map
    let nonce = mappings[0].nonce.clone();
    let uri = format!("/cb/v1/{}", nonce);

    let app = test_router(state, output_dir);
    let response = app
        .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "valid nonce must return 204"
    );
}

/// A callback request with a nonce that has invalid characters (not hex) returns 204 (D-03).
#[tokio::test]
async fn test_callback_invalid_nonce_returns_204() {
    let dir = init_and_generate();
    let (callback_tx, _callback_rx) = mpsc::channel(16);
    let (state, output_dir, _) = build_test_state(&dir, callback_tx).await;

    // "INVALID!abc" has uppercase and a non-hex character — invalid format
    let app = test_router(state, output_dir);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cb/v1/INVALID!abcdefg")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "invalid nonce format must return 204 (D-03)"
    );
}

/// A callback request with a valid-format nonce that is not in the map returns 204 (D-03).
#[tokio::test]
async fn test_callback_unknown_nonce_returns_204() {
    let dir = init_and_generate();
    let (callback_tx, _callback_rx) = mpsc::channel(16);
    let (state, output_dir, _) = build_test_state(&dir, callback_tx).await;

    // 16 lowercase hex chars — valid format but not in the generated nonce_map
    let app = test_router(state, output_dir);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cb/v1/0000000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "unknown nonce must return 204 (D-03)"
    );
}

/// GET / returns 200 and serves the generated honeypot page.
#[tokio::test]
async fn test_static_file_serving() {
    let dir = init_and_generate();
    let (callback_tx, _callback_rx) = mpsc::channel(16);
    let (state, output_dir, _) = build_test_state(&dir, callback_tx).await;

    let app = test_router(state, output_dir);
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "GET / must return 200");

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert!(
        body_str.contains("warning-banner") || body_str.contains("SECURITY RESEARCH CANARY"),
        "index.html must contain warning banner content"
    );
}

/// A valid nonce callback sends a RawCallbackEvent to the mpsc channel.
#[tokio::test]
async fn test_callback_sends_event_to_channel() {
    let dir = init_and_generate();
    let (callback_tx, mut callback_rx) = mpsc::channel(16);
    let (state, output_dir, mappings) = build_test_state(&dir, callback_tx).await;

    let first_mapping = &mappings[0];
    let nonce = first_mapping.nonce.clone();
    let expected_tier = u8::from(first_mapping.tier);
    let uri = format!("/cb/v1/{}", nonce);

    let app = test_router(state, output_dir);
    let response = app
        .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Receive the RawCallbackEvent from the channel
    let received = tokio::time::timeout(std::time::Duration::from_millis(500), callback_rx.recv())
        .await
        .expect("timed out waiting for RawCallbackEvent")
        .expect("channel was closed unexpectedly");

    assert_eq!(
        received.nonce, nonce,
        "received event must have correct nonce"
    );
    assert_eq!(
        received.tier, expected_tier,
        "received event must have correct tier"
    );
}

/// GET /stats returns 200 and valid JSON with correct shape (empty DB — no callback events).
#[tokio::test]
async fn test_stats_empty_db_returns_json() {
    // Use a fresh tempdir with an empty DB (no callback events fired)
    let dir = tempdir().expect("tempdir must create");
    let path = dir.path();
    std::fs::create_dir_all(path.join("output")).expect("output dir must create");
    std::fs::create_dir_all(path.join(".honeyprompt").join("overrides")).expect("overrides dir");
    let config_path = path.join("honeyprompt.toml");
    config::write_default_config(&config_path).expect("write config");
    let db_path = path.join(".honeyprompt").join("events.db");
    let sync_conn = store::open_or_create_db(&db_path).expect("open db");
    let cfg = config::load_config(&config_path).expect("load config");
    generator::generate(&cfg, &sync_conn, path).expect("generate");
    drop(sync_conn);

    let (callback_tx, _rx) = mpsc::channel(16);
    let (state, output_dir, _) = build_test_state(&dir, callback_tx).await;

    let app = test_router(state, output_dir);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /stats must return 200"
    );

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&body_bytes).expect("/stats must return valid JSON");

    // All counts should be zero on empty DB (STATS-03)
    assert_eq!(
        json["total_sessions"], 0,
        "total_sessions must be 0 on empty DB"
    );
    assert_eq!(json["detection_sessions"], 0);
    assert_eq!(json["crawler_sessions"], 0);
    assert_eq!(json["tier1_sessions"], 0);
    assert_eq!(json["tier2_sessions"], 0);
    assert_eq!(json["tier3_sessions"], 0);
    assert!(
        json["earliest_event"].is_null(),
        "earliest_event must be null on empty DB"
    );
    assert!(
        json["latest_event"].is_null(),
        "latest_event must be null on empty DB"
    );
}

/// GET /stats returns correct counts after callback events are inserted (populated DB).
#[tokio::test]
async fn test_stats_populated_db_returns_counts() {
    let dir = init_and_generate();
    let db_path = dir.path().join(".honeyprompt").join("events.db");

    // Insert a test event directly into the DB
    let sync_conn = store::open_or_create_db(&db_path).expect("open db");
    store::insert_nonce(&sync_conn, "statstest00001aa", 1, "t1-html", "html_comment").unwrap();
    store::insert_callback_event(
        &sync_conn,
        "statstest00001aa",
        1,
        "t1-html",
        "html_comment",
        "sess_stats_1",
        "10.0.0.1",
        "TestAgent/1.0",
        r#"{"classification":"Unknown","headers":{}}"#,
        None,
        None,
        None,
    )
    .unwrap();
    drop(sync_conn);

    let (callback_tx, _rx) = mpsc::channel(16);
    let (state, output_dir, _) = build_test_state(&dir, callback_tx).await;

    let app = test_router(state, output_dir);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).expect("valid JSON");

    assert!(
        json["total_sessions"].as_u64().unwrap() >= 1,
        "total_sessions must be >= 1"
    );
    assert!(
        json["detection_sessions"].as_u64().unwrap() >= 1,
        "detection_sessions must be >= 1"
    );
    assert!(
        json["earliest_event"].is_string(),
        "earliest_event must be a string when events exist"
    );
}

/// GET /stats includes Access-Control-Allow-Origin: * header (STATS-02).
#[tokio::test]
async fn test_stats_has_cors_header() {
    let dir = init_and_generate();
    let (callback_tx, _rx) = mpsc::channel(16);
    let (state, output_dir, _) = build_test_state(&dir, callback_tx).await;

    let app = test_router(state, output_dir);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let cors = response
        .headers()
        .get("access-control-allow-origin")
        .expect("Access-Control-Allow-Origin header must be present");
    assert_eq!(cors, "*", "CORS header must be '*'");
}

// ============================================================================
// Phase 13 — T4/T5 integration tests
// ============================================================================
//
// These tests use a dedicated fixture builder (`build_t4_t5_state`) that pre-populates
// AppState with T1/T4/T5 nonces. Unlike `build_test_state`, this does NOT require
// the generator output on disk — it constructs NonceMeta directly, avoiding the cost
// of running the full init+generate pipeline for server-behavior tests.

use honeyprompt::types::{RawCallbackEvent, T5Formula};
use std::time::Duration;

/// Fixture: an AppState populated with T1/T4/T5 nonces for handler testing.
///
/// - T1 nonce "1111111111111111" → tier 1 (proves /cb/v1/ regression)
/// - T4 nonce "aaaaaaaaaaaaaaaa" → tier 4, no formula
/// - T5 nonce "bbbbbbbbbbbbbbbb" → tier 5, formula (a=42, b=17, modulus=1000)
async fn build_t4_t5_state(
    callback_tx: mpsc::Sender<RawCallbackEvent>,
) -> (Arc<server::AppState>, std::path::PathBuf, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir must create");
    let output_dir = dir.path().join("output");
    std::fs::create_dir_all(&output_dir).expect("output dir must create");
    let db_path = dir.path().join(".honeyprompt").join("events.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).expect("db parent must create");
    // Initialise the DB so tokio-rusqlite can open it
    {
        let conn = store::open_or_create_db(&db_path).expect("open_or_create_db must succeed");
        drop(conn);
    }

    let conn = tokio_rusqlite::Connection::open(&db_path)
        .await
        .expect("tokio-rusqlite connection must open");

    let mut nonce_map: HashMap<String, server::NonceMeta> = HashMap::new();
    nonce_map.insert(
        "1111111111111111".to_string(),
        server::NonceMeta {
            tier: 1,
            payload_id: "t1-html-comment".to_string(),
            embedding_loc: "HtmlComment".to_string(),
            t5_formula: None,
        },
    );
    nonce_map.insert(
        "aaaaaaaaaaaaaaaa".to_string(),
        server::NonceMeta {
            tier: 4,
            payload_id: "t4-tools-meta".to_string(),
            embedding_loc: "HtmlComment".to_string(),
            t5_formula: None,
        },
    );
    nonce_map.insert(
        "bbbbbbbbbbbbbbbb".to_string(),
        server::NonceMeta {
            tier: 5,
            payload_id: "t5-semantic-prose".to_string(),
            embedding_loc: "HtmlComment".to_string(),
            t5_formula: Some(T5Formula {
                a: 42,
                b: 17,
                modulus: 1000,
            }),
        },
    );

    let crawler_catalog =
        honeyprompt::crawler_catalog::CrawlerCatalog::load().expect("catalog must load");

    let app_state = Arc::new(server::AppState {
        conn,
        callback_tx,
        nonce_map,
        crawler_catalog,
    });

    (app_state, output_dir, dir)
}

/// T4 happy path: valid b64 payload decodes, sanitizes, and fires an event with t4_capability.
#[tokio::test]
async fn test_t4_callback_happy_path() {
    use base64::{engine::general_purpose, Engine as _};
    let (callback_tx, mut callback_rx) = mpsc::channel::<RawCallbackEvent>(16);
    let (state, output_dir, _dir) = build_t4_t5_state(callback_tx).await;

    let b64 = general_purpose::URL_SAFE_NO_PAD.encode(b"web_search,browse_page");
    let uri = format!("/cb/v4/aaaaaaaaaaaaaaaa/{}", b64);

    let app = test_router(state, output_dir);
    let response = app
        .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let event = tokio::time::timeout(Duration::from_millis(500), callback_rx.recv())
        .await
        .expect("must broadcast within 500ms")
        .expect("channel receive");
    assert_eq!(
        event.t4_capability,
        Some("web_search,browse_page".to_string())
    );
    assert_eq!(event.tier, 4);
    assert!(event.t5_proof.is_none());
    assert!(event.t5_proof_valid.is_none());
}

/// T4 malformed inputs all return 204 and NO event is broadcast.
#[tokio::test]
async fn test_t4_malformed_returns_204() {
    use base64::{engine::general_purpose, Engine as _};
    let (callback_tx, mut callback_rx) = mpsc::channel::<RawCallbackEvent>(16);
    let (state, output_dir, _dir) = build_t4_t5_state(callback_tx).await;

    let invalid_sanitize_b64 = general_purpose::URL_SAFE_NO_PAD.encode(b"has!punctuation@");

    let cases: Vec<String> = vec![
        // Standard-base64 chars '+' that are NOT in URL-safe alphabet → decoder rejects
        format!("/cb/v4/aaaaaaaaaaaaaaaa/{}", "abcd+efgh"),
        format!("/cb/v4/aaaaaaaaaaaaaaaa/{}", "A".repeat(500)), // oversize
        format!("/cb/v4/aaaaaaaaaaaaaaaa/{}", invalid_sanitize_b64), // sanitizer fails
        format!("/cb/v4/NOT_HEX_NONCE___/{}", "ab"),            // bad nonce format
        format!("/cb/v4/cccccccccccccccc/{}", "ab"),            // unknown nonce
        format!("/cb/v4/1111111111111111/{}", "ab"),            // wrong tier (T1 nonce)
    ];

    for uri in &cases {
        let app = test_router(state.clone(), output_dir.clone());
        let response = app
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::NO_CONTENT,
            "malformed T4 URI {} must return 204 (D-13-15)",
            uri
        );
    }

    let no_event = tokio::time::timeout(Duration::from_millis(200), callback_rx.recv()).await;
    assert!(
        no_event.is_err(),
        "no event must be broadcast for malformed T4 inputs (D-13-15)"
    );
}

/// T5 valid proof: server computes expected, stores event with t5_proof_valid=true.
#[tokio::test]
async fn test_t5_callback_valid_proof() {
    let (callback_tx, mut callback_rx) = mpsc::channel::<RawCallbackEvent>(16);
    let (state, output_dir, _dir) = build_t4_t5_state(callback_tx).await;

    // Nonce "bbbbbbbbbbbbbbbb" → first 8 chars "bbbbbbbb" = 0xbbbbbbbb = 3_149_642_683
    // formula (a=42, b=17, mod=1000): proof = ((seed + 42) * 17) % 1000
    // Compute in the test to avoid accidentally testing the same bug twice:
    let seed: u64 = 0xbbbbbbbb;
    let expected: u32 = (((seed.wrapping_add(42)).wrapping_mul(17)) % 1000) as u32;
    let proof_str = format!("{:03}", expected);

    let uri = format!("/cb/v5/bbbbbbbbbbbbbbbb/{}", proof_str);
    let app = test_router(state, output_dir);
    let response = app
        .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let event = tokio::time::timeout(Duration::from_millis(500), callback_rx.recv())
        .await
        .expect("must broadcast within 500ms")
        .expect("channel receive");
    assert_eq!(event.t5_proof, Some(proof_str.clone()));
    assert_eq!(event.t5_proof_valid, Some(true));
    assert_eq!(event.tier, 5);
    assert!(event.t4_capability.is_none());
}

/// T5 invalid proof: server records the submission with t5_proof_valid=false (still 204).
#[tokio::test]
async fn test_t5_callback_invalid_proof() {
    let (callback_tx, mut callback_rx) = mpsc::channel::<RawCallbackEvent>(16);
    let (state, output_dir, _dir) = build_t4_t5_state(callback_tx).await;

    let seed: u64 = 0xbbbbbbbb;
    let expected: u32 = (((seed.wrapping_add(42)).wrapping_mul(17)) % 1000) as u32;
    // Pick a deliberately wrong 3-digit proof that differs from expected
    let wrong_value: u32 = (expected + 1) % 1000;
    let wrong_str = format!("{:03}", wrong_value);

    let uri = format!("/cb/v5/bbbbbbbbbbbbbbbb/{}", wrong_str);
    let app = test_router(state, output_dir);
    let response = app
        .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "D-13-15: wrong proof still returns 204"
    );

    let event = tokio::time::timeout(Duration::from_millis(500), callback_rx.recv())
        .await
        .expect("must broadcast within 500ms")
        .expect("channel receive");
    assert_eq!(event.t5_proof, Some(wrong_str));
    assert_eq!(event.t5_proof_valid, Some(false));
    assert_eq!(event.tier, 5);
}

/// T5 malformed proofs (bad format, wrong tier, unknown nonce) all return 204 and no event.
#[tokio::test]
async fn test_t5_malformed_returns_204() {
    let (callback_tx, mut callback_rx) = mpsc::channel::<RawCallbackEvent>(16);
    let (state, output_dir, _dir) = build_t4_t5_state(callback_tx).await;

    let cases = vec![
        "/cb/v5/bbbbbbbbbbbbbbbb/42",   // 2-digit proof (length != 3)
        "/cb/v5/bbbbbbbbbbbbbbbb/1234", // 4-digit proof (length != 3)
        "/cb/v5/bbbbbbbbbbbbbbbb/abc",  // non-numeric
        "/cb/v5/1111111111111111/042",  // wrong tier (T1 nonce)
        "/cb/v5/cccccccccccccccc/042",  // unknown nonce
        "/cb/v5/NOT_HEX_NONCE___/042",  // bad nonce format
    ];

    for uri in &cases {
        let app = test_router(state.clone(), output_dir.clone());
        let response = app
            .oneshot(Request::builder().uri(*uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::NO_CONTENT,
            "malformed T5 URI {} must return 204 (D-13-15)",
            uri
        );
    }

    let no_event = tokio::time::timeout(Duration::from_millis(200), callback_rx.recv()).await;
    assert!(
        no_event.is_err(),
        "no event must be broadcast for malformed T5 inputs"
    );
}

/// D-13-18 / RESEARCH A4: /cb/v1/ must return 204 with an empty body — byte-identical to v4.0.
#[tokio::test]
async fn test_cb_v1_byte_identical_response() {
    let (callback_tx, _callback_rx) = mpsc::channel::<RawCallbackEvent>(16);
    let (state, output_dir, _dir) = build_t4_t5_state(callback_tx).await;

    let app = test_router(state, output_dir);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cb/v1/1111111111111111")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "D-13-18: /cb/v1/ status must be 204"
    );
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(
        body_bytes.is_empty(),
        "D-13-18: /cb/v1/ body must be empty (byte-identical to v4.0)"
    );
}
