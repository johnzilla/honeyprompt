use tokio::sync::{broadcast, mpsc};

use crate::types::{AgentClass, AppEvent, RawCallbackEvent};

/// Receive RawCallbackEvents, enrich them with a session_id, and pass them to
/// the DB writer.
///
/// Pipeline: server → mpsc(256) → broker → mpsc(256) → `db_writer_task`, which
/// persists each event and only then broadcasts it (with the real replay flag
/// and fire count) to the TUI and stdout logger. The DB writer is never a
/// broadcast subscriber, so it can't lag and lose events.
///
/// The session_id is a salted SHA-256 of ip + ua (see `fingerprint::compute_session_id`).
pub async fn broker_task(
    mut callback_rx: mpsc::Receiver<RawCallbackEvent>,
    db_tx: mpsc::Sender<AppEvent>,
    session_salt: String,
) {
    while let Some(raw) = callback_rx.recv().await {
        let session_id = crate::fingerprint::compute_session_id(
            &session_salt,
            &raw.fingerprint.source_ip.to_string(),
            &raw.fingerprint.user_agent,
        );
        let app_event = AppEvent {
            exfil: raw.exfil,
            nonce: raw.nonce,
            tier: raw.tier,
            payload_id: raw.payload_id,
            embedding_loc: raw.embedding_loc,
            fingerprint: raw.fingerprint,
            classification: raw.classification,
            session_id,
            // Placeholders: db_writer_task sets the real values after insert.
            is_replay: false,
            fire_count: 1,
            received_at: raw.received_at,
            // Phase 13: propagate T4/T5 payload values so db_writer_task can
            // persist them. Missing any of these would silently drop T4/T5
            // data (RESEARCH Pitfall 4).
            t4_capability: raw.t4_capability,
            t5_proof: raw.t5_proof,
            t5_proof_valid: raw.t5_proof_valid,
            // Phase 14: propagate server-supplied t5_formula so the monitor
            // detail pane can render `formula=(seed+A)*B % M` (D-14-02).
            t5_formula: raw.t5_formula,
        };
        // Backpressure instead of loss: wait for the writer. (Overflow is
        // dropped, and counted, at the server's try_send.)
        if db_tx.send(app_event).await.is_err() {
            break; // writer gone: shutting down
        }
    }
}

/// Persist each AppEvent to SQLite, then broadcast it to the TUI and stdout
/// logger with `is_replay` and `fire_count` as stored. Exits (dropping
/// `out`, which closes the broadcast) once the broker's sender is gone and the
/// queue is drained.
pub async fn db_writer_task(
    mut event_rx: mpsc::Receiver<AppEvent>,
    conn: tokio_rusqlite::Connection,
    alerts: Option<crate::webhook::AlertSender>,
    out: broadcast::Sender<AppEvent>,
) {
    while let Some(mut event) = event_rx.recv().await {
        let extra_headers = build_extra_headers(&event.classification, &event.fingerprint.headers);
        let nonce = event.nonce.clone();
        let tier = event.tier;
        let payload_id = event.payload_id.clone();
        let embedding_loc = event.embedding_loc.clone();
        let session_id = event.session_id.clone();
        let remote_addr = event.fingerprint.source_ip.to_string();
        let user_agent = event.fingerprint.user_agent.clone();
        // Phase 13: clone T4/T5 payload fields so the move-closure owns
        // them; these replace the `None, None, None` placeholders from
        // Plan 02. `t5_proof_valid` is `Option<bool>` (Copy), so no
        // clone is needed.
        let t4_capability = event.t4_capability.clone();
        let t5_proof = event.t5_proof.clone();
        let t5_proof_valid = event.t5_proof_valid;
        let exfil = event.exfil;
        let result = conn
            .call(move |conn| {
                crate::store::insert_callback_event_detailed(
                    conn,
                    &nonce,
                    tier,
                    &payload_id,
                    &embedding_loc,
                    &session_id,
                    &remote_addr,
                    &user_agent,
                    &extra_headers,
                    t4_capability.as_deref(),
                    t5_proof.as_deref(),
                    t5_proof_valid,
                    exfil,
                )
                .map_err(tokio_rusqlite::Error::from)
            })
            .await;
        match result {
            Ok(hit) => {
                event.is_replay = hit.is_replay;
                event.fire_count = hit.fire_count;
                if let Some(alerts) = &alerts {
                    alerts.offer(crate::webhook::HitAlert {
                        received_at: hit.received_at,
                        tier: event.tier,
                        nonce: event.nonce.clone(),
                        payload_id: event.payload_id.clone(),
                        timing: hit.timing,
                        since_page_fetch_secs: hit.last_page_view_at.map(|pv| hit.received_at - pv),
                        since_seeding_secs: hit.seeded_at.map(|s| hit.received_at - s),
                        source_ip: event.fingerprint.source_ip.to_string(),
                        user_agent: event.fingerprint.user_agent.clone(),
                        classification: classify_label(&event.classification),
                        t5_proof_valid: event.t5_proof_valid,
                    });
                }
            }
            Err(e) => eprintln!("warning: db_writer failed to persist event: {}", e),
        }
        // No subscribers (serve without a logger, test-agent) is fine.
        let _ = out.send(event);
    }
}

