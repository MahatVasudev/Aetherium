use serde::Deserialize;

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
    pub codex: String,
    pub storage: String,
    pub created_at: String,
}
