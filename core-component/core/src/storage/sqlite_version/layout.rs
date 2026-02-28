use crate::storage::{
    error::SqliteError,
    sqlite::SqliteStore,
    sqlite_version::{SqliteStoreVersion, v1::types},
};

pub trait SqliteLayout {
    fn version(&self) -> SqliteStoreVersion;

    fn create_base(&self, sqlite_store: &SqliteStore) -> Result<(), SqliteError>;

    fn delete(&self, sqlite_store: &SqliteStore, file: String) -> Result<(), SqliteError>;

    fn add_metadata(
        &self,
        sqlite_store: &SqliteStore,
        file: types::Files,
    ) -> Result<(), SqliteError>;

    fn get_all_files(&self, sqlite_store: &SqliteStore) -> Result<Vec<types::Files>, SqliteError>;

    fn update_hash(
        &self,
        sqlite_store: &SqliteStore,
        id: String,
        hash: String,
    ) -> Result<(), SqliteError>;
}
