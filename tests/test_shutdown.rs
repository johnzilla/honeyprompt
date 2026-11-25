//! End-to-end tests against the real `serve` binary: graceful shutdown on
//! SIGTERM/SIGHUP (PID 1 in the container drops unhandled SIGTERM), queued
//! events drained before exit, the `--json` log carrying real replay data,
//! regeneration when tiers change, and refusing to start without a config.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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

/// Start `serve <dir> --bind 127.0.0.1:0 <extra>`, wait for the `listen:`
/// line with the port it actually bound, call `during(addr)`, send `sig`, and
/// collect the output. Binding port 0 inside `serve` avoids the race where a
/// port picked by the test is taken by a parallel test before `serve` binds it
/// (the test then talked to another test's server).
fn serve_run(dir: &Path, extra: &[&str], sig: libc::c_int, during: impl Fn(&str)) -> Run {
    use std::io::BufRead;
    let mut child = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
        .arg("serve")
        .arg(dir)
        .args(extra)
        .args(["--bind", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn serve");

    // Read stdout on a thread: report the listen address, keep every line.
    let stdout = child.stdout.take().unwrap();
    let (addr_tx, addr_rx) = std::sync::mpsc::channel::<String>();
    let reader = std::thread::spawn(move || {
        let mut all = String::new();
        for line in std::io::BufReader::new(stdout).lines() {
            let line = line.unwrap_or_default();
            if let Some(a) = line.trim().strip_prefix("listen:") {
                let _ = addr_tx.send(a.trim().to_string());
            }
            all.push_str(&line);
            all.push('\n');
        }
        all
    });

    let addr = match addr_rx.recv_timeout(Duration::from_secs(15)) {
        Ok(a) => a,
        Err(_) => {
            let _ = child.kill();
            let out = child.wait_with_output().unwrap();
            panic!(
                "serve never reported its listen address: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    };

    during(&addr);

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
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    Run {
        ok: status.success(),
        stdout: reader.join().unwrap(),
        stderr,
    }
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

/// `--domain` accepts only a host or host:port.
#[test]
fn malformed_domain_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    for bad in ["x/evil", "evil.example?a=b", "\"><script>"] {
        let out = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
            .arg("serve")
            .arg(dir.path())
            .args(["--domain", bad])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "clap usage error for {bad}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("not a host name"), "{bad}: {err}");
        assert!(!dir.path().join("honeyprompt.toml").exists());
    }
}
