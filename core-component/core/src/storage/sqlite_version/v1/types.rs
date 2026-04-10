pub struct FileInSQL {
    pub id: String,
    pub name: String,
    pub hash: String,
    pub extension: String,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub indexed_at: Option<String>,
    pub embedded_at: Option<String>,
}

pub struct FileDetailWithTopCluster {
    pub id: String,
    pub name: String,
    pub extension: String,
    pub created_at: Option<String>,
    pub cluster_name: Option<String>,
    pub top_cluster_pct: Option<f64>,
}

pub struct ClusterFile {
    pub file_id: String,
    pub file_name: String,
    pub chunk_count: usize,
    pub cluster_match: f32,
}

pub struct Info {
    codex_version: String,
    storage_version: String,
    sqlite_version: String,
}

pub struct SemanticSearchResult {
    pub chunk_id: String,
    pub doc_id: String,
    pub distance: f32,
    pub start_char: usize,
    pub end_char: usize,
    pub file_name: String,
    pub cluster: Option<String>,
}

pub struct TriggerTables {
    pub table_name: String,
    pub col: String,
}

pub struct ClusteredDocs {
    pub cluster_id: i32,
    pub cluster_name: i32,
    pub percentage: f32,
}

pub struct ChunksSql {
    pub id: String,
    pub doc_id: String,
    pub start_char: usize,
    pub end_char: usize,
}

pub struct ChunkEmbeddingSql {
    pub chunk_id: String,
    pub doc_id: String,
    pub embedding: Vec<f32>,
}
