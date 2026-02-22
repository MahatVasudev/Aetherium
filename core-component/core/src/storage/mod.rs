use blake3::Hash;
use std::path::PathBuf;

use crate::{
    storage::{
        error::StorageError,
        sqlite::SqliteStore,
        versions::{StorageVersion, layout::StorageLayout, load_storageversion},
    },
    storage_assert,
};
pub mod error;
pub mod sqlite;
pub mod sqlite_version;
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
    sqlite: Option<SqliteStore>,
}

impl Storage {
    fn new(root_folder: &PathBuf, version: StorageVersion) -> Storage {
        Storage {
            data_folder: root_folder.join(DATA_FOLDER),
            indexed_folder: root_folder.join(INDEXED_FOLDER),
            database_folder: root_folder.join(DATABASE_FOLDER),
            root_folder: root_folder.clone(),
            layout: load_storageversion(&version),
            sqlite: None,
        }
    }

    pub fn sqlite(&mut self) -> Result<&mut SqliteStore, StorageError> {
        if self.sqlite.is_none() {
            let store = SqliteStore::open(self)?;
            self.sqlite = Some(store);
        }

        Ok(self.sqlite.as_mut().unwrap())
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
    ) -> Result<(Hash, String), StorageError> {
        self.layout.add_files(self, from_filename, byte)
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
}
