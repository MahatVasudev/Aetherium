use std::{default, fmt};

use serde::{Deserialize, Serialize};

pub const CONFIG_ID: &str = "id";
pub const CONFIG_NAME: &str = "name";
pub const CONFIG_CODEX_VERSION: &str = "codex";
pub const CONFIG_STORAGE_VERSION: &str = "storage";
pub const CONFIG_SQLITESTORE_VERSION: &str = "sqlitestore";
pub const CONFIG_CREATED_AT: &str = "created_at";
pub const CONFIG_READ_CHUNK_SIZE: &str = "read_chunk_size";
pub const CONFIG_WRITE_CHUNK_SIZE: &str = "write_chunk_size";
pub const DEFAULT_ML_MODEL: &str = "all-MiniLM-L6-v2";
pub const CONFIG_ML_MODEL: &str = "ml-model";
pub const CONFIG_ML_DIMS: &str = "dims";
pub const DEFAULT_ML_DIMS: u32 = 384;

pub fn get_codex_config_template(config: &CodexConfigWrite) -> String {
    let codex_content = format!(
        "[identity]
{CONFIG_ID}=\"{}\"
{CONFIG_NAME}=\"{}\"
[version]
{CONFIG_CODEX_VERSION}=\"{}\"
{CONFIG_STORAGE_VERSION}=\"{}\"
{CONFIG_SQLITESTORE_VERSION}=\"{}\"
{CONFIG_CREATED_AT}=\"{}\"
[settings]
{CONFIG_READ_CHUNK_SIZE}={}
{CONFIG_WRITE_CHUNK_SIZE}={}
",
        config.identity_id,
        config.identity_name,
        config.version_codex,
        config.version_storage,
        config.version_sqlitestore,
        config.version_created_at,
        config.settings_write_chunk_size,
        config.settings_read_chunk_size
    );

    codex_content
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CodexConfig {
    pub identity: Identity,
    pub version: Version,
    pub settings: Settings,
    #[serde(default)]
    pub ml: MLSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Identity {
    pub id: String,
    pub name: String,
}

pub enum ConfigValue {
    Str(String),
    UINT(usize),
    INT(isize),
    BOOL(bool),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Version {
    pub codex: String,
    pub storage: String,
    pub sqlitestore: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
    pub read_chunk_size: usize,
    pub write_chunk_size: usize,
    pub embedding_batch_size: usize,
    pub embedding_max_token: usize,
    pub embedding_overlap: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct MLSettings {
    pub model_name: String,
    pub dims: u32,
}

impl Default for MLSettings {
    fn default() -> Self {
        Self {
            model_name: DEFAULT_ML_MODEL.to_string(),
            dims: DEFAULT_ML_DIMS,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            read_chunk_size: 512,
            write_chunk_size: 512,
            embedding_batch_size: 4,
            embedding_max_token: 512,
            embedding_overlap: 2,
        }
    }
}

pub struct CodexConfigWrite {
    pub identity_id: String,
    pub identity_name: String,
    pub version_codex: String,
    pub version_storage: String,
    pub version_sqlitestore: String,
    pub version_created_at: String,
    pub settings_read_chunk_size: usize,
    pub settings_write_chunk_size: usize,
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::Str(v) => write!(f, "{}", v),
            ConfigValue::INT(v) => write!(f, "{}", v),
            ConfigValue::UINT(v) => write!(f, "{}", v),
            ConfigValue::BOOL(v) => write!(f, "{}", v),
        }
    }
}
