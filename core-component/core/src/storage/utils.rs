use std::{
    fs::{self, File},
    io::{self, BufReader, Read, Write},
    path::Path,
    time::SystemTime,
};

use blake3::Hash;
use chrono::Utc;
use infer::Infer;

use crate::{
    codex::codex_config::CodexConfig,
    storage::{CODEX_FILE, error::StorageError},
};

pub const UNKNOWN_EXTENSION: &str = "unknown-content";

/// write_to_file
/// returns anyhow::Result<tuple(blake3::Hash,String)>
/// writes file in the foldername given (filename) returns the finalized hash and hash as hex
pub fn write_to_file<R>(file: &mut File, data: R, byte: usize) -> Result<(Hash, String), io::Error>
where
    R: Read,
{
    let mut reader = BufReader::new(data);
    let mut buffer = vec![0; byte];
    let mut hasher = blake3::Hasher::new();
    let result = || -> Result<(), io::Error> {
        loop {
            let left = reader.read(&mut buffer)?;
            if left == 0 {
                break;
            }
            hasher.update(&buffer[..left]);
            file.write_all(&buffer[..left])?;
        }

        file.flush()?;
        file.sync_all()?;

        Ok(())
    }();

    result?;

    let file_hash = hasher.finalize();
    let file_id = file_hash.to_hex().to_string();
    Ok((file_hash, file_id))
}

pub fn read_codex_config<P: AsRef<Path>>(root_folder: P) -> Result<CodexConfig, io::Error> {
    let read_data = fs::read_to_string(root_folder.as_ref().join(CODEX_FILE))?;
    let data: CodexConfig = toml::from_str(read_data.as_str())?;

    Ok(data)
}

pub fn convert_datestring(date: std::time::SystemTime) -> String {
    let date_convert: chrono::DateTime<Utc> = date.into();

    date_convert.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn get_data_extension<P: AsRef<Path>>(file_path: P) -> Result<String, StorageError> {
    // get_data_extension will return the files extension
    // Will return an error if file is not found, or if there are permission issues
    // if no extension is found then UNKNOWN_EXTENSION will be returned
    let extension = Infer::new().get_from_path(&file_path);

    let mut mime: String;
    if let Some(ext) = &extension? {
        mime = ext.mime_type().into();
    } else {
        mime = match file_path.as_ref().extension().and_then(|e| e.to_str()) {
            Some("txt") => "text/plain".to_string(),
            Some("csv") => "text/csv".to_string(),
            Some("md") => "text/markdown".to_string(),
            Some("json") => "text/json".to_string(),
            _ => UNKNOWN_EXTENSION.to_string(),
        }
    }

    Ok(mime)
}
