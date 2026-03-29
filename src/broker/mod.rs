use tokio::sync::{broadcast, mpsc};

use crate::types::{AgentClass, AppEvent, RawCallbackEvent};

/// Receive RawCallbackEvents from mpsc, enrich with session_id, and broadcast AppEvent to all subscribers.
///
/// The session_id is computed as SHA-256(ip + ua) using the fingerprint module.
/// Broadcast send errors (no receivers) are silently dropped — events are best-effort.
pub async fn broker_task(
    mut callback_rx: mpsc::Receiver<RawCallbackEvent>,
    event_tx: broadcast::Sender<AppEvent>,
) {
    while let Some(raw) = callback_rx.recv().await {
        let session_id = crate::fingerprint::compute_session_id(
            &raw.fingerprint.source_ip.to_string(),
            &raw.fingerprint.user_agent,
        );
        let app_event = AppEvent {
            nonce: raw.nonce,
            tier: raw.tier,
            payload_id: raw.payload_id,
            embedding_loc: raw.embedding_loc,
            fingerprint: raw.fingerprint,
            classification: raw.classification,
            session_id,
            is_replay: false,
            fire_count: 1,
            received_at: raw.received_at,
        };
        // Ignore send errors — no receivers means events are dropped safely.
        let _ = event_tx.send(app_event);
    }
}

