use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::storage::{Storage, error::StorageError, utils};

pub struct FileInSystem {
    pub id: String,
    pub extention: String,
    pub modified_at: String,
}

impl FileInSystem {
    pub fn from(storage: &Storage, file_id: String) -> Result<Self, StorageError> {
        let file_name = storage.data_folder().join(&file_id);

        if !file_name.is_file() {
            return Err(StorageError::NotFound(file_id));
        }

        let extensions = file_name
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_string();

        let modified_at = utils::convert_datestring(file_name.metadata()?.modified()?);

        Ok(Self {
            id: file_id,
            extention: extensions,
            modified_at,
        })
    }
    pub fn get_hash(&self, storage: &Storage) -> Result<blake3::Hash, StorageError> {
        let file_name = storage.data_folder().join(&self.id);

        if !file_name.is_file() {
            return Err(StorageError::NotFound(self.id.clone()));
        }

        let file = File::open(file_name)?;
        let buffer_size: usize = 512;
        let mut buffer = vec![0; buffer_size];
        let mut reader = BufReader::new(file);
        let mut hasher = blake3::Hasher::new();

        let result = || -> Result<(), StorageError> {
            loop {
                let left = reader.read(&mut buffer)?;
                if left == 0 {
                    break;
                }

                hasher.update(&buffer[..left]);
            }

            Ok(())
        }();

        result?;

        let file_hash = hasher.finalize();

        Ok(file_hash)
    }
}
