use std::collections::HashMap;

use crate::{
    storage::{
        error::SqliteError,
        sqlite::SqliteStore,
        sqlite_version::{
            SqliteStoreVersion,
            v1::types::{self, FileInSQL, SemanticSearchResult},
        },
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
}
