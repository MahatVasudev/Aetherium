use infer::Infer;
use uuid::Uuid;

use crate::{
    codex::{self, codex_config::CodexConfig},
    storage::{
        CODEX_FILE, DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER, Storage,
        error::StorageError,
        sqlite_version::v1::types::FileInSQL,
        storage_types::{self, FileInSystem, SyncEvent},
        utils,
        versions::{StorageVersion, layout::StorageLayout},
    },
    storage_assert,
    tfidf::TFIDFCorpus,
};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

pub struct StorageV1;

impl StorageLayout for StorageV1 {
    fn version(&self) -> StorageVersion {
        StorageVersion::V1
    }
    fn build(&self, root_folder: &PathBuf) -> Result<Storage, StorageError> {
        match fs::create_dir(&root_folder) {
            Err(error) => {
                if !(error.kind() == ErrorKind::AlreadyExists) {
                    return Err(StorageError::Io(error.to_string()));
                }
            }
            _ => (),
        };
        self.make_dirs(root_folder)?;
        Ok(Storage::new(root_folder, self.version()))
    }

    fn make_dirs(&self, root_folder: &PathBuf) -> Result<(), StorageError> {
        for folder in self.all_folders() {
            if let Err(err) = fs::create_dir_all(root_folder.join(folder)) {
                return Err(StorageError::Io(err.to_string()));
            };
        }
        Ok(())
    }

    fn exists_dirs(&self, root_folder: &PathBuf) -> bool {
        for folders in self.all_folders() {
            if !root_folder.join(folders).exists() {
                return false;
            }
        }

        true
    }

