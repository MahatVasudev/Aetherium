use std::path::{Path, PathBuf};

use blake3::Hash;
use uuid::Uuid;

use crate::{
    codex::{
        Codex,
        versions::{CodexVersion, layout_for},
    },
    storage::{self, CODEX_FILE, error::StorageError, versions::StorageVersion},
};

impl Codex {
    pub fn add_file(
        &self,
        from_filename: &PathBuf,
        bytes: usize,
    ) -> Result<FileAddedResponse, StorageError> {
        self.layout.add_file(self, from_filename, bytes)
    }
    pub fn validate_codex_at(root_folder: &Path) -> bool {
        // Validates Codex - By
        //  - If codex.toml exists in the given path
        //  - If structure is valid
        //  - If id of the codex is valid uuid
        //  - If version is valid
        //
        // If any error occurs, then just return false
        //  All or Nothing Approach
        let codex = root_folder.join(CODEX_FILE);

        if !codex.exists() {
            return false;
        }

        match storage::utils::read_codex_config(root_folder) {
            Ok(codex_conf) => {
                if !Uuid::parse_str(&codex_conf.identity.id).is_ok() {
                    return false;
                }
                let recorded_codex_version = CodexVersion::parse(&codex_conf.version.codex);
                if recorded_codex_version.is_none() {
                    return false;
                }
                if let Some(storage_version) = StorageVersion::parse(&codex_conf.version.storage) {
                    if !layout_for(recorded_codex_version.unwrap())
                        .supported_storage()
                        .contains(&storage_version)
                    {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            _ => return false,
        }
        true
    }
    fn search_files(&self, query: &str) -> Vec<PathBuf> {
        unimplemented!()
    }
    fn read_file(&self, file_name: &str) -> String {
        unimplemented!()
    }
}

pub struct FileAddedResponse {
    pub file_path: PathBuf,
    pub file_id: String,
    pub file_hash: Hash,
}

#[cfg(test)]
mod testing {
    use std::{fs, path::Path};

    use tempfile::{NamedTempFile, tempdir};

    use crate::storage::{CACHE_DB, CODEX_DB, DATA_FOLDER, sqlite_version::SqliteStoreVersion};

    use super::*;
    #[ignore = "Testing in your own environment"]
    #[test]
    // Checking whether it works on my local machine, for evidence and satisfaction
    fn writing_file_ok() {
        let main_path = Path::new("/home/clyde/Documents/first-knowledge1").to_path_buf();

        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let sqlite_version = SqliteStoreVersion::V1;
        let codex =
            Codex::build(&main_path, codex_version, storage_version, sqlite_version).unwrap();
        let raw_filename = Path::new("/home/clyde/Downloads/sml_importance .pdf");
        let buffersize: usize = 512;

        let written = codex
            .add_file(&raw_filename.to_path_buf(), buffersize)
            .unwrap();

        let name = Uuid::new_v4().to_string();
        fs::write(main_path.join(DATA_FOLDER).join(&name), b"hello world").unwrap();

        println!(
            "{:?}",
            fs::read_dir(codex.storage.data_folder())
                .unwrap()
                .map(|f| f.unwrap().path().to_string_lossy().to_string())
                .collect::<Vec<String>>()
        );

        codex.storage.sync().unwrap();

        fs::write(
            main_path.join(DATA_FOLDER).join(&name),
            b"hello world changed",
        )
        .unwrap();

        codex.storage.sync().unwrap();
        let sqlite_opp = codex.storage.sqlite().unwrap();

        let codex_conn = sqlite_opp.codex_conn.borrow_mut();
        let cache_conn = sqlite_opp.cache_conn.borrow_mut();
        // assert!(written.is_ok());
        assert!(codex.storage.database_folder().join(CODEX_DB).is_file());
        assert!(codex.storage.database_folder().join(CACHE_DB).is_file());

        assert!(codex_conn.table_exists(Some("main"), "files").unwrap());
        assert!(codex_conn.table_exists(Some("main"), "info").unwrap());

        assert!(
            cache_conn
                .table_exists(Some("main"), "content_cache")
                .unwrap()
        )
    }
    #[test]
    fn writing_file_ok_2() {
        let codex_path = tempdir().unwrap();
        let mut raw_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut raw_file, b"hello hello hello hello").unwrap();

        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let sqlite_version = SqliteStoreVersion::V1;
        let codex = Codex::build(
            &codex_path.keep(),
            codex_version,
            storage_version,
            sqlite_version,
        )
        .unwrap();

        let sqlite_opp = codex.storage.sqlite().unwrap();

        let buffersize: usize = 512;
        let written = codex
            .add_file(&raw_file.path().to_path_buf(), buffersize)
            .unwrap();

        let codex_conn = codex.storage.sqlite().unwrap().codex_conn.borrow_mut();
        let cache_conn = codex.storage.sqlite().unwrap().cache_conn.borrow_mut();
        // assert!(written.is_ok());
        assert!(codex.storage.database_folder().join(CODEX_DB).is_file());
        assert!(codex.storage.database_folder().join(CACHE_DB).is_file());

        assert!(codex_conn.table_exists(Some("main"), "files").unwrap());
        assert!(codex_conn.table_exists(Some("main"), "info").unwrap());

        assert!(
            cache_conn
                .table_exists(Some("main"), "content_cache")
                .unwrap()
        )
    }
}
