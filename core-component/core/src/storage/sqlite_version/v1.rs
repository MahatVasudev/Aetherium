use crate::storage::{
    error::SqliteError, sqlite::SqliteStore, sqlite_version::layout::SqliteLayout,
};

pub struct SQLITESTOREV1;

impl SqliteLayout for SQLITESTOREV1 {
    fn version(&self) -> super::SqliteStoreVersion {
        super::SqliteStoreVersion::V1
    }

    fn delete(&self, sqlite_store: &SqliteStore) -> Result<(), SqliteError> {
        todo!()
    }

    fn create_base(&self, sqlite_store: &SqliteStore) -> Result<(), SqliteError> {
        // FIX: These are not final stage, we need to think of a concrete database schema
        // For Both codex db and cache db, they should be practical, and be optimized with low
        // powered system...
        // NOTE: Creating Base Data Tables in Codex DB
        {
            sqlite_store.codex_conn.execute(
                "create table if not exists info (
                    version text not null,
                    was_updated datetime default current_timestamp
                )",
                (),
            )?;
            sqlite_store.codex_conn.execute(
                "create table if not exists files (
                        id text primary key,
                        name text not null,
                        created_at datetime default current_timestamp
                )",
                (),
            )?;
        }
        // NOTE: Creating Base Data Tables in Cache DB
        {
            sqlite_store.cache_conn.execute(
                "create table if not exists content_cache (
                    id text primary key,
                    content text not null,
                    cached_at datetime default current_timestamp
                )",
                (),
            )?;
        }

        Ok(())
    }
}
