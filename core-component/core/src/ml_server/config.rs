use serde::{Deserialize, Serialize};

use crate::{CONFIG_DIR, ConfigError};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct MLConfig {
    pub host: String,
    pub port: u16,
    pub model: String,
    pub version: u32,
    pub dims: u32,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 50032,
            version: 1,
            model: "BAAI/bge-small-en-v1.5".to_string(),
            dims: 384,
        }
    }
}

impl MLConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let path = CONFIG_DIR
            .as_ref()
            .ok_or(ConfigError::NoDirsFound)?
            .join("ml.toml");

        if !path.exists() {
            return Ok(MLConfig::default());
        }

        let content = std::fs::read_to_string(path)?;

        Ok(toml::from_str(&content)?)
    }

    pub fn address(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}
