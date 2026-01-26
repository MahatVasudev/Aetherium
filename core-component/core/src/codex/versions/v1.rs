use std::{fs, io::Read, path::Path};

use anyhow::{Context, anyhow};
use uuid::Uuid;

use crate::codex::{
    CODEX_FILE, Codex, DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER, LATEST_VERSION, codex_config,
    file_reading::FileAddedResponse,
    is_valid_version, utils,
    versions::{CodexLayout, CodexVersion},
};

pub struct CodexV1;

impl CodexLayout for CodexV1 {
    fn build(&self, root_folder: &std::path::Path) -> anyhow::Result<crate::codex::Codex> {
        // WARN: Incomplete Implementation (works for now)
        if Self.is_codex(&root_folder) {
            anyhow::bail!("This folder is already an codex")
        }
        let tmp = root_folder.with_extension("codex_tmp");
        fs::create_dir(&tmp)
            .with_context(|| format!("failed to create root dir {:?}", root_folder))?;
        let codex_name = match root_folder.file_name() {
            Some(value) => String::from(value.to_string_lossy()),
            None => return anyhow::bail!("Codex Name Couldnt be determined"),
        };
        let iid = uuid::Uuid::new_v4();
        let result = utils::make_all_codex_dirs(&tmp);
        if result.is_err() {
            let _ = fs::remove_dir_all(&tmp);
        }
        result?;
        if let Err(e) = Self.write_first_codex(&tmp, &codex_name, iid.to_string()) {
            let _ = fs::remove_dir_all(&tmp);
            return Err(e);
        }
        fs::rename(&tmp, &root_folder)?;
        let version = utils::version_error(LATEST_VERSION)?;
        Ok(Codex::new(
            root_folder.to_path_buf(),
            codex_name,
            iid.to_string(),
            version,
        ))
    }

    fn open(&self, root_folder: std::path::PathBuf) -> anyhow::Result<crate::codex::Codex> {
        // WARN: Incomplete Implementation (works for now)
        if !Self.is_codex(&root_folder) {
            anyhow::bail!("This is not a codex")
        }

        // Create the folders if they are missing

        let folder_name = Path::new(&root_folder).to_path_buf();
        let result = utils::make_all_codex_dirs(&folder_name);
        result?;

        let read_codex = codex_config::read_codex_config(&root_folder)?;
        let version = utils::version_error(read_codex.version.version.as_str())?;
        Ok(Codex::new(
            root_folder,
            read_codex.identity.name,
            read_codex.identity.id,
            version,
        ))
    }

    fn write_first_codex(
        &self,
        foldername: &std::path::Path,
        codex_name: &String,
        generated_id: String,
    ) -> anyhow::Result<()> {
        let created_time = chrono::Local::now();
        let codex_content = format!(
            "[identity]\nid=\"{generated_id}\"\nname=\"{codex_name}\"\n[version]\nversion=\"{LATEST_VERSION}\"\ncreated_at=\"{created_time}\""
        );
        fs::write(foldername.join(CODEX_FILE), codex_content)?;

        Ok(())
    }

    fn validate_codex_at(&self, root_folder: &std::path::Path) -> bool {
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

        match codex_config::read_codex_config(root_folder) {
            Ok(codex_conf) => {
                if !Uuid::parse_str(&codex_conf.identity.id).is_ok() {
                    return false;
                }
                match CodexVersion::parse(&codex_conf.version.version) {
                    None => return false,
                    _ => (),
                }
            }
            _ => return false,
        }
        true
    }

    fn is_codex(&self, root_folder: &std::path::Path) -> bool {
        Self.validate_codex_at(root_folder)
    }

    fn add_file(
        &self,
        codex: &Codex,
        data: &mut dyn Read,
        byte: usize,
        filename: &str,
    ) -> anyhow::Result<crate::codex::file_reading::FileAddedResponse> {
        // WARN: Incomplete Implementation (works for now)
        let temp_folder = utils::create_temp_if_not_exists(&codex.root_folder)?;
        let temp_filename = temp_folder.join(filename);
        let (file_hash, file_id) = match utils::write_to_file(&temp_filename, data, byte) {
            Ok(result) => result,
            Err(result) => {
                fs::remove_file(&temp_filename)?;
                return Err(result);
            }
        };

        let final_filename = codex.data_folder.join(&file_id);
        fs::rename(&temp_filename, &final_filename)?;
        Ok(FileAddedResponse {
            file_path: final_filename,
            file_id,
            file_hash,
        })
    }

    fn validate(&self, codex: &Codex) -> bool {
        Self.validate_codex_at(&codex.root_folder)
    }

    fn search_files(&self, query: &str) -> Vec<std::path::PathBuf> {
        todo!()
    }

    fn read_file(&self, file_name: &str) -> String {
        todo!()
    }
}
