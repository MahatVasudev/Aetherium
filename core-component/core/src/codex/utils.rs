use anyhow::anyhow;
use std::{
    fs,
    io::{self},
    path::{Path, PathBuf},
};

use crate::{
    codex::versions::CodexVersion,
    storage::{DATA_FOLDER, DATABASE_FOLDER, INDEXED_FOLDER},
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
