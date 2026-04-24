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

/// Phase 14: insert a Tier-4 event with a decoded capability string.
#[allow(clippy::too_many_arguments)]
fn insert_event_t4(
    conn: &rusqlite::Connection,
    nonce: &str,
    payload_id: &str,
    embedding_loc: &str,
    session_id: &str,
    remote_addr: &str,
    user_agent: &str,
    classification: &str,
    capability: &str,
    first_seen_epoch: Option<u64>,
) {
    let ts = first_seen_epoch.unwrap_or(1_700_000_000);
    conn.execute(
        "INSERT OR IGNORE INTO nonce_map (nonce, tier, payload_id, embedding_loc, generated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![nonce, 4_u8, payload_id, embedding_loc, ts.to_string()],
    ).unwrap();
    let extra = format!(
        r#"{{"classification":"{}","headers":{{}}}}"#,
        classification
    );
    conn.execute(
        "INSERT INTO events (nonce, tier, payload_id, embedding_loc, first_seen_at, last_seen_at, fire_count, is_replay, session_id, remote_addr, user_agent, extra_headers, t4_capability) \
         VALUES (?1, 4, ?2, ?3, ?4, ?4, 1, 0, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![nonce, payload_id, embedding_loc, ts.to_string(), session_id, remote_addr, user_agent, extra, capability],
    ).unwrap();
}

