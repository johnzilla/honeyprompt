//! `honeyprompt seed add` / `seed list` through the real binary.

use std::process::Command;

fn hp(args: &[&str], dir: &std::path::Path) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
        .args(args)
        .arg(dir)
        .output()
        .expect("run honeyprompt");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn seed_add_and_list() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    assert_eq!(hp(&["init"], p).0, 0);

    let (code, stdout, stderr) = hp(&["seed", "add", "claude-md-prod"], p);
    assert_eq!(code, 0, "{stderr}");
    let url = stdout.trim();
    assert!(url.starts_with("http://localhost:8080/r/"), "{url}");
    assert!(!url.contains("claude"), "label never in the URL");
    assert!(
        stderr.contains("nothing outside this machine"),
        "localhost warning: {stderr}"
    );

    let (code, stdout, _) = hp(&["seed", "add", "rag-hr", "--format", "env"], p);
    assert_eq!(code, 0);
    assert!(stdout.starts_with("INTERNAL_DOCS_URL=http://localhost:8080/r/"));

    let (code, _, stderr) = hp(&["seed", "add", "claude-md-prod"], p);
    assert_ne!(code, 0);
    assert!(stderr.contains("already exists"), "{stderr}");
    let (code, _, stderr) = hp(&["seed", "add", "Bad Label"], p);
    assert_ne!(code, 0);
    assert!(stderr.contains("invalid label"), "{stderr}");

    let (code, stdout, _) = hp(&["seed", "list"], p);
    assert_eq!(code, 0);
    assert!(stdout.contains("claude-md-prod") && stdout.contains("rag-hr"));
    assert!(stdout.contains(url), "list shows the URL");
}
