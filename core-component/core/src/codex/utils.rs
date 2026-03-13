use anyhow::anyhow;
use std::{
    fs,
    io::{self},
    path::{Path, PathBuf},
};

use crate::{
    codex::versions::CodexVersion,
    storage::{CODEX_FILE, DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER, error::StorageError},
};

pub fn version_error(version: &str) -> anyhow::Result<CodexVersion> {
    match CodexVersion::parse(version) {
        Some(version) => Ok(version),
        None => return Err(anyhow!("version not found {:?}", version)),
    }
}

pub fn create_temp_if_not_exists(root_folder: &PathBuf) -> anyhow::Result<PathBuf> {
    let temp_folder = root_folder.join("tmp");
    if let Err(e) = fs::create_dir(&temp_folder) {
        if e.kind() != io::ErrorKind::AlreadyExists {
            return Err(e.into());
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

pub fn write_codex_config(root: &Path, content: &str) -> Result<(), StorageError> {
    let path = root.join(CODEX_FILE);

    // lift readonly
    let mut perms = fs::metadata(&path)?.permissions();
    perms.set_readonly(false);
    fs::set_permissions(&path, perms.clone())?;

    let result = fs::write(&path, content);

    // always restore readonly, even if write failed
    perms.set_readonly(true);
    fs::set_permissions(&path, perms)?;

    result?;
    Ok(())
}
