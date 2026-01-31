use anyhow::Context;
use blake3::Hash;
use uuid::Uuid;

use crate::{
    codex::codex_config::CodexConfig,
    storage::{
        CODEX_FILE, DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER, Storage, utils,
        versions::{StorageVersion, layout::StorageLayout},
    },
};
use std::{
    fs::{self, File},
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub struct StorageV1;

impl StorageLayout for StorageV1 {
    fn version(&self) -> StorageVersion {
        StorageVersion::V1
    }
    fn build(&self, root_folder: &PathBuf) -> anyhow::Result<Storage> {
        if let Err(error) = fs::create_dir(&root_folder) {
            if !(error.kind() == ErrorKind::AlreadyExists) {
                return Err(error)
                    .with_context(|| format!("failed to create root dir {:?}", root_folder));
            }
        };
        self.make_dirs(root_folder)?;
        Ok(Storage::new(root_folder, self.version()))
    }

    fn make_dirs(&self, root_folder: &PathBuf) -> anyhow::Result<()> {
        for folder in self.all_folders() {
            fs::create_dir_all(root_folder.join(folder))?;
        }
        Ok(())
    }

    fn exists_dirs(&self, root_folder: &PathBuf) -> bool {
        for folders in self.all_folders() {
            if !root_folder.join(folders).exists() {
                return false;
            }
        }

        return true;
    }

    fn all_folders(&self) -> &'static [&'static str] {
        &[DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER]
    }

    fn add_files(
        &self,
        storage: &Storage,
        from_filename: &PathBuf,
        byte: usize,
    ) -> anyhow::Result<(Hash, String)> {
        let filename = match from_filename.file_name() {
            Some(filename) => filename.to_string_lossy().to_string(),
            None => Uuid::new_v4().to_string(),
        };
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(storage.data_folder.join(filename))?;

        let data = File::open(from_filename)?;
        utils::write_to_file(&mut file, data, byte)
    }

    fn create_new_codex_file(&self, storage: &Storage, content: &str) -> anyhow::Result<()> {
        fs::write(&storage.root_folder.join(CODEX_FILE), content)?;

        Ok(())
    }

    fn append_codex_file(&self, storage: &Storage, content: &str) {
        todo!()
    }

    fn update_codex_properties(&self, storage: &Storage) {
        todo!()
    }

    fn read_codex_file(&self, storage: &Storage) -> anyhow::Result<CodexConfig> {
        utils::read_codex_config(&storage.root_folder)
    }
}
