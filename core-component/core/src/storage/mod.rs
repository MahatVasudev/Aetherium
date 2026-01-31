use anyhow;
use blake3::Hash;
use std::path::{Path, PathBuf};

use crate::storage::versions::{
    StorageVersion,
    layout::{self, StorageLayout},
    load_storageversion,
};
mod sqlite;
pub mod utils;
pub mod versions;

// For Storage Management In SqliteDB

pub const CODEX_FILE: &str = "codex.toml";
pub const DATA_FOLDER: &str = "data";
pub const INDEXED_FOLDER: &str = "indexed";
pub const DATABASE_FOLDER: &str = "database";
pub struct Storage {
    root_folder: PathBuf,
    data_folder: PathBuf,
    indexed_folder: PathBuf,
    database_folder: PathBuf,
    layout: Box<dyn StorageLayout>,
}

impl Storage {
    fn new(root_folder: &PathBuf, version: StorageVersion) -> Storage {
        Storage {
            data_folder: root_folder.join(DATA_FOLDER),
            indexed_folder: root_folder.join(INDEXED_FOLDER),
            database_folder: root_folder.join(DATABASE_FOLDER),
            root_folder: root_folder.clone(),
            layout: load_storageversion(&version),
        }
    }

    pub fn root_folder(&self) -> &PathBuf {
        &self.root_folder
    }
    pub fn version(&self) -> StorageVersion {
        self.layout.version()
    }

    pub fn build(root_folder: &PathBuf, version: StorageVersion) -> anyhow::Result<Storage> {
        load_storageversion(&version).build(root_folder)
    }

    pub fn open(root_folder: &PathBuf) -> anyhow::Result<Storage> {
        let read_codex_file = utils::read_codex_config(root_folder)?;

        let version = StorageVersion::parse(&read_codex_file.version.storage);

        if let None = version {
            anyhow::bail!(
                "Storage Version {} not valid",
                read_codex_file.version.storage
            )
        }
        let ver = load_storageversion(&version.unwrap());

        if !ver.exists_dirs(root_folder) {
            anyhow::bail!(
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
    ) -> anyhow::Result<(Hash, String)> {
        self.layout.add_files(self, from_filename, byte)
    }

    pub fn create_new_codex(&self, content: &str) -> anyhow::Result<()> {
        self.layout.create_new_codex_file(self, content)
    }

    pub fn exists_dirs(&self, root_folder: &PathBuf) -> bool {
        self.layout.exists_dirs(root_folder)
    }

    pub fn all_folders(&self) -> &'static [&'static str] {
        self.layout.all_folders()
    }
}
