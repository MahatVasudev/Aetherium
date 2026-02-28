use std::{io, path::PathBuf};

use blake3::Hash;

use crate::{
    codex::codex_config::CodexConfig,
    storage::{
        Storage,
        error::StorageError,
        storage_types::{self, FileInSystem},
        versions::StorageVersion,
    },
};

pub trait StorageLayout {
    fn version(&self) -> StorageVersion;
    fn build(&self, root_folder: &PathBuf) -> Result<Storage, StorageError>;
    fn make_dirs(&self, root_folder: &PathBuf) -> Result<(), StorageError>;
    fn add_files(
        &self,
        storage: &Storage,
        from_filename: &PathBuf,
        byte: usize,
    ) -> Result<storage_types::FileInSystem, StorageError>;
    fn create_new_codex_file(&self, storage: &Storage, content: &str) -> Result<(), StorageError>;
    fn exists_dirs(&self, root_folder: &PathBuf) -> bool;
    fn all_folders(&self) -> &'static [&'static str];
    fn read_codex_file(&self, storage: &Storage) -> Result<CodexConfig, StorageError>;

    fn read_file(
        &self,
        storage: &Storage,
        file_id: &str,
    ) -> Result<Box<dyn io::Read>, StorageError>;
    fn delete_file(&self, storage: &Storage, file_id: &str) -> Result<(), StorageError>;
    fn list_files(&self, storage: &Storage) -> Result<Vec<FileInSystem>, StorageError>;
    fn sync(&self, storage: &Storage) -> Result<(), StorageError>;
}
