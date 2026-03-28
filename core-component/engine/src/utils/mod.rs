use std::path::Path;

use aetherium_core::{
    codex::{Codex, codex_config::CodexConfig},
    storage::{error::StorageError, sqlite::SqliteStore, storage_types::SyncEvent},
    tfidf::{
        chunkreader::ChunkReader,
        embeddings::{Chunk, ChunkEmbedding},
        sentence_chunker::SentenceChunkerBatcher,
    },
};
use tracing::error;
use uuid::Uuid;

use crate::{
    error::EngineError,
    handlers::grpc_ops,
    ml_client::aetherium_ml::EmbedBatchResponse,
    types::{DocTextChunk, EngineEvent},
};

pub fn open_codex<P: AsRef<Path>>(codex_path: P) -> Result<Codex, EngineError> {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return Err(EngineError::CodexCouldNotOpen(e.message().to_string()));
        }
    };

    Ok(codex)
}

pub fn open_codex_and_config<P: AsRef<Path>>(
    codex_path: P,
) -> Result<(Codex, CodexConfig), EngineError> {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return Err(EngineError::CodexCouldNotOpen(e.message().to_string()));
        }
    };

    let config = match codex.read_config() {
        Ok(c) => c,
        Err(e) => return Err(EngineError::Corrupt(e.message().to_string())),
    };

    Ok((codex, config))
}

pub fn codex_sync_helper(
    codex: &Codex,
    on_progress: &(dyn Fn(EngineEvent) + Send + Sync),
) -> Result<Vec<String>, EngineError> {
    let mut file_to_embed: Vec<String> = Vec::new();
    let mut closure_error: Option<StorageError> = None;

    match codex.storage.sync(&mut |event| {
        match &event {
            SyncEvent::FileAdded { id, .. } => {
                file_to_embed.push(id.clone());
            }
            SyncEvent::FileRemoved { id } => {
                if let Err(e) = codex
                    .storage
                    .sqlite()
                    .and_then(|s| s.delete_chunks(id).map_err(StorageError::from))
                {
                    closure_error = Some(e)
                }
            }

            SyncEvent::FileUpdated { id } => {
                if let Err(e) = codex
                    .storage
                    .sqlite()
                    .and_then(|s| s.delete_chunks(id).map_err(StorageError::from))
                {
                    closure_error = Some(e)
                }

                file_to_embed.push(id.clone());
            }
            _ => {}
        }

        on_progress(EngineEvent::Sync(event.into()))
    }) {
        Ok(_) => {}
        Err(e) => return Err(EngineError::SyncFail(e.message().to_string())),
    }

    if let Some(e) = closure_error {
        return Err(EngineError::SyncFail(e.message().to_string()));
    };

    Ok(file_to_embed)
}

pub async fn embed_file_helper(
    file_id: String,
    codex: &Codex,
    sqlite_client: &SqliteStore,
    config: &CodexConfig,
) -> Result<(), EngineError> {
    let chunker = match ChunkReader::open(
        codex.storage.data_folder().join(&file_id),
        config.settings.read_chunk_size,
    ) {
        Ok(c) => c,
        Err(e) => return Err(EngineError::Corrupt(e.message().to_string())),
    };

    let batcher = SentenceChunkerBatcher::new(
        chunker,
        config.settings.embedding_batch_size,
        config.settings.embedding_max_token,
        config.settings.embedding_overlap,
    );

    for batch in batcher {
        match batch {
            Err(e) => return Err(EngineError::Corrupt(e.message().to_string())),
            Ok(ve) => {
                let doc_text: Vec<DocTextChunk> = ve
                    .into_iter()
                    .map(|c| DocTextChunk {
                        doc_id: file_id.clone(),
                        chunk_id: Uuid::new_v4().to_string(),
                        start_at: c.start_at,
                        end_at: c.end_at,
                        text: c
                            .chunks
                            .into_iter()
                            .map(|txt| txt.text.clone())
                            .collect::<Vec<String>>()
                            .join(" "),
                        file_index: c.index,
                    })
                    .collect();

                match grpc_ops::handler_ml_get_batch_embed(doc_text.clone()).await {
                    Ok(v) => {
                        sqlite_write_chunks_helper(sqlite_client, doc_text, v)?;
                    }
                    Err(e) => {
                        return Err(EngineError::Corrupt(e.message().to_string()));
                    }
                }
            }
        }
    }

    Ok(())
}

fn sqlite_write_chunks_helper(
    sqlite_client: &SqliteStore,
    doc_text: Vec<DocTextChunk>,
    embed_response: EmbedBatchResponse,
) -> Result<(), EngineError> {
    match sqlite_client.write_chunks(
        &doc_text
            .into_iter()
            .map(|f| Chunk {
                doc_id: f.doc_id,
                chunk_id: f.chunk_id,
                chunk_index: f.file_index,
                start_char: f.start_at,
                end_char: f.end_at,
            })
            .collect::<Vec<Chunk>>(),
    ) {
        Ok(_) => {}
        Err(e) => {
            return Err(EngineError::Corrupt(e.message().to_string()));
        }
    };

    match sqlite_client.write_embeddings(
        &embed_response
            .embeddings
            .into_iter()
            .map(|f| ChunkEmbedding {
                chunk_id: f.chunk_id,
                embedding: f.vector,
            })
            .collect::<Vec<ChunkEmbedding>>(),
    ) {
        Ok(_) => {}
        Err(e) => {
            return Err(EngineError::Corrupt(e.message().to_string()));
        }
    };

    Ok(())
}
