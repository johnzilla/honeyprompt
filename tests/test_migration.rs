//! Phase 13 — STORE-03 regression: a v4.0-shape DB migrates to v5.0 schema
//! additively and preserves T1-T3 row data byte-identically.
//!
//! Implements the additive-migration guarantee from D-13-17. Constructs the
//! v4.0 schema programmatically (no binary fixture) per 13-VALIDATION.md Wave 0.

use honeyprompt::store;
use rusqlite::Connection;
use tempfile::NamedTempFile;

/// Construct a v4.0-shape events table (no T4/T5 columns, user_version=0)
/// and insert three representative T1-T3 rows.
///
/// This mirrors the exact CREATE TABLE from src/store/mod.rs::run_migrations
/// at the point Phase 13 began — no additive columns, no user_version bump.
fn build_v4_schema_db(conn: &Connection) {
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
        ",
    )
    .expect("v4.0 baseline schema must apply");

    // Explicit user_version=0 (this is the v4.0 default state).
    conn.execute_batch("PRAGMA user_version = 0;")
        .expect("set user_version=0");

    // Three representative T1-T3 rows
    for (nonce, tier, payload_id, loc) in &[
        ("1111111111111111", 1u8, "t1-html-comment", "html_comment"),
        ("2222222222222222", 2u8, "t2-meta-branch", "meta_tag"),
        (
            "3333333333333333",
            3u8,
            "t3-invisible-element",
            "invisible_element",
        ),
    ] {
        conn.execute(
            "INSERT INTO events (nonce, tier, payload_id, embedding_loc, first_seen_at, last_seen_at, fire_count, is_replay)
             VALUES (?1, ?2, ?3, ?4, '2026-04-01T00:00:00Z', '2026-04-01T00:00:00Z', 1, 0)",
            rusqlite::params![nonce, tier, payload_id, loc],
        )
        .expect("T1-T3 row insert must succeed");
    }
}

#[test]
fn test_v4_db_opens_unchanged() {
    let tmp = NamedTempFile::new().expect("tempfile must open");
    let path = tmp.path().to_path_buf();

    // Build a v4.0-shape DB.
    {
        let conn = Connection::open(&path).expect("open v4 DB");
        build_v4_schema_db(&conn);
    }

    // Now apply Phase 13 migrations by opening via the production entry point.
    let conn = store::open_or_create_db(&path).expect("open_or_create_db must apply migrations");

    // Phase 13 columns must exist.
    let mut stmt = conn.prepare("PRAGMA table_info(events)").expect("pragma");
    let cols: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .expect("query")
        .filter_map(|r| r.ok())
        .collect();
    for required in &["t4_capability", "t5_proof", "t5_proof_valid"] {
        assert!(
            cols.iter().any(|c| c == required),
            "STORE-03 regression: events table missing {} after migration",
            required
        );
    }

    // user_version must be 1.
    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(
        version, 1,
        "STORE-03: user_version must be 1 after migrating v4.0 DB"
    );

    // Existing T1-T3 rows must read back with original field values, with the
    // three new columns all NULL (additive migration — zero-touch on existing rows).
    struct MigratedRow {
        nonce: String,
        tier: u8,
        payload_id: String,
        t4_capability: Option<String>,
        t5_proof: Option<String>,
        t5_proof_valid: Option<i64>,
    }

    let mut stmt = conn
        .prepare(
            "SELECT nonce, tier, payload_id, t4_capability, t5_proof, t5_proof_valid
             FROM events ORDER BY nonce",
        )
        .unwrap();
    let rows: Vec<MigratedRow> = stmt
        .query_map([], |r| {
            Ok(MigratedRow {
                nonce: r.get(0)?,
                tier: r.get(1)?,
                payload_id: r.get(2)?,
                t4_capability: r.get(3)?,
                t5_proof: r.get(4)?,
                t5_proof_valid: r.get(5)?,
            })
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert_eq!(rows.len(), 3, "all 3 v4.0 rows must be readable");
    assert_eq!(rows[0].nonce, "1111111111111111");
    assert_eq!(rows[0].tier, 1);
    assert_eq!(rows[0].payload_id, "t1-html-comment");
    assert_eq!(rows[1].nonce, "2222222222222222");
    assert_eq!(rows[1].tier, 2);
    assert_eq!(rows[1].payload_id, "t2-meta-branch");
    assert_eq!(rows[2].nonce, "3333333333333333");
    assert_eq!(rows[2].tier, 3);
    assert_eq!(rows[2].payload_id, "t3-invisible-element");
    for r in &rows {
        assert!(
            r.t4_capability.is_none(),
            "v4.0 row {} must have NULL t4_capability (additive migration)",
            r.nonce
        );
        assert!(
            r.t5_proof.is_none(),
            "v4.0 row {} must have NULL t5_proof",
            r.nonce
        );
        assert!(
            r.t5_proof_valid.is_none(),
            "v4.0 row {} must have NULL t5_proof_valid",
            r.nonce
        );
    }
}

#[test]
fn test_v4_db_migration_idempotent_across_reopen() {
    let tmp = NamedTempFile::new().expect("tempfile must open");
    let path = tmp.path().to_path_buf();

    // v4.0 → v5.0 once.
    {
        let conn = Connection::open(&path).expect("open v4 DB");
        build_v4_schema_db(&conn);
    }
    {
        let _conn = store::open_or_create_db(&path).expect("first migration run");
    }
    // Now reopen (simulates production process restart). Must not error
    // with "duplicate column name: t4_capability" (RESEARCH.md Pitfall 2).
    let conn =
        store::open_or_create_db(&path).expect("reopen after migration must succeed (idempotent)");
    let version: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 1, "user_version stable across reopen");
}
