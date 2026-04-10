use aetherium_core::storage::{
    self, sqlite_version::v1::types::FileInSQL, storage_types::SyncEvent,
};

use crate::error::EngineError;

pub struct FileDetail {
    pub id: String,
    pub name: String,
    pub hash: String,
    pub extension: String,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
}

pub struct ClusterModelInfo {
    pub name: String,
    pub dimension_reduction_model: String,
    pub dims: usize,
    pub reduced_to: usize,
}

pub struct BasicClusterInfo {
    pub id: i64,
    pub name: String,
    pub top_files: Vec<String>,
    pub chunk_count: usize,
    pub file_count: usize,
    pub created_at: String,
}

pub struct FileDetailWithCluster {
    pub id: String,
    pub name: String,
    pub extension: String,
    pub created_at: Option<String>,
    pub cluster_name: Option<String>,
    pub top_cluster_pct: Option<f64>,
}

pub struct SearchMatchedDetails {
    pub file_id: String,
    pub chunk_id: String,
    pub file_name: String,
    pub cluster: Option<String>,
    pub distance: f32,
}

pub enum SearchType {
    Lexical,
    Semantic,
    Mix,
}

pub enum EngineEvent {
    Sync(SyncProgress),
    OperationStarted,
    OperationFinished,
    Clustering,
    MLUnavailable,
}

pub enum SyncProgress {
    FileAdded {
        id: String,
        name: String,
    },
    FileRemoved {
        id: String,
    },
    FileUpdated {
        id: String,
    },
    Done {
        added: usize,
        removed: usize,
        updated: usize,
    },

    EmbeddingPending {
        file_id: String,
    },

    Embedding {
        file_id: String,
    },

    DimsMISMATCH {
        previous: u32,
        proposed: u32,
    },
    DimsChanged {
        previous: u32,
        now: u32,
    },
}

pub struct ClusterChunkInput {
    pub chunk_id: String,
    pub doc_id: String,
    pub vector: VectorInput,
}

pub enum VectorInput {
    Embedding(Vec<f32>),
    TFIDF(Vec<f32>),
}

#[derive(Clone)]
pub struct DocTextChunk {
    pub doc_id: String,
    pub chunk_id: String,
    pub text: String,
    pub file_index: usize,
    pub start_at: usize,
    pub end_at: usize,
}

impl From<SyncEvent> for SyncProgress {
    fn from(value: SyncEvent) -> Self {
        match value {
            SyncEvent::FileAdded { id, name } => SyncProgress::FileAdded {
                id: id.to_string(),
                name: name.to_string(),
            },
            SyncEvent::FileRemoved { id } => SyncProgress::FileRemoved { id: id.to_string() },
            SyncEvent::FileUpdated { id } => SyncProgress::FileUpdated { id: id.to_string() },
            SyncEvent::Done {
                added,
                removed,
                updated,
            } => SyncProgress::Done {
                added,
                removed,
                updated,
            },

            SyncEvent::FileEmbeddingPending { id } => SyncProgress::EmbeddingPending {
                file_id: id.to_string(),
            },

            SyncEvent::DimsMISMATCH { previous, proposed } => {
                SyncProgress::DimsMISMATCH { previous, proposed }
            }
            SyncEvent::DIMSChanged { previous, now } => SyncProgress::DimsChanged { previous, now },
        }
    }
}

impl SearchType {
    pub fn parse_str(string: &str) -> Result<Self, EngineError> {
        match string.to_lowercase().as_str() {
            "lexical" | "l" | "literal" => Ok(Self::Lexical),
            "semantic" | "s" | "dynamic" | "question" => Ok(Self::Semantic),
            "mix" | "a" | "all" => Ok(Self::Mix),
            _ => Err(EngineError::SearchModeNotFound(string.to_string())),
        }
    }

    pub fn as_str(&self) -> String {
        match self {
            Self::Lexical => "lexical",
            Self::Semantic => "semantic",
            Self::Mix => "mix",
        }
        .to_string()
    }
}

impl From<&FileInSQL> for FileDetail {
    fn from(value: &FileInSQL) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            hash: value.hash.clone(),
            extension: value.extension.clone(),
            created_at: value.created_at.clone(),
            modified_at: value.modified_at.clone(),
        }
    }
}

impl From<storage::storage_types::BasicClusterInfo> for BasicClusterInfo {
    fn from(value: storage::storage_types::BasicClusterInfo) -> Self {
        Self {
            id: value.id,
            name: value.name,
            created_at: value.created_at,
            top_files: value.top_files,
            chunk_count: value.chunk_count,
            file_count: value.file_count,
        }
    }
}
