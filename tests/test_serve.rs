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
fn build_test_state(
    dir: &tempfile::TempDir,
    callback_tx: mpsc::Sender<honeyprompt::types::RawCallbackEvent>,
) -> (
    Arc<server::AppState>,
    std::path::PathBuf,
    Vec<honeyprompt::types::NonceMapping>,
) {
    let path = dir.path();
    let output_dir = path.join("output");

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
            },
        );
    }

    let crawler_catalog =
        honeyprompt::crawler_catalog::CrawlerCatalog::load().expect("catalog must load");

    let app_state = Arc::new(server::AppState {
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
    let (state, output_dir, mappings) = build_test_state(&dir, callback_tx);

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
    let (state, output_dir, _) = build_test_state(&dir, callback_tx);

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
    let (state, output_dir, _) = build_test_state(&dir, callback_tx);

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
    let (state, output_dir, _) = build_test_state(&dir, callback_tx);

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
    let (state, output_dir, mappings) = build_test_state(&dir, callback_tx);

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
