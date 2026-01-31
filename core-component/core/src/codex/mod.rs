pub mod codex_config;
pub mod file_reading;
pub mod utils;
pub mod versions;

use anyhow::anyhow;

use crate::storage::{self, Storage, versions::StorageVersion};
use std::path::{Path, PathBuf};

use crate::codex::versions::{CodexLayout, CodexVersion, layout_for, v1::CodexV1};

pub struct Codex {
    pub id: String,
    pub name: String,
    layout: Box<dyn CodexLayout>,
    pub storage: Storage,
}

impl Codex {
    fn build(
        root_folder: &PathBuf,
        codex_version: CodexVersion,
        storage_version: StorageVersion,
    ) -> anyhow::Result<Codex> {
        // WARN: Incomplete Implementation (works for now)
        layout_for(codex_version).build(root_folder, storage_version)
    }
    fn new(name: String, id: String, version: CodexVersion, storage: Storage) -> Codex {
        Codex {
            id,
            name,
            layout: layout_for(version),
            storage,
        }
    }
    fn open(root_folder: PathBuf) -> anyhow::Result<Codex> {
        // WARN: Incomplete Implementation (works for now)
        if Codex::validate_codex_at(&root_folder) {
            anyhow::bail!(
                "Codex Not Validated, directory given: {}",
                root_folder.to_string_lossy().to_string()
            )
        }

        let read_codex = storage::utils::read_codex_config(&root_folder)?;
        // Safe because already checked in validate_codex_at associated function at the top
        let codex_version = CodexVersion::parse(&read_codex.version.codex).unwrap();
        let storage = Storage::open(&root_folder)?;

        Ok(Codex::new(
            read_codex.identity.name,
            read_codex.identity.id,
            codex_version,
            storage,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt};

    use crate::storage::{CODEX_FILE, DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER};

    use super::*;

    use tempfile::{Builder, tempdir};
    #[test]
    fn it_should_work() {
        let temp = tempdir().unwrap();
        let foldername = temp.path().join("my_codex");
        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let _ = Codex::build(&foldername, codex_version, storage_version)
            .expect("Codex should have worked");

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
        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;

        let result = Codex::build(&foldername, codex_version, storage_version);

        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn own_system_test() {
        use crate::storage::{DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER};

        let codex_path = Path::new("/home/clyde/Documents/first-knowledge").to_path_buf();
        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let codex = Codex::build(&codex_path, codex_version, storage_version);

        assert!(codex_path.join(CODEX_FILE).exists());
        assert!(codex_path.join(DATA_FOLDER).exists());
        assert!(codex_path.join(INDEXED_FOLDER).exists());
        assert!(codex_path.join(DATABASE_FOLDER).exists());
    }

    #[cfg(unix)]
    #[test]
    fn permissions_error() {
        let owner_rwx = fs::Permissions::from_mode(0o400);
        let tempdir = Builder::new().permissions(owner_rwx).tempdir().unwrap();

        let foldername = tempdir.path().join("my_codex");

        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let result = Codex::build(&foldername, codex_version, storage_version);
        assert!(result.is_err());
    }
}
