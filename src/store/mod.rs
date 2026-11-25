use rusqlite::{params, Connection};

/// Open or create a SQLite database at the given path, running schema migrations.
///
/// Ensures the parent directory exists before SQLite tries to open the file —
/// without this, a fresh project (honeyprompt.toml present but `.honeyprompt/`
/// missing) fails with SQLite error 14 "unable to open database file".
pub fn open_or_create_db(path: &std::path::Path) -> anyhow::Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            anyhow::anyhow!("Failed to create DB directory {}: {}", parent.display(), e)
        })?;
    }
    let conn = Connection::open(path)?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Execute all schema migrations against an open connection.
///
/// The v4.0 baseline schema (`CREATE TABLE IF NOT EXISTS events (...)`) is preserved
/// byte-identically (D-13-17 — fresh DBs receive the v4.0 shape first; additive
/// migrations then lift the database to v5.0). Idempotency comes from the
/// `PRAGMA user_version` gate — SQLite has no `ALTER TABLE ADD COLUMN IF NOT EXISTS`,
/// so re-running `run_migrations` on a v5.0 DB skips the ALTER block entirely.
pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    // v4.0 baseline — UNCHANGED (D-13-17). Fresh DBs get v4.0 shape first, then
    // migrations add T4/T5 columns additively.
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;

        CREATE TABLE IF NOT EXISTS events (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            nonce           TEXT NOT NULL,
            tier            INTEGER NOT NULL,
            payload_id      TEXT NOT NULL,
            embedding_loc   TEXT NOT NULL,
            first_seen_at   TEXT NOT NULL,
            last_seen_at    TEXT NOT NULL,
            fire_count      INTEGER NOT NULL DEFAULT 1,
            is_replay       INTEGER NOT NULL DEFAULT 0,
            session_id      TEXT,
            remote_addr     TEXT,
            user_agent      TEXT,
            extra_headers   TEXT
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_events_nonce ON events(nonce);

        CREATE TABLE IF NOT EXISTS nonce_map (
            nonce           TEXT PRIMARY KEY,
            tier            INTEGER NOT NULL,
            payload_id      TEXT NOT NULL,
            embedding_loc   TEXT NOT NULL,
            generated_at    TEXT NOT NULL
        );
        ",
    )?;

    // Phase 13 — gated additive migration (D-13-17). SQLite has no
    // `ALTER TABLE ADD COLUMN IF NOT EXISTS`, so idempotency comes from the
    // PRAGMA user_version gate. `PRAGMA user_version = <N>` cannot use
    // parameter binding — the `1` must be a literal in the SQL text.
    let version: u32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version < 1 {
        conn.execute_batch(
            "
            ALTER TABLE events ADD COLUMN t4_capability TEXT;
            ALTER TABLE events ADD COLUMN t5_proof TEXT;
            ALTER TABLE events ADD COLUMN t5_proof_valid INTEGER;
            PRAGMA user_version = 1;
            ",
        )?;
    }
    // v2: one row per callback. `events` keeps one summary row per nonce; `hits`
    // keeps every individual hit so later fetches (a different network, days
    // after seeding) are preserved as evidence. Rows are never updated.
    if version < 2 {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS hits (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                nonce           TEXT NOT NULL,
                tier            INTEGER NOT NULL,
                received_at     INTEGER NOT NULL,
                remote_addr     TEXT,
                session_id      TEXT,
                user_agent      TEXT,
                extra_headers   TEXT,
                t4_capability   TEXT,
                t5_proof        TEXT,
                t5_proof_valid  INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_hits_nonce ON hits(nonce);
            PRAGMA user_version = 2;
            ",
        )?;
    }
    // v3: page fetches, so each hit can be labelled in-session / delayed /
    // out-of-band by comparing it with when that IP last fetched the page.
    // Hits stored before v3 keep `timing = NULL` (shown as "unknown").
    if version < 3 {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS page_views (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                received_at     INTEGER NOT NULL,
                remote_addr     TEXT NOT NULL,
                user_agent      TEXT,
                path            TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_page_views_addr ON page_views(remote_addr, received_at);
            CREATE INDEX IF NOT EXISTS idx_page_views_time ON page_views(received_at);
            ALTER TABLE hits ADD COLUMN timing TEXT;
            ALTER TABLE hits ADD COLUMN last_page_view_at INTEGER;
            PRAGMA user_version = 3;
            ",
        )?;
    }
    // v4: tier-6 (context exfiltration) outcome. Only match / no-match and the
    // submitted value's length are stored, never the value.
    if version < 4 {
        conn.execute_batch(
            "
            ALTER TABLE events ADD COLUMN exfil_match INTEGER;
            ALTER TABLE hits ADD COLUMN exfil_match INTEGER;
            ALTER TABLE hits ADD COLUMN exfil_value_len INTEGER;
            PRAGMA user_version = 4;
            ",
        )?;
    }
    // v5: seeded canaries (`honeyprompt seed`). Kept apart from `events`/`hits`
    // because seed hits are not tier detections.
    if version < 5 {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS seeds (
                nonce           TEXT PRIMARY KEY,
                label           TEXT NOT NULL UNIQUE,
                note            TEXT,
                created_at      INTEGER NOT NULL,
                hit_count       INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE IF NOT EXISTS seed_hits (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                nonce           TEXT NOT NULL,
                received_at     INTEGER NOT NULL,
                remote_addr     TEXT,
                user_agent      TEXT,
                classification  TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_seed_hits_nonce ON seed_hits(nonce);
            PRAGMA user_version = 5;
            ",
        )?;
    }
    Ok(())
}

/// A hit within this many seconds of the same IP fetching the page is "in-session".
pub const IN_SESSION_WINDOW_SECS: i64 = 30 * 60;

/// Page fetches are kept this long; it is also how far back a hit looks for one.
pub const PAGE_VIEW_RETENTION_SECS: i64 = 90 * 24 * 60 * 60;

/// How a hit relates to page fetches from the same client IP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTiming {
    /// That IP fetched the page within `IN_SESSION_WINDOW_SECS` before the hit.
    InSession,
    /// That IP fetched the page earlier, but longer ago than the window.
    Delayed,
    /// No fetch of the page from that IP in the retention period before the hit.
    OutOfBand,
    /// Hit stored before page fetches were recorded.
    Unknown,
}

impl HitTiming {
    pub fn as_str(self) -> &'static str {
        match self {
            HitTiming::InSession => "in-session",
            HitTiming::Delayed => "delayed",
            HitTiming::OutOfBand => "out-of-band",
            HitTiming::Unknown => "unknown",
        }
    }

    pub fn parse(s: Option<&str>) -> Self {
        match s {
            Some("in-session") => HitTiming::InSession,
            Some("delayed") => HitTiming::Delayed,
            Some("out-of-band") => HitTiming::OutOfBand,
            _ => HitTiming::Unknown,
        }
    }

    /// Classify a hit at `hit_at` given the IP's latest page fetch at or before it.
    pub fn classify(hit_at: i64, last_page_view_at: Option<i64>) -> Self {
        match last_page_view_at {
            Some(pv) if hit_at - pv <= IN_SESSION_WINDOW_SECS => HitTiming::InSession,
            Some(_) => HitTiming::Delayed,
            None => HitTiming::OutOfBand,
        }
    }
}

/// Record one fetch of a honeypot page and prune fetches past the retention period.
pub fn record_page_view(
    conn: &Connection,
    received_at: i64,
    remote_addr: &str,
    user_agent: &str,
    path: &str,
) -> rusqlite::Result<()> {
    record_page_view_capped(
        conn,
        received_at,
        remote_addr,
        user_agent,
        path,
        MAX_PAGE_VIEWS,
    )
}

/// Repeat fetches from one IP within this many seconds update its latest row
/// instead of adding one, so hammering the page can't grow the table.
pub const PAGE_VIEW_COOLDOWN_SECS: i64 = 60;

/// Hard cap on stored page fetches (oldest pruned). A flood from many IPs can
/// still push out real fetches, so later hits from those IPs may be labelled
/// out-of-band; the cap only bounds disk use.
pub const MAX_PAGE_VIEWS: i64 = 100_000;