/// Phase 14: insert a Tier-5 event with a submitted proof + server-computed validity.
#[allow(clippy::too_many_arguments)]
fn insert_event_t5(
    conn: &rusqlite::Connection,
    nonce: &str,
    payload_id: &str,
    embedding_loc: &str,
    session_id: &str,
    remote_addr: &str,
    user_agent: &str,
    classification: &str,
    proof: &str,
    proof_valid: bool,
    first_seen_epoch: Option<u64>,
) {
    let ts = first_seen_epoch.unwrap_or(1_700_000_000);
    conn.execute(
        "INSERT OR IGNORE INTO nonce_map (nonce, tier, payload_id, embedding_loc, generated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![nonce, 5_u8, payload_id, embedding_loc, ts.to_string()],
    ).unwrap();
    let extra = format!(
        r#"{{"classification":"{}","headers":{{}}}}"#,
        classification
    );
    conn.execute(
        "INSERT INTO events (nonce, tier, payload_id, embedding_loc, first_seen_at, last_seen_at, fire_count, is_replay, session_id, remote_addr, user_agent, extra_headers, t5_proof, t5_proof_valid) \
         VALUES (?1, 5, ?2, ?3, ?4, ?4, 1, 0, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![nonce, payload_id, embedding_loc, ts.to_string(), session_id, remote_addr, user_agent, extra, proof, proof_valid as i32],
    ).unwrap();
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

// ---------- Phase 14 Executive Summary extension tests ----------

#[test]
fn test_report_summary_tier4_tier5_counts() {
    let (_tmp, conn) = temp_conn();
    insert_event_t4(
        &conn,
        "aaaaaaaaaaaaaaaa",
        "t4-tools-01",
        "html_comment",
        "sess-t4-1",
        "1.2.3.4",
        "ua-a",
        "Unknown",
        "web_search,browse_page,code_execution",
        None,
    );
    insert_event_t5(
        &conn,
        "bbbbbbbbbbbbbbbb",
        "t5-chain-01",
        "json_ld",
        "sess-t5-1",
        "1.2.3.5",
        "ua-b",
        "Unknown",
        "123",
        true,
        None,
    );
    let md = report::generate_report(&conn).expect("report must render");
    assert!(
        md.contains("| Tier 4 (Capability Introspection) | 1 |"),
        "exec summary missing Tier 4 row:\n{}",
        md
    );
    assert!(
        md.contains("| Tier 5 (Multi-step Compliance) | 1 |"),
        "exec summary missing Tier 5 row:\n{}",
        md
    );
}

// ---------- Phase 14 backward-compat (v4.0-style legacy DB) tests ----------

#[test]
fn test_report_backward_compat_v40_db() {
    // Legacy: only T1 events (no T4/T5 rows written).
    // D-14-12: Tier 4 / Tier 5 rows still present with count=0.
    // D-14-13: Evidence cells for T1 rows show em-dash.
    let (_tmp, conn) = temp_conn();
    insert_event(
        &conn,
        "1111111111111111",
        1,
        "t1-payload-01",
        "html_comment",
        "sess-legacy-1",
        "1.2.3.4",
        "ua-legacy",
        "Unknown",
        None,
    );
    let md = report::generate_report(&conn).expect("report must render");

    // Exec summary always-show (D-14-12)
    assert!(
        md.contains("| Tier 4 (Capability Introspection) | 0 |"),
        "legacy DB exec summary missing Tier 4 | 0:\n{}",
        md
    );
    assert!(
        md.contains("| Tier 5 (Multi-step Compliance) | 0 |"),
        "legacy DB exec summary missing Tier 5 | 0:\n{}",
        md
    );

    // Evidence column on T1 row shows em-dash (D-14-13).
    // The row format is "| sess | 1 | Arbitrary Callback | ... | Unknown | — | t1-payload-01 |"
    // — we assert the em-dash appears between Classification and Payload.
    assert!(
        md.contains("| Unknown | — | t1-payload-01 |"),
        "T1 row should have em-dash in Evidence column:\n{}",
        md
    );
}

// ---------- Phase 14 Evidence column per-tier tests ----------

#[test]
fn test_report_evidence_column_t4() {
    let (_tmp, conn) = temp_conn();
    insert_event_t4(
        &conn,
        "cccccccccccccccc",
        "t4-tools-01",
        "html_comment",
        "sess-t4-x",
        "1.2.3.4",
        "ua-x",
        "Unknown",
        "web_search,browse_page",
        None,
    );
    let md = report::generate_report(&conn).expect("report must render");
    assert!(
        md.contains("web_search,browse_page"),
        "T4 row should surface the capability string in Evidence column:\n{}",
        md
    );
    // The cell appears BETWEEN Classification (Unknown) and Payload (t4-tools-01).
    assert!(
        md.contains("| Unknown | web_search,browse_page | t4-tools-01 |"),
        "T4 row column ordering mismatch:\n{}",
        md
    );
}

#[test]
fn test_report_evidence_column_t5_valid() {
    let (_tmp, conn) = temp_conn();
    insert_event_t5(
        &conn,
        "dddddddddddddddd",
        "t5-chain-01",
        "json_ld",
        "sess-t5-v",
        "1.2.3.4",
        "ua-v",
        "Unknown",
        "123",
        true,
        None,
    );
    let md = report::generate_report(&conn).expect("report must render");
    assert!(
        md.contains("| Unknown | 123 ✓ VALID | t5-chain-01 |"),
        "T5 valid row should render '123 ✓ VALID' in Evidence column:\n{}",
        md
    );
}

#[test]
fn test_report_evidence_column_t5_invalid() {
    let (_tmp, conn) = temp_conn();
    insert_event_t5(
        &conn,
        "eeeeeeeeeeeeeeee",
        "t5-chain-02",
        "json_ld",
        "sess-t5-i",
        "1.2.3.4",
        "ua-i",
        "Unknown",
        "456",
        false,
        None,
    );
    let md = report::generate_report(&conn).expect("report must render");
    assert!(
        md.contains("| Unknown | 456 ✗ INVALID | t5-chain-02 |"),
        "T5 invalid row should render '456 ✗ INVALID' in Evidence column:\n{}",
        md
    );
}

// ---------- Phase 14 full 5-tier end-to-end test ----------

#[test]
fn test_report_full_5tier_markdown() {
    let (_tmp, conn) = temp_conn();
    insert_event(
        &conn,
        "1111111111111111",
        1,
        "t1-p",
        "html_comment",
        "sess-1",
        "1.2.3.4",
        "ua",
        "Unknown",
        None,
    );
    insert_event(
        &conn,
        "2222222222222222",
        2,
        "t2-p",
        "meta_tag",
        "sess-2",
        "1.2.3.5",
        "ua",
        "Unknown",
        None,
    );
    insert_event(
        &conn,
        "3333333333333333",
        3,
        "t3-p",
        "invisible_element",
        "sess-3",
        "1.2.3.6",
        "ua",
        "Unknown",
        None,
    );
    insert_event_t4(
        &conn,
        "4444444444444444",
        "t4-p",
        "json_ld",
        "sess-4",
        "1.2.3.7",
        "ua",
        "Unknown",
        "web_search,code_execution",
        None,
    );
    insert_event_t5(
        &conn,
        "5555555555555555",
        "t5-p",
        "semantic_prose",
        "sess-5",
        "1.2.3.8",
        "ua",
        "Unknown",
        "789",
        true,
        None,
    );
    let md = report::generate_report(&conn).expect("report must render");

    // Executive summary has all 5 tier rows with count=1.
    assert!(
        md.contains("| Tier 1 (Arbitrary Callback) | 1 |"),
        "T1 exec row:\n{}",
        md
    );
    assert!(
        md.contains("| Tier 2 (Conditional Branch) | 1 |"),
        "T2 exec row:\n{}",
        md
    );
    assert!(
        md.contains("| Tier 3 (Computed Callback) | 1 |"),
        "T3 exec row:\n{}",
        md
    );
    assert!(
        md.contains("| Tier 4 (Capability Introspection) | 1 |"),
        "T4 exec row:\n{}",
        md
    );
    assert!(
        md.contains("| Tier 5 (Multi-step Compliance) | 1 |"),
        "T5 exec row:\n{}",
        md
    );

    // Evidence column renders per tier.
    assert!(md.contains("| Unknown | — | t1-p |"), "T1 em-dash:\n{}", md);
    assert!(md.contains("| Unknown | — | t2-p |"), "T2 em-dash:\n{}", md);
    assert!(md.contains("| Unknown | — | t3-p |"), "T3 em-dash:\n{}", md);
    assert!(
        md.contains("| Unknown | web_search,code_execution | t4-p |"),
        "T4 cell:\n{}",
        md
    );
    assert!(
        md.contains("| Unknown | 789 ✓ VALID | t5-p |"),
        "T5 cell:\n{}",
        md
    );

    // Evidence Table header has the new Evidence column.
    assert!(
        md.contains("| Session | Tier | Proof Level | First Seen | Source IP | User Agent | Fire Count | Classification | Evidence | Payload |"),
        "Evidence Table header missing Evidence column:\n{}",
        md
    );
}
