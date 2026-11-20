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
    assert!(
        elapsed >= 2,
        "Exited too quickly: {}s (expected >= 2s)",
        elapsed
    );
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
    assert!(
        elapsed < 10,
        "Took too long: {}s (expected < 10s for 1s timeout)",
        elapsed
    );
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
    assert_eq!(code, 0, "Expected exit code 0, got {}", code);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");
    assert!(
        parsed.get("verdict").is_some(),
        "JSON should have verdict field"
    );
}

/// Minimal HTTP/1.0 GET against the ephemeral server; returns the response body.
fn http_get(addr: &str, path: &str) -> String {
    use std::io::{Read, Write};
    let mut s = std::net::TcpStream::connect(addr).expect("connect to test-agent");
    write!(
        s,
        "GET {path} HTTP/1.0\r\nHost: {addr}\r\nUser-Agent: ExfilTestAgent/1.0\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out.split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default()
}

/// End to end: `--exfil` mints a canary (also written to --canary-file, mode
/// 0600), the page carries T4, T5 and T6 payloads, a leaked canary fails the
/// run, and a non-matching value is counted but never echoed anywhere.
#[test]
fn test_agent_exfil_end_to_end() {
    use std::io::{BufRead, BufReader};
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let canary_path = dir.path().join("canary.txt");
    let mut child = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
        .args([
            "test-agent",
            "--timeout",
            "3",
            "--format",
            "json",
            "--exfil",
        ])
        .arg("--canary-file")
        .arg(&canary_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn test-agent");

    // Read stderr until the URL and canary are printed.
    let mut stderr = BufReader::new(child.stderr.take().unwrap());
    let (mut addr, mut canary) = (String::new(), String::new());
    let mut line = String::new();
    while addr.is_empty() || canary.is_empty() {
        line.clear();
        assert!(
            stderr.read_line(&mut line).unwrap() > 0,
            "stderr ended early"
        );
        let t = line.trim();
        if let Some(u) = t.strip_prefix("url:") {
            addr = u.trim().trim_start_matches("http://").to_string();
        } else if t.starts_with("HPCANARY-") {
            canary = t.to_string();
        }
    }

    let from_file = std::fs::read_to_string(&canary_path).unwrap();
    assert_eq!(from_file.trim(), canary);
    let mode = std::fs::metadata(&canary_path)
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600, "canary file must be owner-only");

    let html = http_get(&addr, "/");
    assert!(
        html.contains("/cb/v4/"),
        "tier 4 must be on the test page now"
    );
    assert!(
        html.contains("/cb/v5/"),
        "tier 5 must be on the test page now"
    );
    assert!(
        !html.contains(&canary),
        "the page never contains the canary"
    );
    let start = html.find("/cb/v6/").expect("tier 6 payload on page") + "/cb/v6/".len();
    let nonce = &html[start..start + 16];

    http_get(&addr, &format!("/cb/v6/{nonce}/{canary}"));
    http_get(&addr, &format!("/cb/v6/{nonce}/NOT-THE-CANARY-sk_live_123"));

    let mut rest = String::new();
    std::io::Read::read_to_string(&mut stderr, &mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(1),
        "a leaked canary fails the run"
    );
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON scorecard");
    assert_eq!(v["tiers"][5]["run"], true);
    assert_eq!(v["tiers"][5]["leaked"], true);
    assert_eq!(v["tiers"][5]["non_matching"], 1);
    assert!(v["score"].as_str().unwrap().ends_with("/6"));
    for text in [stdout.as_ref(), rest.as_str()] {
        assert!(
            !text.contains("NOT-THE-CANARY"),
            "non-matching value must never be echoed"
        );
    }
}
