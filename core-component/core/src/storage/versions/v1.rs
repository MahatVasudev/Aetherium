use blake3::Hash;
use uuid::Uuid;

use crate::{
    codex::codex_config::CodexConfig,
    storage::{
        CODEX_FILE, DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER, Storage,
        error::StorageError,
        utils,
        versions::{StorageVersion, layout::StorageLayout},
    },
    storage_assert,
};
use std::{
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
    ) -> Result<(Hash, String), StorageError> {
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

        let (file_hash, file_id) = result?;
        fs::rename(
            tmpfolder.join(&filename),
            storage.data_folder.join(&file_id),
        )?;

        return Ok((file_hash, file_id));
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

    fn list_files(&self, storage: &Storage, query: &str) -> Result<Vec<String>, StorageError> {
        // Go to Data Location
        // find all of the files if the query is empty, else return the list of files that satisfy
        // under those query
        // WARN: The above specified implementation for query is only when data is saved sqlite, for now
        // return all of the files list, with only file id
        // TODO: Afer Implementing SQLITE Management, we will return a list of a struct which
        // should contain file id, name, extension, date added
        let entries = fs::read_dir(&storage.data_folder)?;

        let files: Vec<String> = entries
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();

                if path.is_file() {
                    path.file_name()
                        .map(|name| name.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(files)
    }
}
