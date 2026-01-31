use std::{fs, io::Read, path::PathBuf};

use anyhow::Context;
use uuid;

use crate::{
    codex::{
        Codex,
        file_reading::FileAddedResponse,
        versions::{CodexLayout, CodexVersion},
    },
    storage::{Storage, versions::StorageVersion},
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
    ) -> anyhow::Result<crate::codex::Codex> {
        // WARN: Incomplete Implementation (works for now)
        if Codex::validate_codex_at(root_folder) {
            anyhow::bail!("This folder is already an codex")
        }

        if !self.supported_storage().contains(&storage_version) {
            anyhow::bail!(
                "Storage Version {} is not supported on Codex Version {}",
                storage_version.as_str(),
                self.version().as_str()
            )
        }
        let tmp = root_folder.with_extension("codex_tmp");
        fs::create_dir_all(&tmp)?;

        let codex_name = match root_folder.file_name() {
            Some(value) => String::from(value.to_string_lossy()),
            None => return anyhow::bail!("Codex Name Couldnt be determined"),
        };
        let iid = uuid::Uuid::new_v4();
        let storage = Storage::build(&tmp.to_path_buf(), storage_version)?;

        let codex_content =
            self.first_codex_content(&codex_name, &iid.to_string(), storage.version());
        if let Err(e) = storage.create_new_codex(&codex_content) {
            let _ = fs::remove_dir_all(&tmp);
            return Err(e);
        }
        fs::rename(&tmp, &root_folder)?;

        let storage = Storage::open(&root_folder.to_path_buf())?;
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
    ) -> anyhow::Result<crate::codex::file_reading::FileAddedResponse> {
        // WARN: Incomplete Implementation (works for now)
        let (file_hash, filename) =
            codex
                .storage
                .add_files(from_filename, byte)
                .with_context(|| {
                    format!(
                        "Error adding file {} to Codex {}; codex_id: {}",
                        from_filename.to_string_lossy().to_string(),
                        codex.name,
                        codex.id
                    )
                })?;

        Ok(FileAddedResponse {
            file_path: from_filename.to_path_buf(),
            file_id: filename,
            file_hash,
        })
    }

    fn search_files(&self, query: &str) -> Vec<std::path::PathBuf> {
        todo!()
    }

    fn read_file(&self, file_name: &str) -> String {
        todo!()
    }
    fn first_codex_content(
        &self,
        codex_name: &str,
        generated_id: &str,
        storage_version: StorageVersion,
    ) -> String {
        let created_time = chrono::Local::now();
        let version = self.version().as_str();
        let storage_ver = storage_version.as_str();
        let codex_content = format!(
            "[identity]\nid=\"{generated_id}\"\nname=\"{codex_name}\"\n[version]\ncodex=\"{version}\"\nstorage=\"{storage_ver}\"\ncreated_at=\"{created_time}\""
        );

        String::from(codex_content)
    }

    fn supported_storage(&self) -> &'static [StorageVersion] {
        &[StorageVersion::V1]
    }
}
