use anyhow::anyhow;
use blake3::Hash;
use std::{
    fs,
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use crate::codex::{DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER, versions::CodexVersion};

pub fn version_error(version: &str) -> anyhow::Result<CodexVersion> {
    match CodexVersion::parse(version) {
        Some(version) => Ok(version),
        None => return Err(anyhow!("version not found {:?}", version)),
    }
}

pub fn create_temp_if_not_exists(root_folder: &PathBuf) -> anyhow::Result<PathBuf> {
    let temp_folder = root_folder.join("tmp");
    if let Err(e) = fs::create_dir(&temp_folder) {
        match e.kind() != io::ErrorKind::AlreadyExists {
            true => return Err(e.into()),
            false => (),
        }
    };

    Ok(temp_folder)
}

pub fn make_all_codex_dirs(root_folder: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(root_folder.join(DATA_FOLDER))?;
    fs::create_dir_all(root_folder.join(INDEXED_FOLDER))?;
    fs::create_dir_all(root_folder.join(DATABASE_FOLDER))?;
    Ok(())
}

/// write_to_file
/// returns anyhow::Result<tuple(blake3::Hash,String)>
/// writes file in the foldername given (filename) returns the finalized hash and hash as hex
pub fn write_to_file<R>(filename: &PathBuf, data: R, byte: usize) -> anyhow::Result<(Hash, String)>
where
    R: Read,
{
    let mut reader = BufReader::new(data);
    let mut buffer = vec![0; byte];
    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(filename)?;
    let mut hasher = blake3::Hasher::new();
    let result = || -> anyhow::Result<()> {
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
