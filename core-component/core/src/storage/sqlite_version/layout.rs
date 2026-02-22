use crate::storage::{
    Storage, error::SqliteError, sqlite::SqliteStore, sqlite_version::SqliteStoreVersion,
};

pub trait SqliteLayout {
    fn version(&self) -> SqliteStoreVersion;

    fn create_base(&self, sqlite_store: &SqliteStore) -> Result<(), SqliteError>;

    fn delete(&self, sqlite_store: &SqliteStore) -> Result<(), SqliteError>;
}
