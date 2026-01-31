use std::path::PathBuf;

use blake3::Hash;

use crate::{
    codex::codex_config::CodexConfig,
    storage::{Storage, versions::StorageVersion},
};

pub trait StorageLayout {
    fn version(&self) -> StorageVersion;
    fn build(&self, root_folder: &PathBuf) -> anyhow::Result<Storage>;
    fn make_dirs(&self, root_folder: &PathBuf) -> anyhow::Result<()>;
    fn add_files(
        &self,
        storage: &Storage,
        from_filename: &PathBuf,
        byte: usize,
    ) -> anyhow::Result<(Hash, String)>;
    fn create_new_codex_file(&self, storage: &Storage, content: &str) -> anyhow::Result<()>;
    fn exists_dirs(&self, root_folder: &PathBuf) -> bool;
    fn all_folders(&self) -> &'static [&'static str];
    fn append_codex_file(&self, storage: &Storage, content: &str);
    fn update_codex_properties(&self, storage: &Storage);
    fn read_codex_file(&self, storage: &Storage) -> anyhow::Result<CodexConfig>;
}
