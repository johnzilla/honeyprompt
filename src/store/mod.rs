use rusqlite::{Connection, params};

/// Open or create a SQLite database at the given path, running schema migrations.
pub fn open_or_create_db(path: &std::path::Path) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Execute all schema migrations against an open connection.
pub fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
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
    )
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

/// Return the current UTC time as an ISO-8601 string without pulling in a time crate.
/// Uses the OS epoch via std and formats manually.
fn chrono_now() -> String {
    // We only need a stable, sortable string for stored timestamps.
    // Using std epoch seconds avoids adding a time crate dependency in Phase 1.
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

        for required in &["fire_count", "is_replay", "session_id", "first_seen_at", "last_seen_at"] {
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
        insert_nonce(&conn, "abcdef1234567890", 1, "t1-html-comment", "html_comment")
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
    fn test_parameterized_insert() {
        let conn = in_memory_conn();
        // SQL injection payload — must be stored literally, not interpreted.
        let malicious_nonce = "'; DROP TABLE nonce_map; --";
        insert_nonce(&conn, malicious_nonce, 1, "t1-html-comment", "html_comment")
            .expect("parameterized insert must handle SQL metacharacters");

        // The nonce_map table must still exist and contain the malicious string literally.
        let stored: String = conn
            .query_row(
                "SELECT nonce FROM nonce_map LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("nonce_map must still exist and contain the row");
        assert_eq!(
            stored, malicious_nonce,
            "SQL injection attempt must be stored literally"
        );
    }
}
