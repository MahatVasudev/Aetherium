pub mod error;
pub mod handler;
mod handlers;
pub mod ml_client;
pub mod types;

use aetherium_core::codex::Codex;
use std::path::PathBuf;

use crate::types::FileDetail;

pub struct Engine {
    codex: Option<Codex>,
}

pub enum EngineRequest {
    CreateCodex {
        path: PathBuf,
        codex_version: String,
        storage_version: String,
        sqlite_version: String,
    },

    OpenCodex {
        path: PathBuf,
    },

    AddFile {
        codex_path: PathBuf,
        file_path: PathBuf,
        file_name: Option<String>,
    },

    DeleteFile {
        codex_path: PathBuf,
        file_id: String,
    },

    ListFiles {
        codex_path: PathBuf,
    },

    GetConfig {
        codex_path: PathBuf,
        key: String,
    },

    SetConfig {
        codex_path: PathBuf,
        key: String,
        val: String,
    },

    Sync {
        codex_path: PathBuf,
    },

    MLHealth,
}

pub enum EngineResponse {
    CodexCreated {
        id: String,
        name: String,
    },
    CodexOpened {
        id: String,
        name: String,
    },
    FileAdded {
        file_id: String,
        hash: String,
    },
    FileDeleted,
    FileList {
        files: Vec<FileDetail>,
    },
    Synced,
    SearchResults,
    Clusters,
    GotConfig {
        value: Option<String>,
    },
    SettedConfig {
        key: String,
        val: String,
    },
    Error {
        message: String,
    },
    MLHealth {
        status: String,
        version: u32,
        model: String,
        dims: u32,
    },
    MLUnavailable(String),
}

pub enum EngineLiveRequest {
    Sync { codex_path: PathBuf },
}