    fn all_folders(&self) -> &'static [&'static str] {
        &[DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER]
    }

    fn add_files(
        &self,
        storage: &Storage,
        from_filename: &PathBuf,
        byte: usize,
    ) -> Result<storage_types::FileInSystem, StorageError> {
        if byte == 0 {
            return storage_assert!("buffer size must be greater than 0, got: {}", byte);
        }

        let filename = Uuid::new_v4().to_string();
        let tmpfolder = storage.data_folder.join(".tmp");
        fs::create_dir_all(&tmpfolder)?;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&tmpfolder.join(&filename))?;

        let data = File::open(from_filename)?;
        let result = utils::write_to_file(&mut file, data, byte);
        if result.is_err() {
            fs::remove_file(&tmpfolder.join(&filename))?;
        }

        result?;

        let final_file_name = storage.data_folder.join(&filename);
        fs::rename(tmpfolder.join(&filename), &final_file_name)?;

        let mime: String = utils::get_data_extension(&from_filename)?;
        let modified_at =
            utils::convert_datestring(storage.data_folder.join(&filename).metadata()?.modified()?);
        Ok(FileInSystem {
            id: filename,
            extention: mime,
            modified_at,
        })
    }

    fn create_new_codex_file(&self, storage: &Storage, content: &str) -> Result<(), StorageError> {
        let codex_file = storage.root_folder.join(CODEX_FILE);
        fs::write(&codex_file, content)?;
        let mut perms = fs::metadata(&codex_file)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&codex_file, perms)?;
        Ok(())
    }

    fn read_codex_file(&self, storage: &Storage) -> Result<CodexConfig, StorageError> {
        let result = utils::read_codex_config(&storage.root_folder)?;
        Ok(result)
    }

    fn read_file(
        &self,
        storage: &Storage,
        file_id: &str,
    ) -> Result<Box<dyn io::Read>, StorageError> {
        // Go to Data Location
        // find the file with the given file_id
        // return the read buffer
        //
        // TODO: After Implementing SQLITE Management, Instead of just read buffer, we will return
        // a struct which will have all of the information with the read buffer

        let file_path = storage.data_folder.join(file_id);

        if !file_path.is_file() {
            return Err(StorageError::not_found(file_id));
        }
        let read_buff = fs::OpenOptions::new().read(true).open(file_path)?;
        return Ok(Box::new(read_buff));
    }

    fn delete_file(&self, storage: &Storage, file_id: &str) -> Result<(), StorageError> {
        // Go to Data Location
        // find the file with the given file_id
        // Try to delete it
        // Return any error if occured, else just return an empty Ok message, signaling the job was
        // successful
        //
        // TODO: After Implementing SQLITE Management, The record of the data, and any indexes,
        // caches should be removed, iff the data is removed from the data location

        let file_path = std::path::absolute(storage.data_folder().join(file_id))?;

        if !file_path.is_file() && file_path.starts_with(storage.data_folder()) {
            return Err(StorageError::not_found(file_id));
        }
        fs::remove_file(file_path)?;

        Ok(())
    }

    fn list_files(&self, storage: &Storage) -> Result<Vec<FileInSystem>, StorageError> {
        // Go to Data Location
        // find all of the files
        // TODO: Afer Implementing SQLITE Management, we will return a list of a struct which
        // should contain file id, name, extension, date added
        let entries = fs::read_dir(&storage.data_folder)?;
        let mut files: Vec<FileInSystem> = vec![];
        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let file_name = path
                .file_name()
                .ok_or_else(|| StorageError::Corrupt("file missing name".into()))?
                .to_string_lossy()
                .to_string();

            let mime = utils::get_data_extension(&path)?;
            let modified_at = utils::convert_datestring(path.metadata()?.modified()?);

            files.push(FileInSystem {
                id: file_name,
                extention: mime,
                modified_at,
            })
        }

        Ok(files)
    }

    fn sync(
        &self,
        storage: &Storage,
        on_progress: &mut dyn FnMut(SyncEvent),
    ) -> Result<(), StorageError> {
        // FIX: Check if data has been updated between file-system and sqlite
        // Treat filesystem as the ground truth, and sqlite as the overview of filesystem
        //
        // NOTE: Working
        // - Check all files present in sqlite, and check all files present in file system,
        // - If files id is present in file system and not in sqlite, add data in sqlite
        // - if files id is present in sqlite and not in file system then remove data id in sqlite

        // Make hash set of ids from both file system and sqlite

        // Make hash of each data to see whether they have been changed or not, if they have update
        // the hash in the sqlite and update modified at

        // VULN: Currently this approach is very unsafe, as we also have to check whether the files
        // in filesystem are valid uuid they are
        // valid uuid....
        // We also have to check before adding the file to the sql that there are no same hash, if
        // they are same, we delete the file
        let fis = storage.list_files()?;

        let sqlite_conn = storage.sqlite()?;
        println!("this was the problem of files");
        let fisql = sqlite_conn.get_all_files()?;
        println!("this was not the problem of files");
        let mut added: usize = 0;
        let mut removed: usize = 0;
        let mut updated: usize = 0;
        let fis_map: HashMap<String, FileInSystem> =
            fis.into_iter().map(|f| (f.id.clone(), f)).collect();
        let fisql_map: HashMap<String, FileInSQL> =
            fisql.into_iter().map(|f| (f.id.clone(), f)).collect();

        for (id, fs_file) in &fis_map {
            if !fisql_map.contains_key(id) {
                let file = FileInSQL {
                    id: id.into(),
                    name: "Untitled".into(),
                    hash: fs_file.get_hash(&storage)?.to_hex().to_string(),
                    extension: fs_file.extention.clone(),
                    created_at: None,
                    modified_at: None,
                    embedded_at: None,
                    indexed_at: None,
                };
                sqlite_conn.add_metadata(&file)?;
                if fs_file.extention.starts_with("text/") {
                    let tf = TFIDFCorpus::compute_tf(storage.data_folder().join(&id), 150)?;
                    sqlite_conn.reindex_file(&file, &tf)?;
                }

                on_progress(SyncEvent::FileAdded {
                    id: file.id,
                    name: file.name,
                });

                added += 1;
            }
        }

        for (id, _) in &fisql_map {
            if !fis_map.contains_key(id) {
                sqlite_conn.delete(id.into())?;

                on_progress(SyncEvent::FileRemoved { id: id.clone() });

                removed += 1;
            }
        }

        for (id, fs_file) in &fis_map {
            if let Some(sqlfile) = fisql_map.get(id) {
                let current_hash = fs_file.get_hash(storage)?.to_hex().to_string();

                if current_hash != sqlfile.hash {
                    sqlite_conn.update_hash(id.into(), current_hash)?;

                    if fs_file.extention.starts_with("text/") {
                        let tf = TFIDFCorpus::compute_tf(storage.data_folder().join(&id), 150)?;
                        sqlite_conn.reindex_file(&sqlfile, &tf)?;
                    }

                    on_progress(SyncEvent::FileUpdated { id: id.clone() });

                    updated += 1;
                }
            }
        }

        on_progress(SyncEvent::Done {
            added,
            removed,
            updated,
        });

        Ok(())
    }

    fn read_file_delimiter(
        &self,
        storage: &Storage,
        file_id: String,
        start_char: usize,
        end_char: usize,
    ) -> Result<String, StorageError> {
        let content = std::fs::read_to_string(storage.data_folder().join(file_id))?;
        let chars: Vec<char> = content.chars().collect();

        // expand start backwards to word boundary
        let mut real_start = start_char;
        while real_start > 0 && !chars[real_start].is_whitespace() {
            real_start -= 1;
        }

        // expand end forwards to word boundary
        let mut real_end = end_char.min(chars.len());
        while real_end < chars.len() && !chars[real_end].is_whitespace() {
            real_end += 1;
        }

        let text: String = chars[real_start..real_end].iter().collect();
        Ok(text.trim().to_string())
    }
}
