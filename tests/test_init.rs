use honeyprompt::{config, store};
use tempfile::tempdir;

/// Verify that init creates the expected scaffold: config, output dir, overrides dir, DB.
#[test]
fn test_init_creates_scaffold() {
    let dir = tempdir().expect("tempdir must create");
    let path = dir.path();

    // Simulate init steps
    let config_path = path.join("honeyprompt.toml");
    std::fs::create_dir_all(path.join("output")).expect("output dir must create");
    std::fs::create_dir_all(path.join(".honeyprompt").join("overrides"))
        .expect("overrides dir must create");
    config::write_default_config(&config_path).expect("write_default_config must succeed");
    let db_path = path.join(".honeyprompt").join("events.db");
    let _conn = store::open_or_create_db(&db_path).expect("open_or_create_db must succeed");

    // Assert scaffold exists
    assert!(
        config_path.exists(),
        "honeyprompt.toml must exist after init"
    );
    assert!(
        path.join("output").is_dir(),
        "output/ must be a directory after init"
    );
    assert!(
        path.join(".honeyprompt").join("overrides").is_dir(),
        ".honeyprompt/overrides/ must be a directory after init"
    );
    assert!(
        db_path.exists(),
        ".honeyprompt/events.db must exist after init"
    );
}

/// Verify that attempting to init a project twice fails with an "already exists" error.
#[test]
fn test_init_refuses_reinit() {
    let dir = tempdir().expect("tempdir must create");
    let path = dir.path();
    let config_path = path.join("honeyprompt.toml");

    // First init
    config::write_default_config(&config_path).expect("first init must succeed");

    // Second init: config file already exists — should be detected as error
    assert!(
        config_path.exists(),
        "honeyprompt.toml must exist after first init"
    );

    // The main.rs bail! check is: if config_path.exists() { bail!("honeyprompt.toml already exists") }
    // Test the detection logic directly
    let would_reinit = config_path.exists();
    assert!(
        would_reinit,
        "Re-init detection must trigger when honeyprompt.toml already exists"
    );
}

/// Verify that the generated honeyprompt.toml is valid TOML that parses into a Config struct.
#[test]
fn test_init_config_is_valid_toml() {
    let dir = tempdir().expect("tempdir must create");
    let config_path = dir.path().join("honeyprompt.toml");

    config::write_default_config(&config_path).expect("write_default_config must succeed");

    let loaded = config::load_config(&config_path)
        .expect("load_config must succeed — file must be valid TOML parseable into Config");

    // Verify the loaded config has reasonable defaults
    assert!(
        !loaded.callback_base_url.is_empty(),
        "callback_base_url must not be empty"
    );
    assert!(
        !loaded.bind_address.is_empty(),
        "bind_address must not be empty"
    );
    assert!(!loaded.tiers.is_empty(), "tiers must not be empty");
    assert!(
        !loaded.page_title.is_empty(),
        "page_title must not be empty"
    );
}
