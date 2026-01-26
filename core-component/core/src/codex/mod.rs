mod codex_config;
pub mod file_reading;
pub mod utils;
pub mod versions;

use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Context;
use uuid::Uuid;

use crate::codex::{
    file_reading::FileAddedResponse,
    versions::{CodexLayout, CodexVersion, layout_for, v1::CodexV1},
};

fn is_valid_version(v: &str) -> bool {
    matches!(v, "v1.0.0")
}

const LATEST_VERSION: &str = "v1.0.0";
const CODEX_FILE: &str = "codex.toml";
const DATA_FOLDER: &str = "data";
const INDEXED_FOLDER: &str = "indexed";
const DATABASE_FOLDER: &str = "database";

pub struct Codex {
    pub id: String,
    pub name: String,
    pub version: CodexVersion,
    pub root_folder: PathBuf,
    pub data_folder: PathBuf,
    pub indexed_folder: PathBuf,
    pub database_folder: PathBuf,
    pub config_file: PathBuf,
}

impl Codex {}

impl Codex {
    fn build(root_folder: &Path) -> anyhow::Result<Codex> {
        // WARN: Incomplete Implementation (works for now)
        let version = CodexVersion::V1;
        layout_for(version).build(root_folder)
    }
    fn new(root_folder: PathBuf, name: String, id: String, version: CodexVersion) -> Codex {
        Codex {
            data_folder: root_folder.join(DATA_FOLDER),
            indexed_folder: root_folder.join(INDEXED_FOLDER),
            database_folder: root_folder.join(DATABASE_FOLDER),
            config_file: root_folder.join(CODEX_FILE),
            version: version,
            name: name,
            root_folder: root_folder,
            id: id,
        }
    }
    fn layout(&self) -> &'static dyn CodexLayout {
        layout_for(self.version)
    }
    fn open(root_folder: PathBuf) -> anyhow::Result<Codex> {
        // WARN: Incomplete Implementation (works for now)
        let version = CodexVersion::V1;
        layout_for(version).open(root_folder)
    }
    fn write_first_codex(
        foldername: &Path,
        codex_name: &String,
        generated_id: String,
    ) -> anyhow::Result<()> {
        let version = CodexVersion::V1;
        layout_for(version).write_first_codex(foldername, codex_name, generated_id)
    }
    fn validate_codex_at(root_folder: &Path) -> bool {
        let version = CodexVersion::V1;
        layout_for(version).validate_codex_at(root_folder)
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    use tempfile::{Builder, tempdir};
    #[test]
    fn it_should_work() {
        let temp = tempdir().unwrap();
        let foldername = temp.path().join("my_codex");
        let _ = Codex::build(&foldername).expect("Codex should have worked");

        assert!(foldername.join(CODEX_FILE).exists());
        assert!(foldername.join(DATA_FOLDER).exists());
        assert!(foldername.join(INDEXED_FOLDER).exists());
        assert!(foldername.join(DATABASE_FOLDER).exists());
    }

    #[test]
    fn it_should_not_work() {
        let temp = tempdir().unwrap();
        let foldername = temp.path().join("my_codex");
        fs::create_dir(&foldername).unwrap();
        fs::write(foldername.join(CODEX_FILE), "somecontent\nversion 1").unwrap();

        let result = Codex::build(&foldername);

        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn permissions_error() {
        let owner_rwx = fs::Permissions::from_mode(0o400);
        let tempdir = Builder::new().permissions(owner_rwx).tempdir().unwrap();

        let foldername = tempdir.path().join("my_codex");

        let result = Codex::build(&foldername);
        assert!(result.is_err());
    }
}
