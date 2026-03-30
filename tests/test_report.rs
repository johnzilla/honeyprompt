use honeyprompt::{report, store};
use tempfile::NamedTempFile;

/// Helper: open a real SQLite DB via tempfile (matches real usage).
fn temp_conn() -> (NamedTempFile, rusqlite::Connection) {
    let tmp = NamedTempFile::new().expect("temp file must be created");
    let conn = store::open_or_create_db(tmp.path()).expect("DB must open");
    (tmp, conn)
}

/// Helper: insert a test event into the DB.
#[allow(clippy::too_many_arguments)]
fn insert_event(
    conn: &rusqlite::Connection,
    nonce: &str,
    tier: u8,
    payload_id: &str,
    embedding_loc: &str,
    session_id: &str,
    remote_addr: &str,
    user_agent: &str,
    classification: &str,
    first_seen_epoch: Option<u64>,
) {
    // Insert nonce_map entry
    let ts = first_seen_epoch.unwrap_or(1_700_000_000);
    conn.execute(
        "INSERT OR IGNORE INTO nonce_map (nonce, tier, payload_id, embedding_loc, generated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![nonce, tier, payload_id, embedding_loc, ts.to_string()],
    )
    .unwrap();
    // Insert events entry directly with epoch string
    let extra = format!(
        r#"{{"classification":"{}","headers":{{}}}}"#,
        classification
    );
    conn.execute(
        "INSERT INTO events (nonce, tier, payload_id, embedding_loc, first_seen_at, last_seen_at, fire_count, is_replay, session_id, remote_addr, user_agent, extra_headers)
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, 1, 0, ?6, ?7, ?8, ?9)",
        rusqlite::params![nonce, tier, payload_id, embedding_loc, ts.to_string(), session_id, remote_addr, user_agent, extra],
    )
    .unwrap();
}

// ---------- md_escape tests ----------

#[test]
fn test_report_md_escape_pipe() {
    // md_escape("pipe|char") returns "pipe\|char"
    assert_eq!(report::md_escape("pipe|char"), r"pipe\|char");
}

#[test]
fn test_report_md_escape_newline() {
    // md_escape("new\nline") returns "new line"
    assert_eq!(report::md_escape("new\nline"), "new line");
}

#[test]
fn test_report_md_escape_backtick() {
    // md_escape("back`tick") returns "back\`tick"
    assert_eq!(report::md_escape("back`tick"), r"back\`tick");
}

#[test]
fn test_report_md_escape_carriage_return() {
    // \r is removed
    assert_eq!(report::md_escape("foo\rbar"), "foobar");
}

// ---------- generate_report: empty DB ----------

#[test]
fn test_report_empty_db() {
    let (_tmp, conn) = temp_conn();
    let md = report::generate_report(&conn).expect("generate_report must not fail on empty DB");
    // Must be valid Markdown with Executive Summary header
    assert!(
        md.contains("## Executive Summary"),
        "must contain Executive Summary"
    );
    // Zero counts
    assert!(
        md.contains("| Total Detection Sessions |") && (md.contains("| 0 |") || md.contains(" 0 ")),
        "must indicate zero detection sessions"
    );
}

// ---------- generate_report: populated DB ----------

#[test]
fn test_report_with_events() {
    let (_tmp, conn) = temp_conn();
    // 2 Unknown sessions (tier 1 and tier 2) + 1 KnownCrawler session
    insert_event(
        &conn,
        "n001",
        1,
        "p1",
        "html_comment",
        "sess-a",
        "1.1.1.1",
        "AgentX/1.0",
        "Unknown",
        None,
    );
    insert_event(
        &conn,
        "n002",
        2,
        "p2",
        "meta_tag",
        "sess-b",
        "2.2.2.2",
        "AgentY/1.0",
        "Unknown",
        None,
    );
    insert_event(
        &conn,
        "n003",
        1,
        "p1",
        "html_comment",
        "sess-c",
        "3.3.3.3",
        "Googlebot/2.1",
        "KnownCrawler:Google",
        None,
    );

    let md = report::generate_report(&conn).expect("generate_report must succeed");

    assert!(
        md.contains("## Executive Summary"),
        "must have Executive Summary"
    );
    assert!(md.contains("## Evidence Table"), "must have Evidence Table");
    assert!(
        md.contains("## Known Crawler Sessions"),
        "must have Known Crawler Sessions section"
    );

    // Detection sessions = 2 (excluding KnownCrawler)
    assert!(
        md.contains("| Total Detection Sessions | 2 |"),
        "detection_sessions must be 2"
    );
}

