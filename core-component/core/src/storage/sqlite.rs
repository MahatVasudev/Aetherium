use rusqlite::Connection;

pub struct SqliteStore {
    conn: Connection,
}
