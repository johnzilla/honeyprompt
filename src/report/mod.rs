// Stub — to be implemented (TDD GREEN phase)
use rusqlite::Connection;

pub fn md_escape(_s: &str) -> String {
    todo!("implement md_escape")
}

pub fn generate_report(_conn: &Connection) -> anyhow::Result<String> {
    todo!("implement generate_report")
}