fn record_page_view_capped(
    conn: &Connection,
    received_at: i64,
    remote_addr: &str,
    user_agent: &str,
    path: &str,
    cap: i64,
) -> rusqlite::Result<()> {
    // Keep "last fetch from this IP" exact (hit timing reads it) while adding
    // at most one row per IP per cooldown window.
    let updated = conn.execute(
        "UPDATE page_views SET received_at = ?1, user_agent = ?2, path = ?3
         WHERE id = (
             SELECT id FROM page_views
             WHERE remote_addr = ?4 AND received_at >= ?5 AND received_at <= ?1
             ORDER BY received_at DESC LIMIT 1
         )",
        params![
            received_at,
            user_agent,
            path,
            remote_addr,
            received_at - PAGE_VIEW_COOLDOWN_SECS
        ],
    )?;
    if updated == 0 {
        conn.execute(
            "INSERT INTO page_views (received_at, remote_addr, user_agent, path)
             VALUES (?1, ?2, ?3, ?4)",
            params![received_at, remote_addr, user_agent, path],
        )?;
        // AUTOINCREMENT ids only grow, so this keeps at most `cap` newest rows.
        conn.execute(
            "DELETE FROM page_views WHERE id <= (SELECT MAX(id) FROM page_views) - ?1",
            params![cap],
        )?;
    }
    conn.execute(
        "DELETE FROM page_views WHERE received_at < ?1",
        params![received_at - PAGE_VIEW_RETENTION_SECS],
    )?;
    Ok(())
}

/// Most hits stored per nonce. `events.fire_count` keeps counting past this, so
/// the true total is still visible; the cap only bounds disk use when one URL
/// is hammered.
pub const MAX_HITS_PER_NONCE: u32 = 1000;

/// Record a callback: upsert the per-nonce summary row in `events` (replay
/// detection) and append a row to `hits` (up to `MAX_HITS_PER_NONCE` per nonce),
/// in one transaction.
///
/// SRV-07: No body parameter — metadata-only mode enforced by API design.
///
/// On first fire: inserts a new row with fire_count=1, is_replay=0.
/// On subsequent fires (same nonce): increments fire_count and sets is_replay=1.
///
/// Returns `(fire_count, is_replay)` so the broker knows if this was a replay.
/// All values are passed via parameterized query — SQL metacharacters cannot corrupt queries.
///
/// Phase 13 extension: three trailing `Option` parameters carry T4/T5 payload data.
/// Per D-13-19 / RESEARCH Risk 6, these fields follow first-write-wins semantics —
/// they are stored on INSERT but the `ON CONFLICT(nonce) DO UPDATE SET` clause does
/// NOT overwrite them on replay (replay only touches last_seen_at, fire_count, is_replay).
#[allow(clippy::too_many_arguments)]
pub fn insert_callback_event_detailed(
    conn: &Connection,
    nonce: &str,
    tier: u8,
    payload_id: &str,
    embedding_loc: &str,
    session_id: &str,
    remote_addr: &str,
    user_agent: &str,
    extra_headers: &str,
    t4_capability: Option<&str>,
    t5_proof: Option<&str>,
    t5_proof_valid: Option<bool>,
    exfil: Option<crate::types::ExfilCheck>,
) -> rusqlite::Result<InsertedHit> {
    let now = chrono_now();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO events (
             nonce, tier, payload_id, embedding_loc,
             first_seen_at, last_seen_at, fire_count, is_replay,
             session_id, remote_addr, user_agent, extra_headers,
             t4_capability, t5_proof, t5_proof_valid, exfil_match
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, 1, 0, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(nonce) DO UPDATE SET
           last_seen_at = ?5,
           fire_count = fire_count + 1,
           is_replay = 1",
        params![
            nonce,
            tier,
            payload_id,
            embedding_loc,
            now,
            session_id,
            remote_addr,
            user_agent,
            extra_headers,
            t4_capability,
            t5_proof,
            t5_proof_valid,
            exfil.map(|e| e.matched),
        ],
    )?;

    let hit_at: i64 = now.parse().unwrap_or(0);
    let last_page_view_at: Option<i64> = tx.query_row(
        "SELECT MAX(received_at) FROM page_views
         WHERE remote_addr = ?1 AND received_at <= ?2 AND received_at >= ?3",
        params![remote_addr, hit_at, hit_at - PAGE_VIEW_RETENTION_SECS],
        |row| row.get(0),
    )?;
    let timing = HitTiming::classify(hit_at, last_page_view_at);

    tx.execute(
        "INSERT INTO hits (
             nonce, tier, received_at, remote_addr, session_id, user_agent,
             extra_headers, t4_capability, t5_proof, t5_proof_valid,
             timing, last_page_view_at, exfil_match, exfil_value_len
         )
         SELECT ?1, ?2, CAST(?3 AS INTEGER), ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?12, ?13, ?14, ?15
         WHERE (SELECT COUNT(*) FROM hits WHERE nonce = ?1) < ?11",
        params![
            nonce,
            tier,
            now,
            remote_addr,
            session_id,
            user_agent,
            extra_headers,
            t4_capability,
            t5_proof,
            t5_proof_valid,
            MAX_HITS_PER_NONCE,
            timing.as_str(),
            last_page_view_at,
            exfil.map(|e| e.matched),
            exfil.map(|e| e.value_len),
        ],
    )?;

    let (fire_count, is_replay_int): (u32, i64) = tx.query_row(
        "SELECT fire_count, is_replay FROM events WHERE nonce = ?1",
        params![nonce],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let seeded_at: Option<i64> = tx
        .query_row(
            "SELECT CAST(generated_at AS INTEGER) FROM nonce_map WHERE nonce = ?1",
            params![nonce],
            |row| row.get(0),
        )
        .ok();
    tx.commit()?;

    Ok(InsertedHit {
        fire_count,
        is_replay: is_replay_int != 0,
        received_at: hit_at,
        timing,
        last_page_view_at,
        seeded_at,
    })
}

/// What `insert_callback_event_detailed` recorded, for alerting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertedHit {
    pub fire_count: u32,
    pub is_replay: bool,
    pub received_at: i64,
    pub timing: HitTiming,
    pub last_page_view_at: Option<i64>,
    /// When the nonce's page was generated (None if the nonce is not in nonce_map).
    pub seeded_at: Option<i64>,
}

/// Record a callback; see `insert_callback_event_detailed`. Returns
/// `(fire_count, is_replay)`.
#[allow(clippy::too_many_arguments)]
pub fn insert_callback_event(
    conn: &Connection,
    nonce: &str,
    tier: u8,
    payload_id: &str,
    embedding_loc: &str,
    session_id: &str,
    remote_addr: &str,
    user_agent: &str,
    extra_headers: &str,
    t4_capability: Option<&str>,
    t5_proof: Option<&str>,
    t5_proof_valid: Option<bool>,
) -> rusqlite::Result<(u32, bool)> {
    let hit = insert_callback_event_detailed(
        conn,
        nonce,
        tier,
        payload_id,
        embedding_loc,
        session_id,
        remote_addr,
        user_agent,
        extra_headers,
        t4_capability,
        t5_proof,
        t5_proof_valid,
        None,
    )?;
    Ok((hit.fire_count, hit.is_replay))
}

