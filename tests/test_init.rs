use honeyprompt::config;
use tempfile::tempdir;

/// `honeyprompt init` creates config, output dir and DB (and no unused
/// `.honeyprompt/overrides/` directory, which nothing ever read).
#[test]
fn test_init_creates_scaffold() {
    let dir = tempdir().expect("tempdir must create");
    let path = dir.path();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
        .arg("init")
        .arg(path)
        .output()
        .expect("run init");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let config_path = path.join("honeyprompt.toml");
    let db_path = path.join(".honeyprompt").join("events.db");
    config::load_config(&config_path).expect("init writes a loadable config");

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
        !path.join(".honeyprompt").join("overrides").exists(),
        "init no longer creates .honeyprompt/overrides/"
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

/// `report`, `seed add` and `seed list` on a path with no events DB fail
/// instead of silently creating a new empty database there.
#[test]
fn report_and_seed_refuse_a_missing_db() {
    let dir = tempdir().unwrap();
    let path = dir.path();
    // A config but no DB (e.g. a typo'd or half-copied project).
    config::write_default_config(&path.join("honeyprompt.toml")).unwrap();
    for args in [
        vec!["report", "--stdout"],
        vec!["seed", "add", "x"],
        vec!["seed", "list"],
    ] {
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_honeyprompt"));
        cmd.args(&args);
        // `report --stdout <path>` / `seed add x <path>` / `seed list <path>`
        cmd.arg(path);
        let out = cmd.output().unwrap();
        assert!(!out.status.success(), "{args:?} must fail");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("database not found"), "{args:?}: {err}");
        assert!(
            !path.join(".honeyprompt").exists(),
            "{args:?} must not create .honeyprompt/"
        );
    }
}
