//! End-to-end tests against the real `serve` binary: graceful shutdown on
//! SIGTERM/SIGHUP (PID 1 in the container drops unhandled SIGTERM), queued
//! events drained before exit, the `--json` log carrying real replay data,
//! regeneration when tiers change, and refusing to start without a config.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Minimal HTTP/1.0 GET; returns the body.
fn http_get(addr: &str, path: &str) -> String {
    let mut s = std::net::TcpStream::connect(addr).expect("connect");
    write!(
        s,
        "GET {path} HTTP/1.0\r\nHost: {addr}\r\nUser-Agent: ShutdownTest/1.0\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out.split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default()
}

struct Run {
    ok: bool,
    stdout: String,
    stderr: String,
}

/// Start `serve <dir> --bind 127.0.0.1:<port> <extra>`, wait until it accepts
/// connections, call `during(addr)`, send `sig`, and collect the output.
/// Retries on a fresh port if another test took it.
fn serve_run(dir: &Path, extra: &[&str], sig: libc::c_int, during: impl Fn(&str)) -> Run {
    for _ in 0..5 {
        let port = free_port();
        let bind = format!("127.0.0.1:{port}");
        let mut child = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
            .arg("serve")
            .arg(dir)
            .args(extra)
            .args(["--bind", &bind])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn serve");

        let start = Instant::now();
        let mut up = false;
        while start.elapsed() < Duration::from_secs(15) {
            if std::net::TcpStream::connect(&bind).is_ok() {
                up = true;
                break;
            }
            if child.try_wait().unwrap().is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if !up {
            let _ = child.kill();
            let out = child.wait_with_output().unwrap();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if stderr.contains("Address already in use") {
                continue;
            }
            panic!("serve never came up: {stderr}");
        }

        during(&bind);

        // SAFETY: sending a signal to our own child process.
        assert_eq!(unsafe { libc::kill(child.id() as libc::pid_t, sig) }, 0);
        let deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(s) = child.try_wait().unwrap() {
                break s;
            }
            if Instant::now() > deadline {
                let _ = child.kill();
                panic!("serve did not exit within 10 s of signal {sig}");
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        let mut stdout = String::new();
        let mut stderr = String::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_string(&mut stdout)
            .unwrap();
        child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();
        return Run {
            ok: status.success(),
            stdout,
            stderr,
        };
    }
    panic!("no free port after 5 tries");
}

#[test]
fn sigterm_shuts_down_gracefully() {
    let dir = tempfile::tempdir().unwrap();
    let r = serve_run(
        dir.path(),
        &["--domain", "localhost"],
        libc::SIGTERM,
        |_| {},
    );
    assert!(r.ok, "clean exit; stderr: {}", r.stderr);
    assert!(r.stderr.contains("Shutting down (SIGTERM)"), "{}", r.stderr);
    assert!(r.stdout.contains("Shutdown complete."), "{}", r.stdout);
}

#[test]
fn sighup_shuts_down_gracefully() {
    let dir = tempfile::tempdir().unwrap();
    let r = serve_run(dir.path(), &["--domain", "localhost"], libc::SIGHUP, |_| {});
    assert!(r.ok, "clean exit; stderr: {}", r.stderr);
    assert!(r.stderr.contains("Shutting down (SIGHUP)"), "{}", r.stderr);
    assert!(r.stdout.contains("Shutdown complete."), "{}", r.stdout);
}

/// Review #5 + drain: three hits on one nonce, then SIGTERM straight away.
/// The `--json` log shows fire counts 1, 2, 3 with the repeats marked as
/// replays, and the shutdown summary includes the hit (it was written before
/// the process exited, not lost in a queue).
#[test]
fn json_log_has_real_replay_data_and_queued_events_are_drained() {
    let dir = tempfile::tempdir().unwrap();
    let r = serve_run(
        dir.path(),
        &["--domain", "localhost", "--json"],
        libc::SIGTERM,
        |addr| {
            let page = http_get(addr, "/");
            let start = page.find("/cb/v1/").expect("tier 1-3 payload on page") + 7;
            let nonce = page[start..start + 16].to_string();
            for _ in 0..3 {
                http_get(addr, &format!("/cb/v1/{nonce}"));
            }
        },
    );
    assert!(r.ok, "{}", r.stderr);
    let hits: Vec<serde_json::Value> = r
        .stdout
        .lines()
        .filter(|l| l.starts_with('{') && l.contains("\"fire_count\""))
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let seen: Vec<(u64, bool)> = hits
        .iter()
        .map(|h| {
            (
                h["fire_count"].as_u64().unwrap(),
                h["is_replay"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(seen, vec![(1, false), (2, true), (3, true)], "{}", r.stdout);
    assert!(
        r.stdout
            .contains("Shutdown complete. 1 detection(s) recorded."),
        "{}",
        r.stdout
    );
}

/// Review #6: a changed tier list regenerates the site instead of being ignored.
#[test]
fn changed_tiers_regenerate_the_site() {
    let dir = tempfile::tempdir().unwrap();
    // First start creates the project (default tiers 1-5).
    let r = serve_run(
        dir.path(),
        &["--domain", "localhost"],
        libc::SIGTERM,
        |_| {},
    );
    assert!(r.ok, "{}", r.stderr);
    assert_eq!(
        honeyprompt::generator::generated_tiers(dir.path()).unwrap(),
        vec![1, 2, 3, 4, 5]
    );

    let r = serve_run(dir.path(), &["--tiers", "1"], libc::SIGTERM, |_| {});
    assert!(r.ok, "{}", r.stderr);
    assert!(
        r.stdout
            .contains("tiers changed ([1, 2, 3, 4, 5] -> [1]); regenerating site"),
        "{}",
        r.stdout
    );
    assert_eq!(
        honeyprompt::generator::generated_tiers(dir.path()).unwrap(),
        vec![1]
    );

    // Same tiers again: no regeneration.
    let r = serve_run(dir.path(), &["--tiers", "1"], libc::SIGTERM, |_| {});
    assert!(!r.stdout.contains("tiers changed"), "{}", r.stdout);
}

/// Review #4: no config and no --domain is an error, not a guessed host.
#[test]
fn serve_without_config_or_domain_refuses_to_start() {
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
        .arg("serve")
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("no honeyprompt.toml") && err.contains("--domain"),
        "{err}"
    );
}
