use std::{
    env,
    fmt::Display,
    path::{self, Path},
    sync::LazyLock,
};

pub mod cacher;
pub mod codex;
pub mod metadata;
pub mod ml_server;
pub mod storage;
pub mod tfidf;

pub static CURRENT_DIR: LazyLock<Option<std::path::PathBuf>> = LazyLock::new(|| {
    let pare_path = env::current_dir().ok();
    path::absolute(&pare_path?).ok()
});

pub static CONFIG_DIR: LazyLock<Option<std::path::PathBuf>> =
    LazyLock::new(|| dirs::config_dir().map(|p| p.join("aetherium")));

#[derive(Debug)]
pub enum ConfigError {
    NoDirsFound,
    Corrupt(String),
}

impl ConfigError {
    pub fn message(&self) -> String {
        match self {
            Self::NoDirsFound => "config dir not found".to_string(),
            Self::Corrupt(v) => format!("codex is corrupted: {v}"),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::NotFound => Self::NoDirsFound,
            _ => Self::Corrupt(value.to_string()),
        }
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::Corrupt(value.to_string())
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Config Error {}", self.message())
    }
}

impl std::error::Error for ConfigError {}