/// Persist page fetches sent by the server's page-fetch middleware.
pub async fn page_view_writer_task(
    mut page_view_rx: mpsc::Receiver<crate::server::PageView>,
    conn: tokio_rusqlite::Connection,
) {
    while let Some(view) = page_view_rx.recv().await {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let result = conn
            .call(move |c| {
                crate::store::record_page_view(
                    c,
                    now,
                    &view.remote_addr,
                    &view.user_agent,
                    &view.path,
                )
                .map_err(tokio_rusqlite::Error::from)
            })
            .await;
        if let Err(e) = result {
            eprintln!("warning: failed to record page fetch: {}", e);
        }
    }
}

/// How the seed writer logs recorded seed hits to stdout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedLog {
    /// No output (the monitor's TUI owns the terminal).
    Quiet,
    Text,
    Json,
}

/// Persist fetches of seeded canaries and alert on the ones that are real seeds.
pub async fn seed_hit_writer_task(
    mut seed_hit_rx: mpsc::Receiver<crate::server::SeedHit>,
    conn: tokio_rusqlite::Connection,
    alerts: Option<crate::webhook::AlertSender>,
    log: SeedLog,
) {
    while let Some(hit) = seed_hit_rx.recv().await {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let h = hit.clone();
        let result = conn
            .call(move |c| {
                crate::store::record_seed_hit(
                    c,
                    &h.nonce,
                    now,
                    &h.remote_addr,
                    &h.user_agent,
                    &h.classification,
                )
                .map_err(tokio_rusqlite::Error::from)
            })
            .await;
        match result {
            Ok(Some(seed)) => {
                match log {
                    SeedLog::Quiet => {}
                    SeedLog::Text => println!(
                        "{} seed={} ip={} ua=\"{}\"",
                        now,
                        seed.label,
                        hit.remote_addr,
                        hit.user_agent.chars().take(60).collect::<String>()
                    ),
                    SeedLog::Json => println!(
                        "{}",
                        serde_json::json!({
                            "timestamp": now,
                            "event": "seed_hit",
                            "seed": &seed.label,
                            "ip": &hit.remote_addr,
                            "ua": &hit.user_agent,
                            "classification": &hit.classification,
                        })
                    ),
                }
                if let Some(alerts) = &alerts {
                    alerts.offer_seed(crate::webhook::SeedAlert {
                        received_at: now,
                        label: seed.label,
                        note: seed.note,
                        nonce: hit.nonce,
                        created_at: seed.created_at,
                        source_ip: hit.remote_addr,
                        user_agent: hit.user_agent,
                        classification: hit.classification,
                    });
                }
            }
            Ok(None) => {} // not a seed: nothing stored
            Err(e) => eprintln!("warning: failed to record seed hit: {}", e),
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
pub async fn stdout_logger_task(mut event_rx: broadcast::Receiver<AppEvent>, json_mode: bool) {
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
                    let ua_snippet: String =
                        event.fingerprint.user_agent.chars().take(60).collect();
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
pub fn classify_label(classification: &AgentClass) -> String {
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

    use crate::types::{AgentClass, AgentFingerprint, RawCallbackEvent, T5Formula};

    fn make_app_event(nonce: &str, tier: u8) -> AppEvent {
        let raw = make_raw_event(nonce, tier);
        AppEvent {
            exfil: raw.exfil,
            nonce: raw.nonce,
            tier: raw.tier,
            payload_id: raw.payload_id,
            embedding_loc: raw.embedding_loc,
            fingerprint: raw.fingerprint,
            classification: raw.classification,
            session_id: "session000000001".to_string(),
            is_replay: false,
            fire_count: 1,
            received_at: raw.received_at,
            t4_capability: None,
            t5_proof: None,
            t5_proof_valid: None,
            t5_formula: None,
        }
    }

    fn make_raw_event(nonce: &str, tier: u8) -> RawCallbackEvent {
        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let fp = AgentFingerprint {
            source_ip: ip,
            user_agent: "TestAgent/1.0".to_string(),
            headers: HashMap::new(),
            received_at: 1000u64,
        };
        RawCallbackEvent {
            exfil: None,
            nonce: nonce.to_string(),
            tier,
            payload_id: format!("t{}-html", tier),
            embedding_loc: "html_comment".to_string(),
            fingerprint: fp,
            classification: AgentClass::Unknown,
            received_at: 1000u64,
            t4_capability: None,
            t5_proof: None,
            t5_proof_valid: None,
            // Phase 14: helper defaults to None; tier-5-specific tests construct
            // their own RawCallbackEvent literal with Some(T5Formula { ... }).
            t5_formula: None,
        }
    }

    #[tokio::test]
    async fn test_broker_task_enriches_and_broadcasts() {
        let (callback_tx, callback_rx) = mpsc::channel::<RawCallbackEvent>(16);
        let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(16);

        // Spawn the broker
        tokio::spawn(broker_task(callback_rx, event_tx, "test-salt".to_string()));

        // Send a raw event
        let raw = make_raw_event("testnonce01", 1);
        callback_tx.send(raw).await.unwrap();

        // Receive the enriched AppEvent
        let app_event =
            tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
                .await
                .expect("timed out waiting for AppEvent")
                .expect("broadcast recv failed");

        assert_eq!(app_event.nonce, "testnonce01");
        assert_eq!(app_event.tier, 1);
        assert!(
            !app_event.session_id.is_empty(),
            "session_id should be computed"
        );
        assert_eq!(
            app_event.session_id.len(),
            16,
            "session_id should be 16-char hex"
        );
    }

    /// Phase 13: broker_task must propagate T4/T5 payload fields from RawCallbackEvent
    /// into AppEvent. A missing field here would silently drop T4/T5 capture data
    /// (RESEARCH Pitfall 4) because the db_writer_task reads these exact fields.
    #[tokio::test]
    async fn test_broker_task_propagates_t4_capability() {
        let (callback_tx, callback_rx) = mpsc::channel::<RawCallbackEvent>(16);
        let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(16);

        tokio::spawn(broker_task(callback_rx, event_tx, "test-salt".to_string()));

        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let fp = AgentFingerprint {
            source_ip: ip,
            user_agent: "TestAgent/1.0".to_string(),
            headers: HashMap::new(),
            received_at: 1000u64,
        };
        let raw = RawCallbackEvent {
            exfil: None,
            nonce: "aaaaaaaaaaaaaaaa".to_string(),
            tier: 4,
            payload_id: "t4-tools-meta".to_string(),
            embedding_loc: "meta_tag".to_string(),
            fingerprint: fp,
            classification: AgentClass::Unknown,
            received_at: 1_700_000_000,
            t4_capability: Some("web_search,browse_page".to_string()),
            t5_proof: None,
            t5_proof_valid: None,
            t5_formula: None,
        };
        callback_tx.send(raw).await.unwrap();

        let app_event =
            tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
                .await
                .expect("broker must broadcast within 500ms")
                .expect("broadcast must succeed");

        assert_eq!(
            app_event.t4_capability,
            Some("web_search,browse_page".to_string()),
            "broker must propagate t4_capability from raw to app"
        );
        assert_eq!(app_event.t5_proof, None);
        assert_eq!(app_event.t5_proof_valid, None);
        assert_eq!(app_event.tier, 4);
    }

    /// Phase 13: broker_task must also propagate T5 proof fields. This guards the
    /// symmetric Pitfall 4 regression for tier-5 callbacks.
    #[tokio::test]
    async fn test_broker_task_propagates_t5_proof() {
        let (callback_tx, callback_rx) = mpsc::channel::<RawCallbackEvent>(16);
        let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(16);

        tokio::spawn(broker_task(callback_rx, event_tx, "test-salt".to_string()));

        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        let fp = AgentFingerprint {
            source_ip: ip,
            user_agent: "TestAgent/1.0".to_string(),
            headers: HashMap::new(),
            received_at: 1000u64,
        };
        let raw = RawCallbackEvent {
            exfil: None,
            nonce: "bbbbbbbbbbbbbbbb".to_string(),
            tier: 5,
            payload_id: "t5-semantic-prose".to_string(),
            embedding_loc: "semantic_prose".to_string(),
            fingerprint: fp,
            classification: AgentClass::Unknown,
            received_at: 1_700_000_000,
            t4_capability: None,
            t5_proof: Some("731".to_string()),
            t5_proof_valid: Some(true),
            // Phase 14: broker must propagate the server-supplied formula.
            t5_formula: Some(T5Formula {
                a: 7,
                b: 13,
                modulus: 1000,
            }),
        };
        callback_tx.send(raw).await.unwrap();

        let app_event =
            tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
                .await
                .expect("broker must broadcast within 500ms")
                .expect("broadcast must succeed");

        assert_eq!(app_event.t4_capability, None);
        assert_eq!(app_event.t5_proof, Some("731".to_string()));
        assert_eq!(app_event.t5_proof_valid, Some(true));
        assert_eq!(app_event.tier, 5);
        assert_eq!(
            app_event.t5_formula,
            Some(T5Formula {
                a: 7,
                b: 13,
                modulus: 1000
            })
        );
    }

    /// Phase 14: broker_task must also propagate `t5_formula: None` faithfully.
    /// This covers attach-mode callers and T1/T4 handlers (all produce None).
    #[tokio::test]
    async fn test_broker_task_t5_formula_none_when_raw_is_none() {
        let (callback_tx, callback_rx) = mpsc::channel::<RawCallbackEvent>(8);
        let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(8);
        tokio::spawn(broker_task(callback_rx, event_tx, "test-salt".to_string()));
        // Helper defaults `t5_formula: None`; (nonce, tier) = ("n1", 1).
        let raw = make_raw_event("n1", 1);
        callback_tx.send(raw).await.unwrap();
        let received = tokio::time::timeout(std::time::Duration::from_millis(500), event_rx.recv())
            .await
            .expect("broker must broadcast within 500ms")
            .expect("broadcast must succeed");
        assert_eq!(received.t5_formula, None);
    }

    /// Review finding #5: the broadcast after insert carries the stored replay
    /// flag and fire count, not the broker's placeholders.
    #[tokio::test]
    async fn test_db_writer_broadcasts_real_fire_count_and_replay() {
        let tk_conn = tokio_rusqlite::Connection::open_in_memory().await.unwrap();
        tk_conn
            .call(|conn| {
                crate::store::run_migrations(conn)?;
                crate::store::insert_nonce(conn, "replay01", 1, "t1-html", "html_comment")?;
                Ok::<_, rusqlite::Error>(())
            })
            .await
            .unwrap();
        let (event_tx, event_rx) = mpsc::channel::<AppEvent>(16);
        let (out_tx, mut out_rx) = broadcast::channel::<AppEvent>(16);
        tokio::spawn(db_writer_task(event_rx, tk_conn.clone(), None, out_tx));

        for _ in 0..3 {
            let mut ev = make_app_event("replay01", 1);
            ev.session_id = "samesession00001".to_string();
            event_tx.send(ev).await.unwrap();
        }
        let mut seen = Vec::new();
        for _ in 0..3 {
            let ev = tokio::time::timeout(std::time::Duration::from_secs(2), out_rx.recv())
                .await
                .expect("timed out waiting for persisted event")
                .unwrap();
            seen.push((ev.fire_count, ev.is_replay));
        }
        assert_eq!(seen, vec![(1, false), (2, true), (3, true)]);

        // Closing the input drains the writer and closes the broadcast.
        drop(event_tx);
        let closed = tokio::time::timeout(std::time::Duration::from_secs(2), out_rx.recv()).await;
        assert!(matches!(
            closed,
            Ok(Err(broadcast::error::RecvError::Closed))
        ));
    }

    #[tokio::test]
    async fn test_db_writer_task_persists_event() {
        // Create a shared in-memory tokio-rusqlite connection
        let tk_conn = tokio_rusqlite::Connection::open_in_memory().await.unwrap();

        // Run migrations
        tk_conn
            .call(|conn| crate::store::run_migrations(conn).map_err(tokio_rusqlite::Error::from))
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

        let (event_tx, event_rx) = mpsc::channel::<AppEvent>(16);
        let (out_tx, _) = broadcast::channel::<AppEvent>(16);
        tokio::spawn(db_writer_task(event_rx, tk_conn.clone(), None, out_tx));

        // Send an AppEvent directly on broadcast
        let ip: IpAddr = "2.3.4.5".parse().unwrap();
        let app_event = AppEvent {
            exfil: None,
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
            t4_capability: None,
            t5_proof: None,
            t5_proof_valid: None,
            t5_formula: None,
        };
        event_tx.send(app_event).await.unwrap();

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
            exfil: None,
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
            t4_capability: None,
            t5_proof: None,
            t5_proof_valid: None,
            t5_formula: None,
        };
        // Send should succeed (receiver exists in spawned task)
        assert!(event_tx.send(app_event).is_ok());
        // Give task a moment to process
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    /// A stored hit is handed to the webhook with its timing label and context.
    #[tokio::test]
    async fn test_db_writer_task_sends_webhook_alert() {
        use std::sync::{Arc, Mutex};
        let received: Arc<Mutex<Vec<serde_json::Value>>> = Arc::default();
        let rec = received.clone();
        let app = axum::Router::new().route(
            "/hook",
            axum::routing::post(move |body: axum::body::Bytes| {
                let rec = rec.clone();
                async move {
                    rec.lock()
                        .unwrap()
                        .push(serde_json::from_slice(&body).unwrap());
                    axum::http::StatusCode::OK
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

        let config = crate::config::Config {
            callback_base_url: "https://honeyprompt.sh".into(),
            webhook: crate::webhook::WebhookConfig {
                url: Some(format!("http://127.0.0.1:{port}/hook")),
                ..Default::default()
            },
            ..Default::default()
        };
        let webhook = crate::webhook::Webhook::from_config(&config, |_| None)
            .unwrap()
            .unwrap();
        let alerts = crate::webhook::spawn(webhook).unwrap();

        let tk_conn = tokio_rusqlite::Connection::open_in_memory().await.unwrap();
        tk_conn
            .call(|conn| {
                crate::store::run_migrations(conn)?;
                crate::store::insert_nonce(conn, "hook0001", 2, "t2-x", "html_comment")?;
                Ok::<_, rusqlite::Error>(())
            })
            .await
            .unwrap();
        let (event_tx, event_rx) = mpsc::channel::<AppEvent>(16);
        let (out_tx, _) = broadcast::channel::<AppEvent>(16);
        tokio::spawn(db_writer_task(
            event_rx,
            tk_conn.clone(),
            Some(alerts),
            out_tx,
        ));

        let event = AppEvent {
            exfil: None,
            nonce: "hook0001".to_string(),
            tier: 2,
            payload_id: "t2-x".to_string(),
            embedding_loc: "html_comment".to_string(),
            fingerprint: AgentFingerprint {
                source_ip: "198.51.100.9".parse::<IpAddr>().unwrap(),
                user_agent: "Harvester/3.0".to_string(),
                headers: HashMap::new(),
                received_at: 2000u64,
            },
            classification: AgentClass::Unknown,
            session_id: "s".to_string(),
            is_replay: false,
            fire_count: 1,
            received_at: 2000u64,
            t4_capability: None,
            t5_proof: None,
            t5_proof_valid: None,
            t5_formula: None,
        };
        event_tx.send(event).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        let got = received.lock().unwrap();
        assert_eq!(got.len(), 1, "out-of-band hit is notable");
        let hit = &got[0]["hit"];
        assert_eq!(hit["timing"], "out-of-band");
        assert_eq!(hit["tier"], 2);
        assert_eq!(hit["source_ip"], "198.51.100.9");
        assert_eq!(hit["user_agent"], "Harvester/3.0");
        assert!(hit["since_seeding_secs"].as_i64().unwrap() >= 0);
        assert_eq!(got[0]["deployment"], "honeyprompt.sh");
    }

    /// Real seeds are stored and alerted; unknown nonces are ignored.
    #[tokio::test]
    async fn test_seed_hit_writer_records_and_alerts() {
        use std::sync::{Arc, Mutex};
        let received: Arc<Mutex<Vec<serde_json::Value>>> = Arc::default();
        let rec = received.clone();
        let app = axum::Router::new().route(
            "/hook",
            axum::routing::post(move |body: axum::body::Bytes| {
                let rec = rec.clone();
                async move {
                    rec.lock()
                        .unwrap()
                        .push(serde_json::from_slice(&body).unwrap());
                    axum::http::StatusCode::OK
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let config = crate::config::Config {
            webhook: crate::webhook::WebhookConfig {
                url: Some(format!("http://127.0.0.1:{port}/hook")),
                ..Default::default()
            },
            ..Default::default()
        };
        let alerts = crate::webhook::spawn(
            crate::webhook::Webhook::from_config(&config, |_| None)
                .unwrap()
                .unwrap(),
        )
        .unwrap();

        let tk_conn = tokio_rusqlite::Connection::open_in_memory().await.unwrap();
        tk_conn
            .call(|c| {
                crate::store::run_migrations(c)?;
                crate::store::insert_seed(c, "aaaaaaaaaaaaaaaa", "claude-md", Some("repo"), 1)?;
                Ok::<_, rusqlite::Error>(())
            })
            .await
            .unwrap();
        let (tx, rx) = mpsc::channel(16);
        tokio::spawn(seed_hit_writer_task(
            rx,
            tk_conn.clone(),
            Some(alerts),
            SeedLog::Quiet,
        ));
        for nonce in ["aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb"] {
            tx.send(crate::server::SeedHit {
                nonce: nonce.into(),
                remote_addr: "198.51.100.9".into(),
                user_agent: "Harvester/3.0".into(),
                classification: "Unknown".into(),
            })
            .await
            .unwrap();
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        let stored: u32 = tk_conn
            .call(|c| {
                c.query_row("SELECT COUNT(*) FROM seed_hits", [], |r| r.get(0))
                    .map_err(tokio_rusqlite::Error::from)
            })
            .await
            .unwrap();
        assert_eq!(stored, 1, "only the real seed is stored");
        let got = received.lock().unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0]["seed"]["label"], "claude-md");
        assert_eq!(got[0]["hit"]["user_agent"], "Harvester/3.0");
    }
}
