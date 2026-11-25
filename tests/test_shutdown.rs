//! `serve` shuts down gracefully on SIGTERM and SIGHUP, not just SIGINT.
//! (In the container image the binary is PID 1, where an unhandled SIGTERM is
//! dropped, so `docker stop` used to end in SIGKILL.)

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// Start `serve`, wait until it accepts connections, send `sig`, and return
/// (exit status success, stdout, stderr). Retries if the port was taken.
fn serve_then_signal(sig: libc::c_int) -> (bool, String, String) {
    for _ in 0..5 {
        let dir = tempfile::tempdir().unwrap();
        let port = free_port();
        let bind = format!("127.0.0.1:{port}");
        let mut child = Command::new(env!("CARGO_BIN_EXE_honeyprompt"))
            .arg("serve")
            .arg(dir.path())
            .args(["--domain", "localhost", "--bind", &bind])
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
                break; // exited early (port taken?)
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if !up {
            let _ = child.kill();
            let _ = child.wait();
            continue;
        }

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
        let mut out = String::new();
        let mut err = String::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_string(&mut out)
            .unwrap();
        child
            .stderr
            .take()
            .unwrap()
            .read_to_string(&mut err)
            .unwrap();
        return (status.success(), out, err);
    }
    panic!("serve never started after 5 tries");
}

#[test]
fn sigterm_shuts_down_gracefully() {
    let (ok, out, err) = serve_then_signal(libc::SIGTERM);
    assert!(ok, "clean exit; stderr: {err}");
    assert!(err.contains("Shutting down (SIGTERM)"), "{err}");
    assert!(out.contains("Shutdown complete."), "summary printed: {out}");
}

#[test]
fn sighup_shuts_down_gracefully() {
    let (ok, out, err) = serve_then_signal(libc::SIGHUP);
    assert!(ok, "clean exit; stderr: {err}");
    assert!(err.contains("Shutting down (SIGHUP)"), "{err}");
    assert!(out.contains("Shutdown complete."), "{out}");
}