/// Look up nonce metadata from the nonce_map table.
///
/// Returns `Some((tier, payload_id, embedding_loc))` if the nonce exists, `None` otherwise.
pub fn lookup_nonce(
    conn: &Connection,
    nonce: &str,
) -> rusqlite::Result<Option<(u8, String, String)>> {
    let result = conn.query_row(
        "SELECT tier, payload_id, embedding_loc FROM nonce_map WHERE nonce = ?1",
        params![nonce],
        |row| {
            Ok((
                row.get::<_, u8>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        },
    );
    match result {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Count unique detection sessions per tier (1..=5), excluding known crawlers.
///
/// Returns a [u32; 5] array where index 0 = tier 1 count, ..., index 4 = tier 5 count.
/// D-07: per-tier hit counts for the test-agent scorecard.
/// D-15-06: extended from 3 to 5 tiers; KnownCrawler exclusion filter unchanged.
/// Number of recorded page fetches.
pub fn page_view_count(conn: &Connection) -> rusqlite::Result<u64> {
    conn.query_row("SELECT COUNT(*) FROM page_views", [], |r| r.get(0))
}

/// Distinct non-empty User-Agent strings from page fetches and callbacks
/// (at most 20, sorted). Used by `test-agent` to fingerprint the agent.
pub fn distinct_user_agents(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT ua FROM (
             SELECT user_agent AS ua FROM page_views
             UNION
             SELECT user_agent AS ua FROM events
         ) WHERE ua IS NOT NULL AND ua != '' ORDER BY ua LIMIT 20",
    )?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    rows.collect()
}

pub fn detections_by_tier(conn: &Connection) -> rusqlite::Result<[u32; 5]> {
    let mut counts = [0u32; 5];
    for tier in 1u8..=5 {
        counts[(tier - 1) as usize] = conn.query_row(
            "SELECT COUNT(DISTINCT session_id) FROM events
             WHERE tier = ?1
             AND extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
            params![tier],
            |row| row.get(0),
        )?;
    }
    Ok(counts)
}

/// Count unique (session_id, tier) detection pairs, excluding known-crawler events.
///
/// Per D-08: detection counting is per-session per-tier. Same session + same tier = 1 detection.
/// Known crawlers (classified as KnownCrawler) are excluded from the detection count.
pub fn count_detections(conn: &Connection) -> rusqlite::Result<u32> {
    let count: u32 = conn.query_row(
        "SELECT COUNT(DISTINCT session_id || '-' || tier) FROM events
         WHERE extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Summary statistics for the report executive summary.
#[derive(serde::Serialize)]
pub struct ReportSummary {
    pub total_sessions: u32,
    pub detection_sessions: u32, // excludes KnownCrawler
    pub crawler_sessions: u32,   // KnownCrawler only
    pub tier1_sessions: u32,
    pub tier2_sessions: u32,
    pub tier3_sessions: u32,
    pub tier4_sessions: u32,            // NEW Phase 14 (D-14-12 always-show)
    pub tier5_sessions: u32,            // NEW Phase 14 (D-14-12 always-show)
    pub earliest_event: Option<String>, // epoch seconds string
    pub latest_event: Option<String>,   // epoch seconds string
}

/// Per-(session_id, tier) row for the evidence table.
pub struct ReportSession {
    pub session_id: String,
    pub tier: u8,
    pub payload_id: String,
    pub embedding_loc: String,
    pub first_seen_at: String, // epoch seconds string
    pub last_seen_at: String,  // epoch seconds string
    pub fire_count: u32,
    pub remote_addr: String,
    pub user_agent: String,
    pub classification: String, // parsed from extra_headers JSON
    /// Phase 14 (D-14-08, D-14-10): decoded T4 capability list; None for non-T4 sessions or legacy rows.
    pub t4_capability: Option<String>,
    /// Phase 14 (D-14-08, D-14-11): submitted T5 proof; None for non-T5 sessions or legacy rows.
    pub t5_proof: Option<String>,
    /// Phase 14 (D-14-08, D-14-11): server-verified T5 proof validity; None for non-T5 sessions or legacy rows.
    pub t5_proof_valid: Option<bool>,
}

/// Query aggregate summary statistics from the events table for the report.
///
/// Counts are session-based (per (session_id, tier) pair) per D-04.
pub fn query_report_summary(conn: &Connection) -> rusqlite::Result<ReportSummary> {
    let total_sessions: u32 = conn.query_row(
        "SELECT COUNT(DISTINCT session_id || '-' || tier) FROM events",
        [],
        |row| row.get(0),
    )?;

    let detection_sessions: u32 = conn.query_row(
        "SELECT COUNT(DISTINCT session_id || '-' || tier) FROM events
         WHERE extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
        [],
        |row| row.get(0),
    )?;

    let crawler_sessions: u32 = conn.query_row(
        "SELECT COUNT(DISTINCT session_id) FROM events
         WHERE extra_headers LIKE '%\"classification\":\"KnownCrawler%'",
        [],
        |row| row.get(0),
    )?;

    let tier1_sessions: u32 = conn.query_row(
        "SELECT COUNT(DISTINCT session_id) FROM events
         WHERE tier = 1 AND extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
        [],
        |row| row.get(0),
    )?;

    let tier2_sessions: u32 = conn.query_row(
        "SELECT COUNT(DISTINCT session_id) FROM events
         WHERE tier = 2 AND extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
        [],
        |row| row.get(0),
    )?;

    let tier3_sessions: u32 = conn.query_row(
        "SELECT COUNT(DISTINCT session_id) FROM events
         WHERE tier = 3 AND extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
        [],
        |row| row.get(0),
    )?;

    let tier4_sessions: u32 = conn.query_row(
        "SELECT COUNT(DISTINCT session_id) FROM events
         WHERE tier = 4 AND extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
        [],
        |row| row.get(0),
    )?;

    let tier5_sessions: u32 = conn.query_row(
        "SELECT COUNT(DISTINCT session_id) FROM events
         WHERE tier = 5 AND extra_headers NOT LIKE '%\"classification\":\"KnownCrawler%'",
        [],
        |row| row.get(0),
    )?;

    let (earliest_event, latest_event): (Option<String>, Option<String>) = conn.query_row(
        "SELECT MIN(first_seen_at), MAX(last_seen_at) FROM events",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    Ok(ReportSummary {
        total_sessions,
        detection_sessions,
        crawler_sessions,
        tier1_sessions,
        tier2_sessions,
        tier3_sessions,
        tier4_sessions, // NEW Phase 14
        tier5_sessions, // NEW Phase 14
        earliest_event,
        latest_event,
    })
}

/// Query all (session_id, tier) pairs from the events table, ordered by first seen descending.
///
/// Groups raw events rows by (session_id, tier) and aggregates metadata.
/// Classification is parsed from the extra_headers JSON blob.
pub fn query_report_sessions(conn: &Connection) -> rusqlite::Result<Vec<ReportSession>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, tier, payload_id, embedding_loc,
                MIN(first_seen_at) as first_seen_at,
                MAX(last_seen_at) as last_seen_at,
                SUM(fire_count) as total_fires,
                MAX(remote_addr) as remote_addr,
                MAX(user_agent) as user_agent,
                MAX(extra_headers) as extra_headers,
                MAX(t4_capability) as t4_capability,
                MAX(t5_proof) as t5_proof,
                MAX(t5_proof_valid) as t5_proof_valid
         FROM events
         GROUP BY session_id, tier
         ORDER BY MIN(first_seen_at) DESC",
    )?;

    let sessions = stmt
        .query_map([], |row| {
            let extra_headers: Option<String> = row.get(9)?;
            let classification = parse_classification(extra_headers.as_deref());
            Ok(ReportSession {
                session_id: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                tier: row.get::<_, u8>(1)?,
                payload_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                embedding_loc: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                first_seen_at: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                last_seen_at: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                fire_count: row.get::<_, u32>(6)?,
                remote_addr: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                user_agent: row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                classification,
                // Phase 14: NULL-safe Option<T> mapping. Per D-14-14 we always SELECT the columns;
                // legacy rows and non-T4/T5 sessions return NULL -> None.
                t4_capability: row.get::<_, Option<String>>(10)?,
                t5_proof: row.get::<_, Option<String>>(11)?,
                t5_proof_valid: row.get::<_, Option<bool>>(12)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(sessions)
}

/// One stored hit on a nonce that fired more than once.
///
/// Nonces are baked into the generated page, so on a public site every visitor
/// shares them: repeat hits are usually different visitors. `timing` (relative
/// to that IP's own page fetches) is what separates in-session from later use.
pub struct RepeatHit {
    pub nonce: String,
    pub tier: u8,
    /// 1-based position of this hit among the nonce's stored hits.
    pub hit_number: u32,
    pub received_at: i64,
    /// When the page carrying this nonce was generated (None if unknown).
    pub seeded_at: Option<i64>,
    pub remote_addr: String,
    pub user_agent: String,
    pub classification: String,
    pub timing: HitTiming,
    /// Hits stored for this nonce (capped at MAX_HITS_PER_NONCE).
    pub stored_hits: u32,
    /// Total fires recorded in `events` (not capped).
    pub total_fires: u32,
}

/// Every stored hit for nonces with more than one hit, ordered by nonce then time.
pub fn query_repeat_hits(conn: &Connection) -> rusqlite::Result<Vec<RepeatHit>> {
    let mut stmt = conn.prepare(
        "SELECT h.nonce, h.tier, h.received_at, h.remote_addr, h.user_agent, h.extra_headers,
                CAST(n.generated_at AS INTEGER), h.timing,
                g.stored, COALESCE(e.fire_count, g.stored),
                (SELECT COUNT(*) FROM hits x WHERE x.nonce = h.nonce AND x.id <= h.id)
         FROM hits h
         JOIN (SELECT nonce, MIN(id) AS first_id, COUNT(*) AS stored
               FROM hits GROUP BY nonce HAVING COUNT(*) > 1) g ON g.nonce = h.nonce
         JOIN hits f ON f.id = g.first_id
         LEFT JOIN nonce_map n ON n.nonce = h.nonce
         LEFT JOIN events e ON e.nonce = h.nonce
         ORDER BY f.received_at DESC, h.nonce, h.id",
    )?;
    let rows = stmt
        .query_map([], |row| {
            let extra_headers: Option<String> = row.get(5)?;
            let timing: Option<String> = row.get(7)?;
            Ok(RepeatHit {
                nonce: row.get(0)?,
                tier: row.get(1)?,
                received_at: row.get(2)?,
                remote_addr: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                user_agent: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                classification: parse_classification(extra_headers.as_deref()),
                seeded_at: row.get(6)?,
                timing: HitTiming::parse(timing.as_deref()),
                stored_hits: row.get(8)?,
                total_fires: row.get(9)?,
                hit_number: row.get(10)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// Tier-6 outcome for a test-agent run.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExfilSummary {
    /// The run's canary token came back.
    pub leaked: bool,
    /// Tier-6 callbacks whose value was not the canary (values discarded).
    pub non_matching: u32,
}

/// Summarise tier-6 hits (stored hits, so capped per nonce like all hits).
pub fn exfil_summary(conn: &Connection) -> rusqlite::Result<ExfilSummary> {
    conn.query_row(
        "SELECT COALESCE(SUM(exfil_match = 1), 0), COALESCE(SUM(exfil_match = 0), 0)
         FROM hits WHERE tier = 6",
        [],
        |r| {
            Ok(ExfilSummary {
                leaked: r.get::<_, u32>(0)? > 0,
                non_matching: r.get(1)?,
            })
        },
    )
}

/// Most hits stored per seed; `seeds.hit_count` keeps the true total.
pub const MAX_HITS_PER_SEED: u32 = 1000;

/// Create a seed. Fails if the label (or nonce) already exists.
pub fn insert_seed(
    conn: &Connection,
    nonce: &str,
    label: &str,
    note: Option<&str>,
    created_at: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO seeds (nonce, label, note, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![nonce, label, note, created_at],
    )?;
    Ok(())
}

/// The seed a recorded hit belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedRef {
    pub label: String,
    pub note: Option<String>,
    pub created_at: i64,
}

/// Record a fetch of `/r/{nonce}`. Returns `None` (and stores nothing) when the
/// nonce is not a seed. Every hit bumps `hit_count`; rows stop at the cap.
pub fn record_seed_hit(
    conn: &Connection,
    nonce: &str,
    received_at: i64,
    remote_addr: &str,
    user_agent: &str,
    classification: &str,
) -> rusqlite::Result<Option<SeedRef>> {
    let tx = conn.unchecked_transaction()?;
    let seed = tx.query_row(
        "SELECT label, note, created_at FROM seeds WHERE nonce = ?1",
        params![nonce],
        |r| {
            Ok(SeedRef {
                label: r.get(0)?,
                note: r.get(1)?,
                created_at: r.get(2)?,
            })
        },
    );
    let seed = match seed {
        Ok(s) => s,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(e),
    };
    tx.execute(
        "UPDATE seeds SET hit_count = hit_count + 1 WHERE nonce = ?1",
        params![nonce],
    )?;
    tx.execute(
        "INSERT INTO seed_hits (nonce, received_at, remote_addr, user_agent, classification)
         SELECT ?1, ?2, ?3, ?4, ?5
         WHERE (SELECT COUNT(*) FROM seed_hits WHERE nonce = ?1) < ?6",
        params![
            nonce,
            received_at,
            remote_addr,
            user_agent,
            classification,
            MAX_HITS_PER_SEED
        ],
    )?;
    tx.commit()?;
    Ok(Some(seed))
}

/// One seed with its hit statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedSummary {
    pub label: String,
    pub note: Option<String>,
    pub nonce: String,
    pub created_at: i64,
    /// Every hit, including ones past the storage cap.
    pub hit_count: u32,
    pub first_hit_at: Option<i64>,
    pub last_hit_at: Option<i64>,
    /// Distinct client IPs among stored hits.
    pub distinct_ips: u32,
}

/// All seeds, oldest first.
pub fn list_seeds(conn: &Connection) -> rusqlite::Result<Vec<SeedSummary>> {
    let mut stmt = conn.prepare(
        "SELECT s.label, s.note, s.nonce, s.created_at, s.hit_count,
                MIN(h.received_at), MAX(h.received_at), COUNT(DISTINCT h.remote_addr)
         FROM seeds s LEFT JOIN seed_hits h ON h.nonce = s.nonce
         GROUP BY s.nonce
         ORDER BY s.created_at, s.label",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SeedSummary {
                label: r.get(0)?,
                note: r.get(1)?,
                nonce: r.get(2)?,
                created_at: r.get(3)?,
                hit_count: r.get(4)?,
                first_hit_at: r.get(5)?,
                last_hit_at: r.get(6)?,
                distinct_ips: r.get(7)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// One stored seed hit, for the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedHitRow {
    pub label: String,
    pub received_at: i64,
    pub created_at: i64,
    pub remote_addr: String,
    pub user_agent: String,
    pub classification: String,
}

/// The most recent stored seed hits (up to `limit`) and the stored total.
pub fn recent_seed_hits(conn: &Connection, limit: u32) -> rusqlite::Result<(Vec<SeedHitRow>, u32)> {
    let total: u32 = conn.query_row("SELECT COUNT(*) FROM seed_hits", [], |r| r.get(0))?;
    let mut stmt = conn.prepare(
        "SELECT s.label, h.received_at, s.created_at, h.remote_addr, h.user_agent, h.classification
         FROM seed_hits h JOIN seeds s ON s.nonce = h.nonce
         ORDER BY h.received_at DESC, h.id DESC
         LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit], |r| {
            Ok(SeedHitRow {
                label: r.get(0)?,
                received_at: r.get(1)?,
                created_at: r.get(2)?,
                remote_addr: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
                user_agent: r.get::<_, Option<String>>(4)?.unwrap_or_default(),
                classification: r
                    .get::<_, Option<String>>(5)?
                    .unwrap_or_else(|| "Unknown".to_string()),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok((rows, total))
}

/// Stored hits per tier, split by timing label.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TimingCounts {
    pub tier: u8,
    pub in_session: u32,
    pub delayed: u32,
    pub out_of_band: u32,
    pub unknown: u32,
}

/// Count stored hits per tier by timing label. Tiers with no hits are omitted.
pub fn query_hit_timing_counts(conn: &Connection) -> rusqlite::Result<Vec<TimingCounts>> {
    let mut stmt = conn.prepare(
        "SELECT tier,
                SUM(timing = 'in-session'),
                SUM(timing = 'delayed'),
                SUM(timing = 'out-of-band'),
                SUM(timing IS NULL OR timing NOT IN ('in-session', 'delayed', 'out-of-band'))
         FROM hits GROUP BY tier ORDER BY tier",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(TimingCounts {
                tier: row.get(0)?,
                in_session: row.get::<_, Option<u32>>(1)?.unwrap_or(0),
                delayed: row.get::<_, Option<u32>>(2)?.unwrap_or(0),
                out_of_band: row.get::<_, Option<u32>>(3)?.unwrap_or(0),
                unknown: row.get::<_, Option<u32>>(4)?.unwrap_or(0),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// A delayed or out-of-band hit, for the report's Hit Timing table.
pub struct NotableHit {
    pub nonce: String,
    pub tier: u8,
    pub received_at: i64,
    pub timing: HitTiming,
    /// Latest page fetch from the same IP before the hit (None for out-of-band).
    pub last_page_view_at: Option<i64>,
    pub seeded_at: Option<i64>,
    pub remote_addr: String,
    pub user_agent: String,
    pub classification: String,
}

/// The most recent delayed / out-of-band hits (up to `limit`), plus the total count.
pub fn query_notable_hits(
    conn: &Connection,
    limit: u32,
) -> rusqlite::Result<(Vec<NotableHit>, u32)> {
    let total: u32 = conn.query_row(
        "SELECT COUNT(*) FROM hits WHERE timing IN ('delayed', 'out-of-band')",
        [],
        |r| r.get(0),
    )?;
    let mut stmt = conn.prepare(
        "SELECT h.nonce, h.tier, h.received_at, h.timing, h.last_page_view_at,
                CAST(n.generated_at AS INTEGER), h.remote_addr, h.user_agent, h.extra_headers
         FROM hits h
         LEFT JOIN nonce_map n ON n.nonce = h.nonce
         WHERE h.timing IN ('delayed', 'out-of-band')
         ORDER BY h.received_at DESC, h.id DESC
         LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit], |row| {
            let timing: Option<String> = row.get(3)?;
            let extra_headers: Option<String> = row.get(8)?;
            Ok(NotableHit {
                nonce: row.get(0)?,
                tier: row.get(1)?,
                received_at: row.get(2)?,
                timing: HitTiming::parse(timing.as_deref()),
                last_page_view_at: row.get(4)?,
                seeded_at: row.get(5)?,
                remote_addr: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                user_agent: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                classification: parse_classification(extra_headers.as_deref()),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok((rows, total))
}

/// Parse the classification string from the extra_headers JSON blob.
///
/// Expected format: `{"classification": "KnownCrawler:OpenAI", "headers": {...}}`
/// Falls back to "Unknown" if the JSON cannot be parsed or the field is missing.
fn parse_classification(extra_headers: Option<&str>) -> String {
    #[derive(serde::Deserialize)]
    struct ExtraHeaders {
        classification: Option<String>,
    }
    match extra_headers {
        Some(s) => serde_json::from_str::<ExtraHeaders>(s)
            .ok()
            .and_then(|h| h.classification)
            .unwrap_or_else(|| "Unknown".to_string()),
        None => "Unknown".to_string(),
    }
}

/// Insert a nonce-to-payload mapping into the nonce_map table.
///
/// All values are passed via parameterized query — SQL metacharacters in any
/// argument are stored literally and cannot corrupt the query (Pitfall 3).
pub fn insert_nonce(
    conn: &Connection,
    nonce: &str,
    tier: u8,
    payload_id: &str,
    embedding_loc: &str,
) -> rusqlite::Result<()> {
    let generated_at = chrono_now();
    conn.execute(
        "INSERT INTO nonce_map (nonce, tier, payload_id, embedding_loc, generated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![nonce, tier, payload_id, embedding_loc, generated_at],
    )?;
    Ok(())
}

/// Current time as Unix epoch seconds, formatted as a decimal string (the
/// `events.first_seen_at` / `last_seen_at` text columns store it this way).
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_view_repeats_within_cooldown_update_one_row() {
        let conn = in_memory_conn();
        let now = 1_800_000_000;
        for i in 0..5 {
            record_page_view(&conn, now + i * 10, "9.9.9.9", "Hammer/1.0", "/").unwrap();
        }
        let (rows, last): (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), MAX(received_at) FROM page_views",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(rows, 1, "one row per IP per cooldown window");
        assert_eq!(last, now + 40, "latest fetch time is kept exact");
        // After the cooldown, a new row; another IP always gets its own.
        record_page_view(
            &conn,
            now + 40 + PAGE_VIEW_COOLDOWN_SECS + 1,
            "9.9.9.9",
            "H",
            "/",
        )
        .unwrap();
        record_page_view(&conn, now + 41, "8.8.8.8", "Other", "/").unwrap();
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM page_views", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 3);
    }

    #[test]
    fn page_views_are_capped_oldest_first() {
        let conn = in_memory_conn();
        let now = 1_800_000_000;
        for i in 0..12 {
            record_page_view_capped(&conn, now + i, &format!("10.0.0.{i}"), "UA", "/", 5).unwrap();
        }
        let (rows, oldest): (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), MIN(received_at) FROM page_views",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(rows, 5);
        assert_eq!(oldest, now + 7, "the newest 5 survive");
    }

    fn in_memory_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory DB must open");
        run_migrations(&conn).expect("migrations must succeed");
        conn
    }

    #[test]
    fn test_schema_creates() {
        // Verifies that run_migrations completes without error on an in-memory DB.
        let _conn = in_memory_conn();
    }

    // --- insert_callback_event tests ---

    #[test]
    fn test_insert_callback_event_new_nonce() {
        let conn = in_memory_conn();
        // First insert a nonce in nonce_map
        insert_nonce(&conn, "nonce001", 1, "t1-html", "html_comment").unwrap();
        let (fire_count, is_replay) = insert_callback_event(
            &conn,
            "nonce001",
            1,
            "t1-html",
            "html_comment",
            "sess01",
            "1.2.3.4",
            "TestBot/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(fire_count, 1, "new nonce should have fire_count=1");
        assert!(!is_replay, "new nonce should not be a replay");
    }

    #[test]
    fn test_insert_callback_event_replay() {
        let conn = in_memory_conn();
        insert_nonce(&conn, "nonce002", 1, "t1-html", "html_comment").unwrap();
        // First fire
        insert_callback_event(
            &conn,
            "nonce002",
            1,
            "t1-html",
            "html_comment",
            "sess02",
            "1.2.3.4",
            "TestBot/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        // Second fire — same nonce
        let (fire_count, is_replay) = insert_callback_event(
            &conn,
            "nonce002",
            1,
            "t1-html",
            "html_comment",
            "sess02",
            "1.2.3.4",
            "TestBot/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(fire_count, 2, "second fire should have fire_count=2");
        assert!(is_replay, "second fire should be a replay");
    }

    #[test]
    fn test_insert_callback_event_stores_metadata() {
        let conn = in_memory_conn();
        insert_nonce(&conn, "nonce003", 2, "t2-cond", "meta_tag").unwrap();
        let extra = r#"{"classification":"KnownAgent","headers":{"accept":"text/html"}}"#;
        insert_callback_event(
            &conn,
            "nonce003",
            2,
            "t2-cond",
            "meta_tag",
            "sess03",
            "5.6.7.8",
            "GPTBot/1.0",
            extra,
            None,
            None,
            None,
        )
        .unwrap();
        let (session_id, remote_addr, user_agent, extra_headers): (String, String, String, String) = conn
            .query_row(
                "SELECT session_id, remote_addr, user_agent, extra_headers FROM events WHERE nonce = ?1",
                params!["nonce003"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            ).unwrap();
        assert_eq!(session_id, "sess03");
        assert_eq!(remote_addr, "5.6.7.8");
        assert_eq!(user_agent, "GPTBot/1.0");
        assert_eq!(extra_headers, extra);
    }

    #[test]
    fn test_insert_callback_event_no_body_param() {
        // SRV-07: This test verifies the function signature has no body parameter.
        // The function insert_callback_event does NOT accept a body argument.
        // This is enforced by the type system — the test just confirms it compiles
        // without a body parameter. The comment below references the enforcement.
        // "// SRV-07: No body parameter — metadata-only mode enforced by API design."
        let conn = in_memory_conn();
        insert_nonce(&conn, "nonce004", 1, "t1-html", "html_comment").unwrap();
        // Calling without any body parameter — if a body param existed this wouldn't compile
        let result = insert_callback_event(
            &conn,
            "nonce004",
            1,
            "t1-html",
            "html_comment",
            "sess04",
            "9.9.9.9",
            "NoBodyBot/1.0",
            "{}",
            None,
            None,
            None,
        );
        assert!(result.is_ok());
    }

    // --- lookup_nonce tests ---

    #[test]
    fn test_lookup_nonce_existing() {
        let conn = in_memory_conn();
        insert_nonce(&conn, "findme", 3, "t3-comp", "json_ld").unwrap();
        let result = lookup_nonce(&conn, "findme").unwrap();
        assert!(result.is_some());
        let (tier, payload_id, embedding_loc) = result.unwrap();
        assert_eq!(tier, 3u8);
        assert_eq!(payload_id, "t3-comp");
        assert_eq!(embedding_loc, "json_ld");
    }

    #[test]
    fn test_lookup_nonce_missing() {
        let conn = in_memory_conn();
        let result = lookup_nonce(&conn, "nonexistent_nonce").unwrap();
        assert!(result.is_none());
    }

    // --- count_detections tests ---

    #[test]
    fn test_count_detections_excludes_known_crawlers() {
        let conn = in_memory_conn();
        // Insert a KnownCrawler event — should NOT count
        insert_nonce(&conn, "crawler01", 1, "t1-html", "html_comment").unwrap();
        insert_callback_event(
            &conn,
            "crawler01",
            1,
            "t1-html",
            "html_comment",
            "sess_crawler",
            "10.0.0.1",
            "Googlebot/2.1",
            r#"{"classification":"KnownCrawler:Google","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        // Insert an Unknown event — should count
        insert_nonce(&conn, "agent01", 1, "t1-html", "html_comment").unwrap();
        insert_callback_event(
            &conn,
            "agent01",
            1,
            "t1-html",
            "html_comment",
            "sess_agent",
            "10.0.0.2",
            "EvilAgent/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        let count = count_detections(&conn).unwrap();
        assert_eq!(count, 1, "only non-crawler events should be counted");
    }

    #[test]
    fn test_count_detections_per_session_per_tier() {
        let conn = in_memory_conn();
        // Same session, same tier — should count as 1
        insert_nonce(&conn, "same01", 1, "t1-html", "html_comment").unwrap();
        insert_callback_event(
            &conn,
            "same01",
            1,
            "t1-html",
            "html_comment",
            "sess_same",
            "1.1.1.1",
            "Agent/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        insert_nonce(&conn, "same02", 1, "t1-other", "meta_tag").unwrap();
        insert_callback_event(
            &conn,
            "same02",
            1,
            "t1-other",
            "meta_tag",
            "sess_same",
            "1.1.1.1",
            "Agent/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        // Different tier, same session — should count as 2
        insert_nonce(&conn, "tier2a", 2, "t2-cond", "meta_tag").unwrap();
        insert_callback_event(
            &conn,
            "tier2a",
            2,
            "t2-cond",
            "meta_tag",
            "sess_same",
            "1.1.1.1",
            "Agent/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        let count = count_detections(&conn).unwrap();
        assert_eq!(
            count, 2,
            "same session + same tier should count as 1 detection, different tier as another"
        );
    }

    #[test]
    fn test_schema_replay_fields() {
        let conn = in_memory_conn();
        // Query PRAGMA table_info to enumerate columns for the events table.
        let mut stmt = conn
            .prepare("PRAGMA table_info(events)")
            .expect("pragma must prepare");
        let column_names: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query must execute")
            .filter_map(|r| r.ok())
            .collect();

        for required in &[
            "fire_count",
            "is_replay",
            "session_id",
            "first_seen_at",
            "last_seen_at",
        ] {
            assert!(
                column_names.iter().any(|c| c == required),
                "events table missing required column: {}",
                required
            );
        }
    }

    #[test]
    fn test_insert_nonce() {
        let conn = in_memory_conn();
        insert_nonce(
            &conn,
            "abcdef1234567890",
            1,
            "t1-html-comment",
            "html_comment",
        )
        .expect("insert must succeed");

        let (nonce, tier, payload_id, embedding_loc): (String, u8, String, String) = conn
            .query_row(
                "SELECT nonce, tier, payload_id, embedding_loc FROM nonce_map WHERE nonce = ?1",
                params!["abcdef1234567890"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("row must be retrievable");

        assert_eq!(nonce, "abcdef1234567890");
        assert_eq!(tier, 1u8);
        assert_eq!(payload_id, "t1-html-comment");
        assert_eq!(embedding_loc, "html_comment");
    }

    #[test]
    fn test_detections_by_tier() {
        let conn = in_memory_conn();
        // Tier 1 event
        insert_nonce(&conn, "tier1a", 1, "t1-html", "html_comment").unwrap();
        insert_callback_event(
            &conn,
            "tier1a",
            1,
            "t1-html",
            "html_comment",
            "sess_t1",
            "10.0.0.1",
            "Agent/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        // Tier 3 event
        insert_nonce(&conn, "tier3a", 3, "t3-comp", "json_ld").unwrap();
        insert_callback_event(
            &conn,
            "tier3a",
            3,
            "t3-comp",
            "json_ld",
            "sess_t3",
            "10.0.0.2",
            "Agent/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        // Known crawler should be excluded (T2)
        insert_nonce(&conn, "crawler_t2", 2, "t2-cond", "meta_tag").unwrap();
        insert_callback_event(
            &conn,
            "crawler_t2",
            2,
            "t2-cond",
            "meta_tag",
            "sess_crawler",
            "10.0.0.3",
            "Googlebot/2.1",
            r#"{"classification":"KnownCrawler:Google","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        // Tier 4 event (legitimate)
        insert_nonce(&conn, "tier4a", 4, "t4-meta", "meta_tag").unwrap();
        insert_callback_event(
            &conn,
            "tier4a",
            4,
            "t4-meta",
            "meta_tag",
            "sess_t4",
            "10.0.0.4",
            "Agent/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        // Tier 4 event from a KnownCrawler (must be excluded — D-15-06: filter applies at T4 too)
        insert_nonce(&conn, "crawler_t4", 4, "t4-cmt", "html_comment").unwrap();
        insert_callback_event(
            &conn,
            "crawler_t4",
            4,
            "t4-cmt",
            "html_comment",
            "sess_crawler_t4",
            "10.0.0.5",
            "Googlebot/2.1",
            r#"{"classification":"KnownCrawler:Google","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        // Tier 5 event (legitimate)
        insert_nonce(&conn, "tier5a", 5, "t5-prose", "semantic_prose").unwrap();
        insert_callback_event(
            &conn,
            "tier5a",
            5,
            "t5-prose",
            "semantic_prose",
            "sess_t5",
            "10.0.0.6",
            "Agent/1.0",
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
        let counts = detections_by_tier(&conn).unwrap();
        assert_eq!(
            counts,
            [1, 0, 1, 1, 1],
            "tier 1=1, tier 2=0 (crawler excluded), tier 3=1, tier 4=1 (crawler excluded), tier 5=1"
        );
    }

    #[test]
    fn test_query_report_summary_empty_db() {
        let conn = in_memory_conn();
        let summary = query_report_summary(&conn).unwrap();
        assert_eq!(summary.total_sessions, 0);
        assert_eq!(summary.detection_sessions, 0);
        assert_eq!(summary.crawler_sessions, 0);
        assert_eq!(summary.tier1_sessions, 0);
        assert_eq!(summary.tier2_sessions, 0);
        assert_eq!(summary.tier3_sessions, 0);
        assert_eq!(summary.tier4_sessions, 0);
        assert_eq!(summary.tier5_sessions, 0);
        assert!(summary.earliest_event.is_none());
        assert!(summary.latest_event.is_none());
    }

    #[test]
    fn test_query_report_sessions_null_t4_t5_for_legacy_rows() {
        // Phase 14 D-14-14: always SELECT T4/T5 columns; Option<T> mapping returns None
        // for rows that never wrote those columns (Phase 13 migration guarantees they exist).
        let conn = in_memory_conn();
        // Insert a T1 row (no t4_capability / t5_proof / t5_proof_valid)
        conn.execute(
            "INSERT OR IGNORE INTO nonce_map (nonce, tier, payload_id, embedding_loc, generated_at) \
             VALUES ('n1', 1, 'p1', 'html_comment', '1700000000')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO events (nonce, tier, payload_id, embedding_loc, first_seen_at, last_seen_at, fire_count, is_replay, session_id, remote_addr, user_agent, extra_headers) \
             VALUES ('n1', 1, 'p1', 'html_comment', '1700000000', '1700000000', 1, 0, 'sess1', '1.2.3.4', 'ua', '{\"classification\":\"Unknown\",\"headers\":{}}')",
            [],
        ).unwrap();
        let sessions = query_report_sessions(&conn).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].tier, 1);
        assert_eq!(sessions[0].t4_capability, None);
        assert_eq!(sessions[0].t5_proof, None);
        assert_eq!(sessions[0].t5_proof_valid, None);
    }

    #[test]
    fn test_parameterized_insert() {
        let conn = in_memory_conn();
        // SQL injection payload — must be stored literally, not interpreted.
        let malicious_nonce = "'; DROP TABLE nonce_map; --";
        insert_nonce(&conn, malicious_nonce, 1, "t1-html-comment", "html_comment")
            .expect("parameterized insert must handle SQL metacharacters");

        // The nonce_map table must still exist and contain the malicious string literally.
        let stored: String = conn
            .query_row("SELECT nonce FROM nonce_map LIMIT 1", [], |row| row.get(0))
            .expect("nonce_map must still exist and contain the row");
        assert_eq!(
            stored, malicious_nonce,
            "SQL injection attempt must be stored literally"
        );
    }

    // --- Phase 13 STORE-01 / STORE-02 / STORE-03 migration tests ---

    #[test]
    fn test_schema_t4_columns() {
        let conn = in_memory_conn();
        let mut stmt = conn
            .prepare("PRAGMA table_info(events)")
            .expect("pragma must prepare");
        let column_rows: Vec<(String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
            })
            .expect("query must execute")
            .filter_map(|r| r.ok())
            .collect();
        let t4 = column_rows
            .iter()
            .find(|(n, _)| n == "t4_capability")
            .expect("events must have t4_capability column (STORE-01)");
        assert_eq!(t4.1.to_uppercase(), "TEXT", "t4_capability must be TEXT");
    }

    #[test]
    fn test_schema_t5_columns() {
        let conn = in_memory_conn();
        let mut stmt = conn
            .prepare("PRAGMA table_info(events)")
            .expect("pragma must prepare");
        let column_rows: Vec<(String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
            })
            .expect("query must execute")
            .filter_map(|r| r.ok())
            .collect();
        let proof = column_rows
            .iter()
            .find(|(n, _)| n == "t5_proof")
            .expect("events must have t5_proof column (STORE-02)");
        assert_eq!(proof.1.to_uppercase(), "TEXT", "t5_proof must be TEXT");
        let valid = column_rows
            .iter()
            .find(|(n, _)| n == "t5_proof_valid")
            .expect("events must have t5_proof_valid column (STORE-02)");
        assert_eq!(
            valid.1.to_uppercase(),
            "INTEGER",
            "t5_proof_valid must be INTEGER"
        );
    }

    #[test]
    fn test_migration_idempotent() {
        let conn = Connection::open_in_memory().expect("in-memory DB must open");
        run_migrations(&conn).expect("first migration must succeed");
        run_migrations(&conn)
            .expect("second migration must be a no-op (D-13-17 idempotency via user_version gate)");
        // Confirm user_version is still at the latest schema
        let version: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            version, 5,
            "user_version must remain 5 after multiple run_migrations calls"
        );
    }

    #[test]
    fn test_fresh_db_ends_at_latest_user_version() {
        let conn = in_memory_conn();
        let version: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 5, "fresh DB must end at user_version=5");
    }

    // --- Phase 13 STORE-04 first-write-wins replay tests (D-13-19 / RESEARCH Risk 6) ---

    #[test]
    fn test_insert_callback_event_replay_t4_first_write_wins() {
        let conn = in_memory_conn();
        // Insert first — legitimate capability
        let _ = insert_callback_event(
            &conn,
            "aaaaaaaaaaaaaaaa",
            4,
            "t4-tools-meta",
            "meta_tag",
            "sess_1",
            "127.0.0.1",
            "Mozilla/5.0",
            "{}",
            Some("web_search,browse_page"),
            None,
            None,
        )
        .expect("first insert must succeed");
        // Replay with a different capability — MUST NOT overwrite (D-13-19)
        let (fire_count, is_replay) = insert_callback_event(
            &conn,
            "aaaaaaaaaaaaaaaa",
            4,
            "t4-tools-meta",
            "meta_tag",
            "sess_1",
            "127.0.0.1",
            "Mozilla/5.0",
            "{}",
            Some("SHOULD_NOT_OVERWRITE"),
            None,
            None,
        )
        .expect("second insert must succeed");
        assert_eq!(fire_count, 2, "replay must increment fire_count");
        assert!(is_replay, "replay must set is_replay = true");
        // Confirm original capability preserved (first-write-wins)
        let stored: String = conn
            .query_row(
                "SELECT t4_capability FROM events WHERE nonce = ?1",
                rusqlite::params!["aaaaaaaaaaaaaaaa"],
                |r| r.get(0),
            )
            .expect("row must exist");
        assert_eq!(
            stored, "web_search,browse_page",
            "D-13-19: first-write-wins for t4_capability"
        );
    }

    #[test]
    fn test_insert_callback_event_replay_t5_first_write_wins() {
        let conn = in_memory_conn();
        let _ = insert_callback_event(
            &conn,
            "bbbbbbbbbbbbbbbb",
            5,
            "t5-semantic-prose",
            "semantic_prose",
            "sess_1",
            "127.0.0.1",
            "Mozilla/5.0",
            "{}",
            None,
            Some("042"),
            Some(true),
        )
        .expect("first insert must succeed");
        let (fire_count, is_replay) = insert_callback_event(
            &conn,
            "bbbbbbbbbbbbbbbb",
            5,
            "t5-semantic-prose",
            "semantic_prose",
            "sess_1",
            "127.0.0.1",
            "Mozilla/5.0",
            "{}",
            None,
            Some("999"),
            Some(false),
        )
        .expect("second insert must succeed");
        assert_eq!(fire_count, 2);
        assert!(is_replay);
        let (stored_proof, stored_valid): (String, i64) = conn
            .query_row(
                "SELECT t5_proof, t5_proof_valid FROM events WHERE nonce = ?1",
                rusqlite::params!["bbbbbbbbbbbbbbbb"],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("row must exist");
        assert_eq!(
            stored_proof, "042",
            "D-13-19: first-write-wins for t5_proof"
        );
        assert_eq!(
            stored_valid, 1,
            "D-13-19: first-write-wins for t5_proof_valid"
        );
    }

    // --- hits table (every callback stored) ---

    fn fire(conn: &Connection, nonce: &str, ip: &str, ua: &str) {
        insert_callback_event(
            conn,
            nonce,
            1,
            "t1-html",
            "html_comment",
            "sess",
            ip,
            ua,
            r#"{"classification":"Unknown","headers":{}}"#,
            None,
            None,
            None,
        )
        .unwrap();
    }

    fn hit_count(conn: &Connection, nonce: &str) -> u32 {
        conn.query_row(
            "SELECT COUNT(*) FROM hits WHERE nonce = ?1",
            params![nonce],
            |r| r.get(0),
        )
        .unwrap()
    }

    #[test]
    fn test_every_hit_gets_its_own_row() {
        let conn = in_memory_conn();
        insert_nonce(&conn, "nonce001", 1, "t1-html", "html_comment").unwrap();
        fire(&conn, "nonce001", "1.1.1.1", "AgentA/1.0");
        fire(&conn, "nonce001", "2.2.2.2", "ScraperB/2.0");
        fire(&conn, "nonce001", "2.2.2.2", "ScraperB/2.0");

        assert_eq!(hit_count(&conn, "nonce001"), 3);
        let ips: Vec<String> = conn
            .prepare("SELECT remote_addr FROM hits WHERE nonce = 'nonce001' ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(ips, ["1.1.1.1", "2.2.2.2", "2.2.2.2"]);
        // The summary row still keeps the first fingerprint and counts every fire.
        let (fire_count, ip): (u32, String) = conn
            .query_row(
                "SELECT fire_count, remote_addr FROM events WHERE nonce = 'nonce001'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((fire_count, ip.as_str()), (3, "1.1.1.1"));
    }

    #[test]
    fn test_hits_capped_per_nonce_but_fire_count_continues() {
        let conn = in_memory_conn();
        insert_nonce(&conn, "nonce001", 1, "t1-html", "html_comment").unwrap();
        for _ in 0..(MAX_HITS_PER_NONCE + 5) {
            fire(&conn, "nonce001", "1.1.1.1", "AgentA/1.0");
        }
        fire(&conn, "nonce002", "1.1.1.1", "AgentA/1.0");
        assert_eq!(hit_count(&conn, "nonce001"), MAX_HITS_PER_NONCE);
        assert_eq!(hit_count(&conn, "nonce002"), 1, "cap is per nonce");
        let fires: u32 = conn
            .query_row(
                "SELECT fire_count FROM events WHERE nonce = 'nonce001'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fires, MAX_HITS_PER_NONCE + 5);
    }

    #[test]
    fn test_query_repeat_hits() {
        let conn = in_memory_conn();
        insert_nonce(&conn, "nonce001", 1, "t1-html", "html_comment").unwrap();
        insert_nonce(&conn, "nonce002", 1, "t1-html", "html_comment").unwrap();
        fire(&conn, "nonce001", "1.1.1.1", "AgentA/1.0");
        fire(&conn, "nonce001", "2.2.2.2", "AgentA/1.0");
        fire(&conn, "nonce002", "3.3.3.3", "Once/1.0"); // single hit: excluded

        let hits = query_repeat_hits(&conn).unwrap();
        assert_eq!(hits.len(), 2, "only the nonce with >1 hit is listed");
        assert!(hits.iter().all(|h| h.nonce == "nonce001"));
        assert_eq!(hits[0].hit_number, 1);
        assert_eq!(hits[1].hit_number, 2);
        assert_eq!(hits[1].remote_addr, "2.2.2.2");
        assert_eq!(
            hits[1].timing,
            HitTiming::OutOfBand,
            "no page fetch recorded"
        );
        assert!(
            hits[1].seeded_at.is_some(),
            "seeded_at comes from nonce_map"
        );
        assert_eq!((hits[1].stored_hits, hits[1].total_fires), (2, 2));
    }

    fn now_secs() -> i64 {
        chrono_now().parse().unwrap()
    }

    fn timing_of_last_hit(conn: &Connection) -> (Option<String>, Option<i64>) {
        conn.query_row(
            "SELECT timing, last_page_view_at FROM hits ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    }

    #[test]
    fn test_hit_timing_classify_boundaries() {
        assert_eq!(HitTiming::classify(1000, Some(1000)), HitTiming::InSession);
        assert_eq!(
            HitTiming::classify(1000 + IN_SESSION_WINDOW_SECS, Some(1000)),
            HitTiming::InSession
        );
        assert_eq!(
            HitTiming::classify(1001 + IN_SESSION_WINDOW_SECS, Some(1000)),
            HitTiming::Delayed
        );
        assert_eq!(HitTiming::classify(1000, None), HitTiming::OutOfBand);
        assert_eq!(HitTiming::parse(None), HitTiming::Unknown);
        for t in [
            HitTiming::InSession,
            HitTiming::Delayed,
            HitTiming::OutOfBand,
        ] {
            assert_eq!(HitTiming::parse(Some(t.as_str())), t);
        }
    }

    #[test]
    fn test_hit_after_recent_page_view_is_in_session() {
        let conn = in_memory_conn();
        let now = now_secs();
        record_page_view(&conn, now - 60, "1.1.1.1", "AgentA/1.0", "/").unwrap();
        fire(&conn, "nonce001", "1.1.1.1", "AgentA/1.0");
        let (timing, pv) = timing_of_last_hit(&conn);
        assert_eq!(timing.as_deref(), Some("in-session"));
        assert_eq!(pv, Some(now - 60));
    }

    #[test]
    fn test_hit_long_after_page_view_is_delayed() {
        let conn = in_memory_conn();
        let now = now_secs();
        record_page_view(&conn, now - 3 * 86400, "1.1.1.1", "AgentA/1.0", "/").unwrap();
        fire(&conn, "nonce001", "1.1.1.1", "AgentA/1.0");
        assert_eq!(timing_of_last_hit(&conn).0.as_deref(), Some("delayed"));
    }

    #[test]
    fn test_hit_from_ip_that_never_fetched_is_out_of_band() {
        let conn = in_memory_conn();
        let now = now_secs();
        record_page_view(&conn, now - 60, "1.1.1.1", "AgentA/1.0", "/").unwrap();
        fire(&conn, "nonce001", "9.9.9.9", "Fetcher/1.0");
        let (timing, pv) = timing_of_last_hit(&conn);
        assert_eq!(timing.as_deref(), Some("out-of-band"));
        assert_eq!(pv, None);
    }

    #[test]
    fn test_page_views_pruned_after_retention() {
        let conn = in_memory_conn();
        let now = now_secs();
        record_page_view(
            &conn,
            now - PAGE_VIEW_RETENTION_SECS - 10,
            "1.1.1.1",
            "A",
            "/",
        )
        .unwrap();
        record_page_view(&conn, now, "2.2.2.2", "B", "/").unwrap();
        let n: u32 = conn
            .query_row("SELECT COUNT(*) FROM page_views", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 1, "old fetch pruned");
    }

    #[test]
    fn test_timing_counts_and_notable_hits() {
        let conn = in_memory_conn();
        let now = now_secs();
        insert_nonce(&conn, "nonce001", 1, "t1-html", "html_comment").unwrap();
        record_page_view(&conn, now - 10, "1.1.1.1", "A", "/").unwrap();
        record_page_view(&conn, now - 7200, "2.2.2.2", "B", "/").unwrap();
        fire(&conn, "nonce001", "1.1.1.1", "A"); // in-session
        fire(&conn, "nonce001", "2.2.2.2", "B"); // delayed
        fire(&conn, "nonce001", "3.3.3.3", "C"); // out-of-band
                                                 // A pre-v3 hit with no timing.
        conn.execute(
            "INSERT INTO hits (nonce, tier, received_at) VALUES ('old', 2, 1)",
            [],
        )
        .unwrap();

        let counts = query_hit_timing_counts(&conn).unwrap();
        assert_eq!(
            counts,
            vec![
                TimingCounts {
                    tier: 1,
                    in_session: 1,
                    delayed: 1,
                    out_of_band: 1,
                    unknown: 0
                },
                TimingCounts {
                    tier: 2,
                    unknown: 1,
                    ..Default::default()
                },
            ]
        );

        let (notable, total) = query_notable_hits(&conn, 10).unwrap();
        assert_eq!(total, 2);
        assert_eq!(notable[0].timing, HitTiming::OutOfBand, "newest first");
        assert_eq!(notable[1].timing, HitTiming::Delayed);
        assert_eq!(notable[1].last_page_view_at, Some(now - 7200));
        assert!(notable[1].seeded_at.is_some());

        let (limited, total) = query_notable_hits(&conn, 1).unwrap();
        assert_eq!((limited.len(), total), (1, 2));
    }

    #[test]
    fn test_exfil_summary_and_no_value_stored() {
        let conn = in_memory_conn();
        insert_nonce(&conn, "nonce006", 6, "t6-canary-comment", "html_comment").unwrap();
        assert_eq!(exfil_summary(&conn).unwrap(), ExfilSummary::default());
        for exfil in [
            crate::types::ExfilCheck {
                matched: false,
                value_len: 30,
            },
            crate::types::ExfilCheck {
                matched: true,
                value_len: 25,
            },
            crate::types::ExfilCheck {
                matched: false,
                value_len: 9,
            },
        ] {
            insert_callback_event_detailed(
                &conn,
                "nonce006",
                6,
                "t6-canary-comment",
                "html_comment",
                "sess",
                "1.1.1.1",
                "Agent/1.0",
                r#"{"classification":"Unknown","headers":{}}"#,
                None,
                None,
                None,
                Some(exfil),
            )
            .unwrap();
        }
        assert_eq!(
            exfil_summary(&conn).unwrap(),
            ExfilSummary {
                leaked: true,
                non_matching: 2
            }
        );
        let lens: Vec<u32> = conn
            .prepare("SELECT exfil_value_len FROM hits ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(lens, [30, 25, 9]);
    }

    #[test]
    fn test_seed_insert_duplicate_label_rejected() {
        let conn = in_memory_conn();
        insert_seed(
            &conn,
            "aaaaaaaaaaaaaaaa",
            "claude-md",
            Some("api repo"),
            100,
        )
        .unwrap();
        assert!(insert_seed(&conn, "bbbbbbbbbbbbbbbb", "claude-md", None, 101).is_err());
    }

    #[test]
    fn test_seed_hits_recorded_only_for_known_seeds_and_capped() {
        let conn = in_memory_conn();
        insert_seed(
            &conn,
            "aaaaaaaaaaaaaaaa",
            "claude-md",
            Some("api repo"),
            100,
        )
        .unwrap();
        insert_seed(&conn, "cccccccccccccccc", "unused", None, 100).unwrap();
        assert_eq!(
            record_seed_hit(&conn, "ffffffffffffffff", 200, "1.1.1.1", "UA", "Unknown").unwrap(),
            None
        );
        let seed = record_seed_hit(&conn, "aaaaaaaaaaaaaaaa", 200, "1.1.1.1", "UA", "Unknown")
            .unwrap()
            .unwrap();
        assert_eq!(seed.label, "claude-md");
        assert_eq!(seed.note.as_deref(), Some("api repo"));
        for i in 0..(MAX_HITS_PER_SEED + 2) {
            record_seed_hit(
                &conn,
                "aaaaaaaaaaaaaaaa",
                300 + i as i64,
                "2.2.2.2",
                "UA2",
                "KnownCrawler:Google",
            )
            .unwrap();
        }
        let seeds = list_seeds(&conn).unwrap();
        assert_eq!(seeds.len(), 2);
        let s = &seeds[0];
        assert_eq!(s.label, "claude-md");
        assert_eq!(
            s.hit_count,
            MAX_HITS_PER_SEED + 3,
            "true total keeps counting"
        );
        assert_eq!(s.first_hit_at, Some(200));
        assert_eq!(s.distinct_ips, 2);
        assert_eq!(seeds[1].hit_count, 0);
        assert_eq!(seeds[1].first_hit_at, None);

        let (rows, total) = recent_seed_hits(&conn, 5).unwrap();
        assert_eq!(total, MAX_HITS_PER_SEED, "stored rows stop at the cap");
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[0].classification, "KnownCrawler:Google");
        // Seed hits never touch the tier tables.
        let events: u32 = conn
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .unwrap();
        let hits: u32 = conn
            .query_row("SELECT COUNT(*) FROM hits", [], |r| r.get(0))
            .unwrap();
        assert_eq!((events, hits), (0, 0));
    }
}
