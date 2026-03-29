use honeyprompt::{config, generator, store};
use serde_json::Value;
use tempfile::tempdir;

/// Helper: run init + generate in a tempdir, return the path.
fn init_and_generate() -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir must create");
    let path = dir.path();

    // Init
    std::fs::create_dir_all(path.join("output")).expect("output dir must create");
    std::fs::create_dir_all(path.join(".honeyprompt").join("overrides"))
        .expect("overrides dir must create");
    let config_path = path.join("honeyprompt.toml");
    config::write_default_config(&config_path).expect("write_default_config must succeed");
    let db_path = path.join(".honeyprompt").join("events.db");
    let conn = store::open_or_create_db(&db_path).expect("open_or_create_db must succeed");

    // Generate
    let cfg = config::load_config(&config_path).expect("load_config must succeed");
    generator::generate(&cfg, &conn, path).expect("generate must succeed");

    dir
}

/// Verify all 4 output files are created.
#[test]
fn test_generate_creates_output_files() {
    let dir = init_and_generate();
    let path = dir.path();
    assert!(path.join("output/index.html").exists(), "output/index.html must exist");
    assert!(path.join("output/robots.txt").exists(), "output/robots.txt must exist");
    assert!(path.join("output/ai.txt").exists(), "output/ai.txt must exist");
    assert!(
        path.join("output/callback-map.json").exists(),
        "output/callback-map.json must exist"
    );
}

/// Verify the generated HTML has the hard-coded warning banner and inline notice.
#[test]
fn test_generate_html_has_warning() {
    let dir = init_and_generate();
    let html =
        std::fs::read_to_string(dir.path().join("output/index.html")).expect("index.html must be readable");
    assert!(
        html.contains("SECURITY RESEARCH CANARY"),
        "index.html must contain 'SECURITY RESEARCH CANARY'"
    );
    assert!(
        html.contains("Notice:"),
        "index.html must contain inline 'Notice:' text"
    );
}

/// Verify the generated HTML contains callback URLs (payloads were injected).
#[test]
fn test_generate_html_has_payloads() {
    let dir = init_and_generate();
    let html =
        std::fs::read_to_string(dir.path().join("output/index.html")).expect("index.html must be readable");
    assert!(
        html.contains("/cb/"),
        "index.html must contain callback URLs matching '/cb/' pattern"
    );
}

/// Verify payloads appear in multiple distinct embedding locations.
#[test]
fn test_generate_html_multiple_locations() {
    let dir = init_and_generate();
    let html =
        std::fs::read_to_string(dir.path().join("output/index.html")).expect("index.html must be readable");

    let mut found_locations = 0;
    if html.contains("<!--") {
        found_locations += 1;
    }
    if html.contains("<meta ") {
        found_locations += 1;
    }
    if html.contains("application/ld+json") {
        found_locations += 1;
    }
    if html.contains("aria-hidden") {
        found_locations += 1;
    }
    // Semantic prose: the instruction text from catalog tier1-semantic-prose
    if html.contains("security research artifact") {
        found_locations += 1;
    }

    assert!(
        found_locations >= 3,
        "index.html must have payloads in at least 3 distinct embedding locations, found {}",
        found_locations
    );
}

/// Verify each nonce in callback-map.json is exactly 16 lowercase hex characters.
#[test]
fn test_generate_nonce_format() {
    let dir = init_and_generate();
    let json_str =
        std::fs::read_to_string(dir.path().join("output/callback-map.json")).expect("callback-map.json must be readable");
    let entries: Value = serde_json::from_str(&json_str).expect("callback-map.json must be valid JSON");
    let arr = entries.as_array().expect("callback-map.json must be a JSON array");

    assert!(!arr.is_empty(), "callback-map.json must have at least one entry");

    for entry in arr {
        let nonce = entry["nonce"].as_str().expect("each entry must have a 'nonce' string field");
        assert_eq!(
            nonce.len(),
            16,
            "nonce '{}' must be exactly 16 characters",
            nonce
        );
        assert!(
            nonce.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()),
            "nonce '{}' must be lowercase hex",
            nonce
        );
    }
}

/// Verify all nonces in callback-map.json are unique.
#[test]
fn test_generate_nonce_uniqueness() {
    let dir = init_and_generate();
    let json_str =
        std::fs::read_to_string(dir.path().join("output/callback-map.json")).expect("callback-map.json must be readable");
    let entries: Value = serde_json::from_str(&json_str).expect("callback-map.json must be valid JSON");
    let arr = entries.as_array().expect("callback-map.json must be a JSON array");

    let mut seen = std::collections::HashSet::new();
    for entry in arr {
        let nonce = entry["nonce"].as_str().expect("nonce field must exist");
        assert!(
            seen.insert(nonce.to_string()),
            "Duplicate nonce found: '{}' — all nonces must be unique",
            nonce
        );
    }
}

/// Verify robots.txt contains AI-specific user-agent disallow rules.
#[test]
fn test_generate_robots_has_ai_bots() {
    let dir = init_and_generate();
    let robots =
        std::fs::read_to_string(dir.path().join("output/robots.txt")).expect("robots.txt must be readable");
    assert!(robots.contains("GPTBot"), "robots.txt must contain 'GPTBot'");
    assert!(robots.contains("ClaudeBot"), "robots.txt must contain 'ClaudeBot'");
    assert!(
        robots.contains("Google-Extended"),
        "robots.txt must contain 'Google-Extended'"
    );
}

/// Verify ai.txt is non-empty and contains agent policy declarations.
#[test]
fn test_generate_ai_txt_exists() {
    let dir = init_and_generate();
    let ai_txt =
        std::fs::read_to_string(dir.path().join("output/ai.txt")).expect("ai.txt must be readable");
    assert!(!ai_txt.trim().is_empty(), "ai.txt must not be empty");
    assert!(
        ai_txt.contains("Disallow"),
        "ai.txt must contain 'Disallow' policy declarations"
    );
}

/// Verify callback-map.json entries have all required fields.
#[test]
fn test_generate_callback_map_structure() {
    let dir = init_and_generate();
    let json_str =
        std::fs::read_to_string(dir.path().join("output/callback-map.json")).expect("callback-map.json must be readable");
    let entries: Value = serde_json::from_str(&json_str).expect("callback-map.json must be valid JSON");
    let arr = entries.as_array().expect("callback-map.json must be a JSON array");

    assert!(!arr.is_empty(), "callback-map.json must have at least one entry");

    for entry in arr {
        let obj = entry.as_object().expect("each entry must be a JSON object");
        assert!(obj.contains_key("nonce"), "entry must have 'nonce' field: {:?}", obj);
        assert!(obj.contains_key("tier"), "entry must have 'tier' field: {:?}", obj);
        assert!(obj.contains_key("payload_id"), "entry must have 'payload_id' field: {:?}", obj);
        assert!(
            obj.contains_key("embedding_location"),
            "entry must have 'embedding_location' field: {:?}",
            obj
        );
        assert!(
            obj.contains_key("callback_url"),
            "entry must have 'callback_url' field: {:?}",
            obj
        );
    }
}
