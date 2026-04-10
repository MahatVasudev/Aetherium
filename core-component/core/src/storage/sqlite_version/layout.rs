use std::collections::HashMap;

use crate::{
    storage::{
        error::SqliteError,
        sqlite::SqliteStore,
        sqlite_version::{
            SqliteStoreVersion,
            v1::types::{
                self, ChunkEmbeddingSql, ChunksSql, ClusterFile, ClusteredDocs,
                FileDetailWithTopCluster, FileInSQL, SemanticSearchResult,
            },
        },
        storage_types::BasicClusterInfo,
    },
    tfidf::embeddings::{Chunk, ChunkEmbedding},
};

pub trait SqliteLayout: Send + Sync {
    fn version(&self) -> SqliteStoreVersion;

    fn create_base(&self, sqlite_store: &SqliteStore) -> Result<(), SqliteError>;

    fn delete(&self, sqlite_store: &SqliteStore, file: String) -> Result<(), SqliteError>;

    fn add_metadata(
        &self,
        sqlite_store: &SqliteStore,
        file: &types::FileInSQL,
    ) -> Result<(), SqliteError>;

    fn get_all_files(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<types::FileInSQL>, SqliteError>;

    fn update_hash(
        &self,
        sqlite_store: &SqliteStore,
        id: String,
        hash: String,
    ) -> Result<(), SqliteError>;

    fn reindex_file(
        &self,
        sqlite_store: &SqliteStore,
        file: &FileInSQL,
        term_frequencies: &HashMap<String, usize>,
    ) -> Result<(), SqliteError>;

    fn write_embeddings(
        &self,
        sqlite_store: &SqliteStore,
        chunk_embedding: &[ChunkEmbedding],
    ) -> Result<(), SqliteError>;

    fn write_chunktext(
        &self,
        sqlite_store: &SqliteStore,
        chunk: &[Chunk],
    ) -> Result<(), SqliteError>;

    fn delete_chunk(&self, sqlite_store: &SqliteStore, docid: String) -> Result<(), SqliteError>;

    fn find_similar_files_embedding(
        &self,
        sqlite_store: &SqliteStore,
        query: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SemanticSearchResult>, SqliteError>;

    fn list_embeded_files(&self, sqlite_store: &SqliteStore)
    -> Result<Vec<FileInSQL>, SqliteError>;

    fn list_not_embeded_files(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<types::FileInSQL>, SqliteError>;

    fn get_doc_cluster(
        &self,
        sqlite_store: &SqliteStore,
        doc_id: &str,
    ) -> Result<Vec<ClusteredDocs>, SqliteError>;

    fn write_cluster_info(
        &self,
        sqlite_store: &SqliteStore,
        cluster_id: i32,
        name: &str,
    ) -> Result<(), SqliteError>;

    fn write_cluster_chunks(
        &self,
        sqlite_store: &SqliteStore,
        assignments: &[(String, i32)],
    ) -> Result<(), SqliteError>;

    fn get_tfidf_chunks(
        &self,
        sqlite_store: &SqliteStore,
        chunk_id: &str,
    ) -> Result<Vec<(String, String, Vec<f32>)>, SqliteError>;

    fn get_all_chunks(&self, sqlite_store: &SqliteStore) -> Result<Vec<ChunksSql>, SqliteError>;

    fn get_all_embeddings(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<ChunkEmbeddingSql>, SqliteError>;

    fn clear_clusters(&self, sqlite_store: &SqliteStore) -> Result<(), SqliteError>;

    fn list_files_with_top_clusters(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<FileDetailWithTopCluster>, SqliteError>;

    fn get_cluster_files(
        &self,
        sqlite_store: &SqliteStore,
        cluster_id: i32,
    ) -> Result<Vec<ClusterFile>, SqliteError>;

    fn get_embeds_dim(&self, sqlite_store: &SqliteStore) -> Result<u32, SqliteError>;
    fn reset_embedding_tables(
        &self,
        sqlite_store: &SqliteStore,
        dims: u32,
    ) -> Result<(), SqliteError>;
    fn get_basic_cluster_info(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<BasicClusterInfo>, SqliteError>;
}
