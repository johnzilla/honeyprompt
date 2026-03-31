use std::process::Command;

/// CLI-04: `honeyprompt monitor --help` exits 0
#[test]
fn test_monitor_help_exits_zero() {
    let output = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
        .args(["monitor", "--help"])
        .output()
        .expect("failed to run honeyprompt");
    assert!(output.status.success(), "monitor --help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("monitor"),
        "help output should mention monitor"
    );
    assert!(
        stdout.contains("--attach"),
        "help output should mention --attach flag"
    );
}

/// CLI-04: `honeyprompt monitor` with nonexistent project dir fails with meaningful error
#[test]
fn test_monitor_missing_project_dir_fails() {
    let output = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
        .args(["monitor", "/tmp/nonexistent-honeyprompt-project-dir-12345"])
        .output()
        .expect("failed to run honeyprompt");
    assert!(!output.status.success(), "monitor with bad dir should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("honeyprompt.toml")
            || stderr.contains("not found")
            || stderr.contains("No such file"),
        "error should mention missing config: {}",
        stderr
    );
}

/// CLI-04: `honeyprompt monitor --attach` with nonexistent DB fails
#[test]
fn test_monitor_attach_missing_db_fails() {
    let output = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
        .args([
            "monitor",
            "--attach",
            "/tmp/nonexistent-honeyprompt-project-dir-12345",
        ])
        .output()
        .expect("failed to run honeyprompt");
    assert!(!output.status.success(), "attach with bad dir should fail");
}