// ---------- generate_report: crawlers separated ----------

#[test]
fn test_report_separates_crawlers() {
    let (_tmp, conn) = temp_conn();
    insert_event(
        &conn,
        "n010",
        1,
        "p1",
        "html_comment",
        "sess-agent",
        "10.0.0.1",
        "AgentBot/1.0",
        "Unknown",
        None,
    );
    insert_event(
        &conn,
        "n011",
        1,
        "p1",
        "html_comment",
        "sess-crawler",
        "10.0.0.2",
        "Googlebot/2.1",
        "KnownCrawler:Google",
        None,
    );

    let md = report::generate_report(&conn).expect("generate_report must succeed");

    // sess-agent should appear in evidence table (before Known Crawler Sessions section)
    let evidence_section_start = md
        .find("## Evidence Table")
        .expect("must have Evidence Table");
    let crawler_section_start = md
        .find("## Known Crawler Sessions")
        .expect("must have Known Crawler Sessions");

    let evidence_section = &md[evidence_section_start..crawler_section_start];
    let crawler_section = &md[crawler_section_start..];

    // sess-agent must NOT be in crawler section
    assert!(
        !crawler_section.contains("sess-agent"),
        "agent must NOT appear in crawler section"
    );
    // sess-crawler must NOT be in evidence table
    assert!(
        !evidence_section.contains("sess-craw"),
        "crawler must NOT appear in evidence table"
    );
}

// ---------- generate_report: session-based counting ----------

#[test]
fn test_report_session_based_counting() {
    let (_tmp, conn) = temp_conn();
    // Two events with same session_id + tier — summary should show 1 session, not 2
    insert_event(
        &conn,
        "n020",
        1,
        "p1",
        "html_comment",
        "sess-same",
        "1.1.1.1",
        "Agent/1.0",
        "Unknown",
        None,
    );
    insert_event(
        &conn,
        "n021",
        1,
        "p2",
        "meta_tag",
        "sess-same",
        "1.1.1.1",
        "Agent/1.0",
        "Unknown",
        None,
    );

    let md = report::generate_report(&conn).expect("generate_report must succeed");

    // Both events share session_id + tier 1 — should count as 1 unique (session, tier) pair
    assert!(
        md.contains("| Total Detection Sessions | 1 |"),
        "two events with same session+tier must count as 1 detection"
    );
}

// ---------- generate_report: timestamp formatting ----------

#[test]
fn test_report_timestamp_formatting() {
    let (_tmp, conn) = temp_conn();
    // Insert with a known epoch timestamp: 1700000000 = 2023-11-14T22:13:20Z
    insert_event(
        &conn,
        "n030",
        1,
        "p1",
        "html_comment",
        "sess-ts",
        "5.5.5.5",
        "Agent/1.0",
        "Unknown",
        Some(1_700_000_000),
    );

    let md = report::generate_report(&conn).expect("generate_report must succeed");

    // Must contain a human-readable date, not a raw epoch string like "1700000000"
    assert!(
        !md.contains("| 1700000000"),
        "report must not contain raw epoch seconds"
    );
    // Must contain 2023 (the year for epoch 1700000000)
    assert!(
        md.contains("2023"),
        "report must contain human-readable year 2023"
    );
}
