//! Integration test: --domain flag tempdir serve mode
//! Validates SERVE-01 (generates and serves without config file) and
//! SERVE-02 (callback_base_url set to https://{domain}).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use honeyprompt::{config, generator, server, store};
use http::Request;
use tokio::sync::mpsc;
use tower::ServiceExt;

/// SERVE-01 + SERVE-02: tempdir generate with --domain produces working honeypot
/// with correct callback_base_url baked into the HTML.
#[test]
fn test_domain_tempdir_generates_correct_callback_urls() {
    let tmp = tempfile::TempDir::new().unwrap();
    let tmp_path = tmp.path().to_path_buf();

    std::fs::create_dir_all(tmp_path.join(".honeyprompt")).unwrap();
    std::fs::create_dir_all(tmp_path.join("output")).unwrap();

    let base = config::Config::default();
    let cfg = config::config_with_overrides(&base, Some("test.example.com"), None, None);

    // Verify SERVE-02 defaults applied by config_with_overrides
    assert_eq!(cfg.callback_base_url, "https://test.example.com");
    assert_eq!(cfg.bind_address, "0.0.0.0:8080");
    assert_eq!(cfg.tiers, vec![1, 2, 3]);

    // Write config and generate project
    let config_path = tmp_path.join("honeyprompt.toml");
    let toml_str = toml::to_string_pretty(&cfg).unwrap();
    std::fs::write(&config_path, toml_str).unwrap();

    let db_path = tmp_path.join(".honeyprompt").join("events.db");
    let conn = store::open_or_create_db(&db_path).unwrap();
    generator::generate(&cfg, &conn, &tmp_path).unwrap();
    drop(conn);

    // Verify generated HTML contains the domain-based callback URL
    let html = std::fs::read_to_string(tmp_path.join("output/index.html")).unwrap();
    assert!(
        html.contains("https://test.example.com/cb/v1/"),
        "Generated HTML must contain domain-based callback URLs, got:\n{}",
        &html[..html.len().min(500)]
    );
}

/// SERVE-01: tempdir serve returns 200 for GET /
#[tokio::test]
async fn test_domain_tempdir_serves_index() {
    // Setup tempdir project with --domain config
    let tmp = tempfile::TempDir::new().unwrap();
    let tmp_path = tmp.path().to_path_buf();
    std::fs::create_dir_all(tmp_path.join(".honeyprompt")).unwrap();
    std::fs::create_dir_all(tmp_path.join("output")).unwrap();

    let base = config::Config::default();
    let cfg = config::config_with_overrides(&base, Some("test.example.com"), None, None);
    let config_path = tmp_path.join("honeyprompt.toml");
    std::fs::write(&config_path, toml::to_string_pretty(&cfg).unwrap()).unwrap();

    let db_path = tmp_path.join(".honeyprompt").join("events.db");
    let sync_conn = store::open_or_create_db(&db_path).unwrap();
    generator::generate(&cfg, &sync_conn, &tmp_path).unwrap();
    drop(sync_conn);

    // Build router the same way server::serve does
    let callback_map_path = tmp_path.join("output/callback-map.json");
    let json_str = std::fs::read_to_string(&callback_map_path).unwrap();
    let mappings: Vec<honeyprompt::types::NonceMapping> = serde_json::from_str(&json_str).unwrap();

    let mut nonce_map: HashMap<String, server::NonceMeta> = HashMap::new();
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

    let crawler_catalog = honeyprompt::crawler_catalog::CrawlerCatalog::load().unwrap();
    let async_conn = tokio_rusqlite::Connection::open(&db_path).await.unwrap();
    let (callback_tx, _rx) = mpsc::channel(256);

    let state = Arc::new(server::AppState {
        conn: async_conn,
        callback_tx,
        nonce_map,
        crawler_catalog,
    });

    let mock_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
    let app =
        server::build_router(state, tmp_path.join("output")).layer(MockConnectInfo(mock_addr));

    // GET / should return 200
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "GET / on tempdir domain serve must return 200"
    );
}
