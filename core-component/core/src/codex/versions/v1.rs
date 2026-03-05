use std::{ffi::OsStr, fs, path::PathBuf};

use uuid;

use crate::{
    codex::{Codex, file_reading::FileAddedResponse, layout::CodexLayout, versions::CodexVersion},
    storage::{
        Storage,
        error::StorageError,
        sqlite_version::{self, SqliteStoreVersion},
        versions::StorageVersion,
    },
    storage_assert,
    tfidf::TFIDFCorpus,
};

pub struct CodexV1;

impl CodexLayout for CodexV1 {
    fn version(&self) -> CodexVersion {
        CodexVersion::V1
    }
    fn build(
        &self,
        root_folder: &std::path::Path,
        storage_version: StorageVersion,
        sqlite_version: SqliteStoreVersion,
    ) -> Result<crate::codex::Codex, StorageError> {
        // WARN: Incomplete Implementation (works for now)
        if Codex::validate_codex_at(root_folder) {
            storage_assert!("This folder is already an codex")
        }

        if !self.supported_storage().contains(&storage_version) {
            storage_assert!(
                "Storage Version {} is not supported on Codex Version {}",
                storage_version.as_str(),
                self.version().as_str()
            )
        }
        let tmp = root_folder.with_extension("codex_tmp");
        fs::create_dir_all(&tmp)?;

        let codex_name = match root_folder.file_name() {
            Some(value) => String::from(value.to_string_lossy()),
            None => return storage_assert!("Codex Name Couldnt be determined"),
        };
        let iid = uuid::Uuid::new_v4();
        let storage = Storage::build(&tmp.to_path_buf(), storage_version)?;

        let codex_content = self.first_codex_content(
            &codex_name,
            &iid.to_string(),
            storage.version(),
            sqlite_version,
        );
        if let Err(e) = storage.create_new_codex(&codex_content) {
            let _ = fs::remove_dir_all(&tmp);
            return Err(e);
        }
        fs::rename(&tmp, &root_folder)?;

        let storage = Storage::open(&root_folder.to_path_buf())?;

        storage.sqlite()?.create_base()?;
        Ok(Codex::new(
            codex_name,
            iid.to_string(),
            self.version(),
            storage,
        ))
    }

    fn add_file(
        &self,
        codex: &Codex,
        from_filename: &PathBuf,
        byte: usize,
    ) -> Result<crate::codex::file_reading::FileAddedResponse, StorageError> {
        // WARN: Incomplete Implementation (works for now)
        let file = codex.storage.add_files(from_filename, byte)?;
        let hash = file.get_hash(&codex.storage)?;

        let fileinsql = sqlite_version::v1::types::FileInSQL {
            id: file.id.clone(),
            name: from_filename
                .file_name()
                .unwrap_or(&OsStr::new("Untitled"))
                .to_string_lossy()
                .to_string(),
            hash: hash.to_hex().to_string(),
            extension: file.extention.clone(),
            created_at: None,
            modified_at: None,
        };
        codex.storage.sqlite()?.add_metadata(&fileinsql)?;

        if file.extention.starts_with("text/") {
            let tf = TFIDFCorpus::compute_tf(codex.storage.data_folder().join(&file.id), byte)?;
            codex.storage.sqlite()?.reindex_file(&fileinsql, &tf)?;
        }

        Ok(FileAddedResponse {
            file_path: from_filename.to_path_buf(),
            file_id: file.id.clone(),
            file_hash: file.get_hash(&codex.storage)?,
        })
    }

    fn search_files(&self, query: &str) -> Vec<std::path::PathBuf> {
        todo!()
    }

    fn read_file(&self, codex: &Codex, file_id: &str) -> String {
        todo!()
    }
    fn first_codex_content(
        &self,
        codex_name: &str,
        generated_id: &str,
        storage_version: StorageVersion,
        sqlite_version: SqliteStoreVersion,
    ) -> String {
        let created_time = chrono::Local::now();
        let version = self.version().as_str();
        let storage_ver = storage_version.as_str();
        let sqlite_ver = sqlite_version.as_str();
        let codex_content = format!(
            "[identity]
id=\"{generated_id}\"
name=\"{codex_name}\"
[version]
codex=\"{version}\"
storage=\"{storage_ver}\"
storage_sqlite=\"{sqlite_ver}\"
created_at=\"{created_time}\""
        );

        codex_content
    }

    fn supported_storage(&self) -> &'static [StorageVersion] {
        &[StorageVersion::V1]
    }
}
