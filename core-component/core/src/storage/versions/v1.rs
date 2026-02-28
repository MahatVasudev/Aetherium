use uuid::Uuid;

use crate::{
    codex::codex_config::CodexConfig,
    storage::{
        CODEX_FILE, DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER, Storage,
        error::StorageError,
        sqlite_version::v1::types::Files,
        storage_types::{self, FileInSystem},
        utils,
        versions::{StorageVersion, layout::StorageLayout},
    },
    storage_assert,
};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, ErrorKind},
    path::PathBuf,
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
                    return Err(StorageError::Io(error.kind()));
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
                return Err(StorageError::Io(err.kind()));
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

        return true;
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
        fs::rename(
            tmpfolder.join(&filename),
            storage.data_folder.join(&filename),
        )?;

        let modified_at =
            utils::convert_datestring(storage.data_folder.join(&filename).metadata()?.modified()?);
        return Ok(FileInSystem {
            id: filename,
            extention: "".into(),
            modified_at,
        });
    }

    fn create_new_codex_file(&self, storage: &Storage, content: &str) -> Result<(), StorageError> {
        fs::write(&storage.root_folder.join(CODEX_FILE), content)?;

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

        let file_path = storage.data_folder.join(file_id);

        if !file_path.is_file() {
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

            let extention = path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_string();

            let modified_at = utils::convert_datestring(path.metadata()?.modified()?);

            files.push(FileInSystem {
                id: file_name,
                extention,
                modified_at,
            })
        }

        Ok(files)
    }

    fn sync(&self, storage: &Storage) -> Result<(), StorageError> {
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
        let fisql = storage.sqlite()?.get_all_files()?;

        let fis_map: HashMap<String, FileInSystem> =
            fis.into_iter().map(|f| (f.id.clone(), f)).collect();
        let fisql_map: HashMap<String, Files> =
            fisql.into_iter().map(|f| (f.id.clone(), f)).collect();

        for (id, fs_file) in &fis_map {
            if !fisql_map.contains_key(id) {
                storage.sqlite()?.add_metadata(Files {
                    id: id.into(),
                    name: "some name".into(),
                    hash: fs_file.get_hash(&storage)?.to_hex().to_string(),
                    extension: fs_file.extention.clone(),
                    created_at: None,
                    modified_at: None,
                })?;
            }
        }

        for (id, _) in &fisql_map {
            if !fisql_map.contains_key(id) {
                storage.sqlite()?.delete(id.into())?;
            }
        }

        for (id, fs_file) in &fis_map {
            if let Some(sqlfile) = fisql_map.get(id) {
                let current_hash = fs_file.get_hash(storage)?.to_hex().to_string();

                if current_hash != sqlfile.hash {
                    storage.sqlite()?.update_hash(id.into(), current_hash)?;
                }
            }
        }

        Ok(())
    }
}
