use crate::{
    codex::{Codex, file_reading::FileAddedResponse, versions::CodexVersion},
    storage::{error::StorageError, sqlite_version::SqliteStoreVersion, versions::StorageVersion},
};

use std::path::{Path, PathBuf};

pub trait CodexLayout: Send + Sync {
    fn version(&self) -> CodexVersion;
    fn build(
        &self,
        root_folder: &Path,
        storage_version: StorageVersion,
        sqlite_version: SqliteStoreVersion,
    ) -> Result<Codex, StorageError>;
    fn first_codex_content(
        &self,
        codex_name: &str,
        generated_id: &str,
        storage_version: StorageVersion,
        sqlite_version: SqliteStoreVersion,
    ) -> String;
    fn add_file(
        &self,
        codex: &Codex,
        from_filename: &PathBuf,
        name: Option<String>,
        byte: usize,
    ) -> Result<FileAddedResponse, StorageError>;

    fn delete_file(&self, codex: &Codex, file_id: &str) -> Result<(), StorageError>;
    fn search_files(&self, query: &str) -> Vec<PathBuf>;
    fn read_file(&self, codex: &Codex, file_id: &str) -> String;
    fn supported_storage(&self) -> &'static [StorageVersion];
}
