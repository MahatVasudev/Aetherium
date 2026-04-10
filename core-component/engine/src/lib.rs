pub mod error;
pub mod handler;
mod handlers;
pub mod ml_client;
pub mod types;
pub mod utils;

use aetherium_core::codex::Codex;
use std::path::PathBuf;

use crate::types::{
    BasicClusterInfo, ClusterModelInfo, FileDetail, FileDetailWithCluster, SearchMatchedDetails,
};

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

    SearchFiles {
        codex_path: String,
        query: String,
        query_type: String,
        top_k: usize,
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

    Cluster {
        codex_path: PathBuf,
    },

    MLConfigSetup,
    MLHealth,
    ListFileWithClusters {
        codex_path: PathBuf,
    },

    ClusterStats {
        codex_path: PathBuf,
    },
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
    PartialSynced,
    SearchResults {
        results: Vec<(SearchMatchedDetails, String)>,
    },
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
        version: String,
        model: String,
        dims: u32,
    },
    Clustered {
        clusters_found: u32,
        unique_clusters: Vec<(String, i32)>,
    },

    ClusterStats {
        model_info: ClusterModelInfo,
        stats: Vec<BasicClusterInfo>,
    },

    MLUnavailable(String),
    MLConfigSetupSuccessful {
        path: String,
    },
    FileListWithClusters {
        files: Vec<FileDetailWithCluster>,
    },
}

pub enum EngineLiveRequest {
    Sync {
        codex_path: PathBuf,
    },

    Cluster {
        codex_path: PathBuf,
    },

    AddFile {
        codex_path: PathBuf,
        file_path: PathBuf,
        file_name: Option<String>,
    },
}

#[macro_export]
macro_rules! engine_err {
    ($e:expr) => {
        return EngineResponse::Error { message: $e.into() }
    };
}
