use blake3::Hash;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::{
    storage::{
        error::StorageError,
        sqlite::SqliteStore,
        storage_types::{FileInSystem, SyncEvent},
        versions::{StorageVersion, layout::StorageLayout, load_storageversion},
    },
    storage_assert,
};
pub mod error;
pub mod sqlite;
pub mod sqlite_version;
pub mod storage_types;
pub mod utils;
pub mod versions;

// For Storage Management In SqliteDB

pub const CODEX_FILE: &str = "codex.toml";
pub const DATA_FOLDER: &str = "data";
pub const INDEXED_FOLDER: &str = "indexed";
pub const DATABASE_FOLDER: &str = "database";
pub const CACHE_DB: &str = "cache.sqlite";
pub const CODEX_DB: &str = "codex.sqlite";

pub struct Storage {
    root_folder: PathBuf,
    data_folder: PathBuf,
    indexed_folder: PathBuf,
    database_folder: PathBuf,
    layout: Box<dyn StorageLayout>,
    sqlite: OnceLock<SqliteStore>,
}

impl Storage {
    fn new(root_folder: &PathBuf, version: StorageVersion) -> Storage {
        Storage {
            data_folder: root_folder.join(DATA_FOLDER),
            indexed_folder: root_folder.join(INDEXED_FOLDER),
            database_folder: root_folder.join(DATABASE_FOLDER),
            root_folder: root_folder.clone(),
            layout: load_storageversion(&version),
            sqlite: OnceLock::new(),
        }
    }

    pub fn sqlite(&self) -> Result<&SqliteStore, StorageError> {
        if let Some(store) = self.sqlite.get() {
            return Ok(store);
        }

        let store = SqliteStore::open(self)?;

        // ignore error if already initialized by race
        let _ = self.sqlite.set(store);

        Ok(self.sqlite.get().unwrap())
    }

    pub fn root_folder(&self) -> &PathBuf {
        &self.root_folder
    }

    pub fn database_folder(&self) -> &PathBuf {
        &self.database_folder
    }
    pub fn data_folder(&self) -> &PathBuf {
        &self.data_folder
    }

    pub fn indexed_folder(&self) -> &PathBuf {
        &self.indexed_folder
    }

    pub fn version(&self) -> StorageVersion {
        self.layout.version()
    }

    pub fn build(root_folder: &PathBuf, version: StorageVersion) -> Result<Storage, StorageError> {
        load_storageversion(&version).build(root_folder)
    }

    pub fn open(root_folder: &PathBuf) -> Result<Storage, StorageError> {
        let read_codex_file = utils::read_codex_config(root_folder)?;

        let version = StorageVersion::parse(&read_codex_file.version.storage);

        if version.is_none() {
            storage_assert!(
                "Storage Version {} not valid",
                read_codex_file.version.storage
            )
        }
        let ver = load_storageversion(&version.unwrap());

        if !ver.exists_dirs(root_folder) {
            storage_assert!(
                "Storage Folder Structure Not Validated, version: {}",
                version.unwrap().as_str()
            )
        }

        Ok(Storage::new(root_folder, version.unwrap()))
    }

    pub fn add_files(
        &self,
        from_filename: &PathBuf,
        byte: usize,
    ) -> Result<FileInSystem, StorageError> {
        self.layout.add_files(self, from_filename, byte)
    }

    pub fn delete_file(&self, file_id: &str) -> Result<(), StorageError> {
        self.layout.delete_file(self, file_id)
    }

    pub fn list_files(&self) -> Result<Vec<FileInSystem>, StorageError> {
        self.layout.list_files(self)
    }

    pub fn create_new_codex(&self, content: &str) -> Result<(), StorageError> {
        self.layout.create_new_codex_file(self, content)
    }

    pub fn exists_dirs(&self, root_folder: &PathBuf) -> bool {
        self.layout.exists_dirs(root_folder)
    }

    pub fn all_folders(&self) -> &'static [&'static str] {
        self.layout.all_folders()
    }

    pub fn sync(&self, on_progress: &dyn Fn(SyncEvent)) -> Result<(), StorageError> {
        self.layout.sync(self, on_progress)
    }
}
