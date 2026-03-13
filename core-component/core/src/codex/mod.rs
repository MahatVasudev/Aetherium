pub mod codex_config;
pub mod file_reading;
pub mod layout;
pub mod utils;
pub mod versions;

use crate::{
    codex::layout::CodexLayout,
    storage::{
        self, Storage,
        error::StorageError,
        sqlite_version::{self, SqliteStoreVersion},
        versions::StorageVersion,
    },
    storage_assert,
};

use std::path::{Path, PathBuf};

use crate::codex::versions::{CodexVersion, layout_for};

pub const DEFAULT_CHUNK_SIZE: usize = 512;
pub const DEFAULT_READ_CHUNK_SIZE: usize = 512;

pub struct Codex {
    pub id: String,
    pub name: String,
    layout: Box<dyn CodexLayout>,
    pub storage: Storage,
}

impl Codex {
    pub fn version(&self) -> CodexVersion {
        self.layout.version()
    }
    pub fn build<P: AsRef<Path>>(
        root_folder: P,
        codex_version: CodexVersion,
        storage_version: StorageVersion,
        sqlite_version: SqliteStoreVersion,
    ) -> Result<Codex, StorageError> {
        // WARN: Incomplete Implementation (works for now)
        layout_for(codex_version).build(root_folder.as_ref(), storage_version, sqlite_version)
    }
    fn new(name: String, id: String, version: CodexVersion, storage: Storage) -> Codex {
        Codex {
            id,
            name,
            layout: layout_for(version),
            storage,
        }
    }
    pub fn open(root_folder: PathBuf) -> Result<Codex, StorageError> {
        // WARN: Incomplete Implementation (works for now)

        let mut root_folder = root_folder;
        if !Codex::validate_codex_at(&root_folder) {
            root_folder = match Codex::find_codex_root(&root_folder) {
                Some(fl) => fl,
                None => {
                    storage_assert!(
                        "Codex Not Validated, directory given: {}",
                        root_folder.to_string_lossy().to_string()
                    )
                }
            }
        }

        let read_codex = storage::utils::read_codex_config(&root_folder)?;
        // NOTE: Safe because already checked in validate_codex_at associated function at the top
        let codex_version = CodexVersion::parse(&read_codex.version.codex).unwrap();
        let storage = Storage::open(&root_folder)?;

        Ok(Codex::new(
            read_codex.identity.name,
            read_codex.identity.id,
            codex_version,
            storage,
        ))
    }
    pub fn find_codex_root(start: &Path) -> Option<PathBuf> {
        let mut current = start;
        loop {
            if Codex::validate_codex_at(current) {
                return Some(current.to_path_buf());
            }
            match current.parent() {
                Some(parent) => current = parent,
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt};

    use crate::storage::{CODEX_FILE, DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER};
    use std::path::Path;

    use super::*;

    use tempfile::{Builder, tempdir};
    #[test]
    fn it_should_work() {
        let temp = tempdir().unwrap();
        let foldername = temp.path().join("my_codex");
        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let sqlite_version = SqliteStoreVersion::V1;
        let _ = Codex::build(&foldername, codex_version, storage_version, sqlite_version)
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
        let sqlite_version = SqliteStoreVersion::V1;

        let result = Codex::build(&foldername, codex_version, storage_version, sqlite_version);

        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[ignore = "Running In Own System"]
    #[test]
    fn own_system_test() {
        use crate::storage::{DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER};

        let codex_path = Path::new("/home/clyde/Documents/first-knowledge").to_path_buf();
        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let sqlite_version = SqliteStoreVersion::V1;
        let codex = Codex::build(&codex_path, codex_version, storage_version, sqlite_version);

        assert!(codex.is_ok());
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
        let sqlite_version = SqliteStoreVersion::V1;
        let result = Codex::build(&foldername, codex_version, storage_version, sqlite_version);
        assert!(result.is_err());
    }
}