/// Receive AppEvents from broadcast and persist them to SQLite via tokio-rusqlite.
///
/// Handles broadcast lag gracefully: a warning is printed to stderr if events were dropped
/// (Pitfall 4 from research). Exits when the broadcast channel is closed.
pub async fn db_writer_task(
    mut event_rx: broadcast::Receiver<AppEvent>,
    conn: tokio_rusqlite::Connection,
) {
    loop {
        match event_rx.recv().await {
            Ok(event) => {
                let extra_headers = build_extra_headers(&event.classification, &event.fingerprint.headers);
                let nonce = event.nonce.clone();
                let tier = event.tier;
                let payload_id = event.payload_id.clone();
                let embedding_loc = event.embedding_loc.clone();
                let session_id = event.session_id.clone();
                let remote_addr = event.fingerprint.source_ip.to_string();
                let user_agent = event.fingerprint.user_agent.clone();
                let result = conn
                    .call(move |conn| {
                        crate::store::insert_callback_event(
                            conn,
                            &nonce,
                            tier,
                            &payload_id,
                            &embedding_loc,
                            &session_id,
                            &remote_addr,
                            &user_agent,
                            &extra_headers,
                        )
                        .map_err(tokio_rusqlite::Error::from)
                    })
                    .await;
                if let Err(e) = result {
                    eprintln!("warning: db_writer failed to persist event: {}", e);
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                eprintln!("warning: db writer lagged, dropped {} events", n);
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }
}

/// Receive AppEvents from broadcast and print structured log lines to stdout.
///
/// If `json_mode` is true, emits JSON lines (one per event).
/// Otherwise emits a human-readable one-liner per D-10:
///   `{timestamp} tier={tier} class={classification} ip={ip} ua="{ua_snippet}"`
/// where ua_snippet is the first 60 characters of the user-agent.
///
/// Handles broadcast lag gracefully. Exits when the channel is closed.
pub async fn stdout_logger_task(
    mut event_rx: broadcast::Receiver<AppEvent>,
    json_mode: bool,
) {
    loop {
        match event_rx.recv().await {
            Ok(event) => {
                if json_mode {
                    let classification = classify_label(&event.classification);
                    let log_entry = serde_json::json!({
                        "timestamp": event.received_at,
                        "tier": event.tier,
                        "classification": classification,
                        "ip": event.fingerprint.source_ip.to_string(),
                        "user_agent": event.fingerprint.user_agent,
                        "session_id": event.session_id,
                        "nonce": event.nonce,
                        "is_replay": event.is_replay,
                        "fire_count": event.fire_count,
                    });
                    println!("{}", serde_json::to_string(&log_entry).unwrap_or_default());
                } else {
                    let classification = classify_label(&event.classification);
                    let ua_snippet: String = event
                        .fingerprint
                        .user_agent
                        .chars()
                        .take(60)
                        .collect();
                    println!(
                        "{} tier={} class={} ip={} ua=\"{}\"",
                        event.received_at,
                        event.tier,
                        classification,
                        event.fingerprint.source_ip,
                        ua_snippet,
                    );
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                eprintln!("warning: stdout logger lagged, dropped {} events", n);
            }
            Err(broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }
}

/// Build the extra_headers JSON blob containing classification and headers map.
///
/// Format: `{"classification": "...", "headers": {...}}`
fn build_extra_headers(
    classification: &AgentClass,
    headers: &std::collections::HashMap<String, String>,
) -> String {
    let label = classify_label(classification);
    let extra = serde_json::json!({
        "classification": label,
        "headers": headers,
    });
    extra.to_string()
}

/// Convert AgentClass to a stable string label for storage/logging.
fn classify_label(classification: &AgentClass) -> String {
    match classification {
        AgentClass::KnownCrawler { provider } => format!("KnownCrawler:{}", provider),
        AgentClass::KnownAgent { provider } => format!("KnownAgent:{}", provider),
        AgentClass::Unknown => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::net::IpAddr;
    use tokio::sync::{broadcast, mpsc};

    use crate::types::{AgentClass, AgentFingerprint, RawCallbackEvent};

    fn make_raw_event(nonce: &str, tier: u8) -> RawCallbackEvent {
        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let fp = AgentFingerprint {
            source_ip: ip,
            user_agent: "TestAgent/1.0".to_string(),
            headers: HashMap::new(),
            received_at: 1000u64,
        };
        RawCallbackEvent {
            nonce: nonce.to_string(),
            tier,
            payload_id: format!("t{}-html", tier),
            embedding_loc: "html_comment".to_string(),
            fingerprint: fp,
            classification: AgentClass::Unknown,
            received_at: 1000u64,
        }
    }

    #[tokio::test]
    async fn test_broker_task_enriches_and_broadcasts() {
        let (callback_tx, callback_rx) = mpsc::channel::<RawCallbackEvent>(16);
        let (event_tx, mut event_rx) = broadcast::channel::<AppEvent>(16);

        // Spawn the broker
        tokio::spawn(broker_task(callback_rx, event_tx));

        // Send a raw event
        let raw = make_raw_event("testnonce01", 1);
        callback_tx.send(raw).await.unwrap();

        // Receive the enriched AppEvent
        let app_event = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            event_rx.recv(),
        )
        .await
        .expect("timed out waiting for AppEvent")
        .expect("broadcast recv failed");

        assert_eq!(app_event.nonce, "testnonce01");
        assert_eq!(app_event.tier, 1);
        assert!(!app_event.session_id.is_empty(), "session_id should be computed");
        assert_eq!(app_event.session_id.len(), 16, "session_id should be 16-char hex");
    }

    #[tokio::test]
    async fn test_db_writer_task_persists_event() {
        // Create a shared in-memory tokio-rusqlite connection
        let tk_conn = tokio_rusqlite::Connection::open_in_memory().await.unwrap();

        // Run migrations
        tk_conn
            .call(|conn| {
                crate::store::run_migrations(conn).map_err(tokio_rusqlite::Error::from)
            })
            .await
            .unwrap();

        // Insert a nonce so the lookup works
        tk_conn
            .call(|conn| {
                crate::store::insert_nonce(conn, "dbwrite01", 1, "t1-html", "html_comment")
                    .map_err(tokio_rusqlite::Error::from)
            })
            .await
            .unwrap();

        let (event_tx, event_rx) = broadcast::channel::<AppEvent>(16);
        tokio::spawn(db_writer_task(event_rx, tk_conn.clone()));

        // Send an AppEvent directly on broadcast
        let ip: IpAddr = "2.3.4.5".parse().unwrap();
        let app_event = AppEvent {
            nonce: "dbwrite01".to_string(),
            tier: 1,
            payload_id: "t1-html".to_string(),
            embedding_loc: "html_comment".to_string(),
            fingerprint: AgentFingerprint {
                source_ip: ip,
                user_agent: "DBWriterBot/1.0".to_string(),
                headers: HashMap::new(),
                received_at: 2000u64,
            },
            classification: AgentClass::Unknown,
            session_id: "aabbccdd11223344".to_string(),
            is_replay: false,
            fire_count: 1,
            received_at: 2000u64,
        };
        event_tx.send(app_event).unwrap();

        // Wait for DB write to complete
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Verify the event was written
        let fire_count: u32 = tk_conn
            .call(|conn| {
                conn.query_row(
                    "SELECT fire_count FROM events WHERE nonce = ?1",
                    rusqlite::params!["dbwrite01"],
                    |row| row.get(0),
                )
                .map_err(tokio_rusqlite::Error::from)
            })
            .await
            .unwrap();
        assert_eq!(fire_count, 1);
    }

    #[tokio::test]
    async fn test_stdout_logger_task_formats_log_line() {
        // This test verifies the logger doesn't panic and processes events.
        // We spawn the task and send an event; we can't easily capture stdout in unit tests,
        // but we verify the task processes the event without errors.
        let (event_tx, event_rx) = broadcast::channel::<AppEvent>(16);
        tokio::spawn(stdout_logger_task(event_rx, false));

        let ip: IpAddr = "3.4.5.6".parse().unwrap();
        let app_event = AppEvent {
            nonce: "logtest01".to_string(),
            tier: 2,
            payload_id: "t2-cond".to_string(),
            embedding_loc: "meta_tag".to_string(),
            fingerprint: AgentFingerprint {
                source_ip: ip,
                user_agent: "LoggerTestBot/1.0 with a very long user agent string that exceeds sixty characters".to_string(),
                headers: HashMap::new(),
                received_at: 3000u64,
            },
            classification: AgentClass::KnownCrawler { provider: "OpenAI".to_string() },
            session_id: "11223344aabbccdd".to_string(),
            is_replay: false,
            fire_count: 1,
            received_at: 3000u64,
        };
        // Send should succeed (receiver exists in spawned task)
        assert!(event_tx.send(app_event).is_ok());
        // Give task a moment to process
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}
