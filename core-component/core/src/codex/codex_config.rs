use std::{fs, path::Path};

use serde::Deserialize;

use crate::codex::CODEX_FILE;

#[derive(Debug, Deserialize)]
pub struct CodexConfig {
    pub identity: Identity,
    pub version: Version,
}

#[derive(Debug, Deserialize)]
pub struct Identity {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Version {
    pub version: String,
    pub created_at: String,
}

pub fn read_codex_config(root_folder: &Path) -> anyhow::Result<CodexConfig> {
    let read_data = fs::read_to_string(root_folder.join(CODEX_FILE))?;
    let data: CodexConfig = toml::from_str(read_data.as_str())?;

    Ok(data)
}
