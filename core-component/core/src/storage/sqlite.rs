use rusqlite::Connection;

use crate::storage::{
    CACHE_DB, CODEX_DB, Storage,
    error::{SqliteError, StorageError},
    sqlite_version::{SqliteStoreVersion, layout::SqliteLayout, v1::SQLITESTOREV1},
    utils,
};

pub struct SqliteStore {
    pub cache_conn: Connection,
    pub codex_conn: Connection,
    layout: Box<dyn SqliteLayout>,
}

impl SqliteStore {
    fn new(codex_conn: Connection, cache_conn: Connection, version: SqliteStoreVersion) -> Self {
        SqliteStore {
            cache_conn,
            codex_conn,
            layout: get_sqlite_version(version),
        }
    }

    fn version(&self) -> SqliteStoreVersion {
        self.layout.version()
    }

    pub fn build(
        storage: crate::storage::Storage,
        version: SqliteStoreVersion,
    ) -> Result<SqliteStore, SqliteError> {
        let database_folder = &storage.database_folder;
        let codex_db = Connection::open(database_folder.join(CODEX_DB))?;
        let cache_db = Connection::open(database_folder.join(CACHE_DB))?;

        Ok(SqliteStore::new(codex_db, cache_db, version))
    }

    pub fn open(
        storage: &crate::storage::Storage,
    ) -> Result<crate::storage::sqlite::SqliteStore, SqliteError> {
        if let Ok(read_codex) = utils::read_codex_config(storage.root_folder()) {
            let database_folder = &storage.database_folder;
            let version = SqliteStoreVersion::parse(&read_codex.version.storage_sqlite);
            let codex_db = Connection::open(database_folder.join(CODEX_DB))?;
            let cache_db = Connection::open(database_folder.join(CACHE_DB))?;

            return Ok(SqliteStore::new(codex_db, cache_db, version));
        }

        Err(SqliteError::AssertionFail(
            "Failed to open codex file".into(),
        ))
    }

    pub fn create_base(&self) -> Result<(), SqliteError> {
        self.layout.create_base(self)
    }

    pub fn delete(&self) -> Result<(), SqliteError> {
        self.layout.delete(self)
    }
}

fn get_sqlite_version(version: SqliteStoreVersion) -> Box<dyn SqliteLayout> {
    match version {
        SqliteStoreVersion::V1 => Box::new(SQLITESTOREV1),
    }
}
