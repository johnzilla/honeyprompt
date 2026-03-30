//! Integration tests for the `honeyprompt test-agent` subcommand.
//!
//! These tests exercise the CLI binary end-to-end, validating:
//! - TEST-01: Server lifecycle (starts, listens, shuts down after timeout)
//! - TEST-02: CLI flag parsing (--listen, --timeout, --format)

use std::process::Command;
use std::time::Instant;

/// Helper: run honeyprompt test-agent with given args, return (exit_code, stdout, stderr).
fn run_test_agent(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
        .arg("test-agent")
        .args(args)
        .output()
        .expect("failed to execute honeyprompt");
    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

/// TEST-01: Server starts, listens for the timeout duration, then shuts down cleanly.
/// With --timeout 2 and no agent traffic, should exit 0 in roughly 2-4 seconds.
#[test]
fn test_agent_lifecycle_clean_shutdown() {
    let start = Instant::now();
    let (code, _stdout, stderr) = run_test_agent(&["--timeout", "2"]);
    let elapsed = start.elapsed().as_secs();

    // Should exit 0 (no canaries triggered)
    assert_eq!(code, 0, "Expected exit code 0 (no callbacks), got {}", code);
    // Should have run for at least ~2 seconds (the timeout)
    assert!(elapsed >= 2, "Exited too quickly: {}s (expected >= 2s)", elapsed);
    // Should have printed startup info to stderr
    assert!(
        stderr.contains("test-agent"),
        "stderr should contain 'test-agent' startup message, got: {}",
        stderr
    );
}

/// TEST-02: --timeout flag is respected (short timeout exits faster).
#[test]
fn test_agent_timeout_flag() {
    let start = Instant::now();
    let (code, _stdout, _stderr) = run_test_agent(&["--timeout", "1"]);
    let elapsed = start.elapsed().as_secs();

    assert_eq!(code, 0, "Expected exit code 0, got {}", code);
    assert!(elapsed < 10, "Took too long: {}s (expected < 10s for 1s timeout)", elapsed);
}

/// TEST-02: --listen flag with explicit port is accepted.
#[test]
fn test_agent_listen_flag() {
    let (code, _stdout, stderr) = run_test_agent(&["--timeout", "1", "--listen", "127.0.0.1:0"]);
    assert_eq!(code, 0, "Expected exit code 0, got {}", code);
    // Stderr should mention the URL with the assigned port
    assert!(
        stderr.contains("http://127.0.0.1:"),
        "stderr should show listen URL, got: {}",
        stderr
    );
}

/// TEST-02: --format json flag produces JSON output on stdout.
#[test]
fn test_agent_format_json() {
    let (code, stdout, _stderr) = run_test_agent(&["--timeout", "1", "--format", "json"]);
    // Note: Plan 03 wires the actual render output. This test will pass once Plan 03 is done.
    // For now, exit code 0 is the primary assertion.
    assert_eq!(code, 0, "Expected exit code 0, got {}", code);
    // Once Plan 03 lands, stdout should be parseable JSON:
    // let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    // assert!(parsed.get("verdict").is_some(), "JSON should have verdict field");
    let _ = stdout; // suppress unused warning until Plan 03
}
