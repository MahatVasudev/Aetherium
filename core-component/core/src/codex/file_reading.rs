use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use blake3::Hash;
use uuid::Uuid;

use crate::{
    codex::{
        Codex, CodexLayout, utils,
        versions::{CodexVersion, layout_for},
    },
    storage::{self, CODEX_FILE, versions::StorageVersion},
};

impl Codex {
    fn add_file(&self, from_filename: &PathBuf, bytes: usize) -> anyhow::Result<FileAddedResponse> {
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
                if let None = recorded_codex_version {
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
    use std::{fs::File, path::Path};

    use tempfile::{NamedTempFile, tempdir};

    use super::*;
    #[ignore = "Testing in your own environment"]
    #[test]
    // Checking whether it works on my local machine, for evidence and satisfaction
    fn writing_file_ok() {
        let codex =
            Codex::open(Path::new("/home/clyde/Documents/first-knowledge").to_path_buf()).unwrap();
        let raw_filename = Path::new("/home/clyde/Downloads/sml_importance .pdf");
        let buffersize: usize = 512;

        let written = codex.add_file(&raw_filename.to_path_buf(), buffersize);

        assert!(written.is_ok())
    }
    #[test]
    fn writing_file_ok_2() {
        let codex_path = tempdir().unwrap();
        let mut raw_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut raw_file, b"hello hello hello hello").unwrap();

        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let codex = Codex::build(&codex_path.keep(), codex_version, storage_version).unwrap();
        let buffersize: usize = 512;
        let written = codex.add_file(&raw_file.path().to_path_buf(), buffersize);

        assert!(written.is_ok())
    }
}
