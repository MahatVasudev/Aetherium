pub mod types;
pub mod utils;
use std::result;

use crate::{
    ml_server::config::MLConfig,
    storage::{
        error::SqliteError,
        sqlite::SqliteStore,
        sqlite_version::{
            layout::SqliteLayout,
            v1::types::{SemanticSearchResult, TriggerTables},
        },
    },
    tfidf::embeddings::{Chunk, ChunkEmbedding},
};
use uuid::Uuid;
use zerocopy::IntoBytes;

pub struct SQLITESTOREV1;

impl SqliteLayout for SQLITESTOREV1 {
    fn version(&self) -> super::SqliteStoreVersion {
        super::SqliteStoreVersion::V1
    }

    fn update_hash(
        &self,
        sqlite_store: &SqliteStore,
        id: String,
        hash: String,
    ) -> Result<(), SqliteError> {
        let mut codex_conn = sqlite_store.codex_conn.lock()?;

        let codex_txn = codex_conn.transaction()?;

        codex_txn.execute("update files set hash = ?2 where id = ?1", (id, hash))?;

        codex_txn.commit()?;

        Ok(())
    }

    fn delete(&self, sqlite_store: &SqliteStore, fileid: String) -> Result<(), SqliteError> {
        let codex_conn = sqlite_store.codex_conn.lock()?;

        codex_conn.execute(
            "UPDATE keywords SET doc_count = doc_count - 1 WHERE id IN (
            SELECT keyword_id FROM files_keywords WHERE file_id = ?1
        )",
            rusqlite::params![fileid],
        )?;
        codex_conn.execute("DELETE FROM keywords WHERE doc_count <= 0", [])?;
        codex_conn.execute(
            "DELETE FROM files_keywords WHERE file_id = ?1",
            rusqlite::params![fileid],
        )?;
        codex_conn.execute("DELETE FROM files WHERE id = ?1", rusqlite::params![fileid])?;
        Ok(())
    }

    fn get_all_files(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<types::FileInSQL>, SqliteError> {
        // FIXME: Add a batch retrieval, so that it is scalable
        let codex_conn = sqlite_store.codex_conn.lock()?;
        println!("got error after starting sqlite");
        let mut query = codex_conn.prepare(
            "
            select 
                id, 
                name, 
                hash,
                extensions, 
                created_at, 
                modified_at,
                indexed_at,
                embedded_at
            from files;",
        )?;

        println!("got error after starting sqlite query creation");
        let files = query
            .query_map((), |row| {
                Ok(types::FileInSQL {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    hash: row.get(2)?,
                    extension: row.get(3)?,
                    created_at: Some(row.get(4)?),
                    modified_at: Some(row.get(5)?),
                    indexed_at: None,
                    embedded_at: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(files)
    }

    fn delete_chunk(&self, sqlite_store: &SqliteStore, docid: String) -> Result<(), SqliteError> {
        let mut codex_conn = sqlite_store.codex_conn.lock()?;
        let codex_txn = codex_conn.transaction()?;

        codex_txn.execute(
            "delete from chunk_embeddings where chunk_id in (
                select id in chunks where doc_id = ?1
        )",
            [&docid],
        )?;

        codex_txn.execute(
            "
        delete from chunks where doc_id = ?1;
            ",
            [docid],
        )?;

        codex_txn.commit()?;

        Ok(())
    }

    fn find_similar_files_embedding(
        &self,
        sqlite_store: &SqliteStore,
        query_vector: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SemanticSearchResult>, SqliteError> {
        use zerocopy::IntoBytes;
        let conn = sqlite_store.codex_conn.lock()?;

        let mut query = conn.prepare(
            "
SELECT
    ce.chunk_id,
    ce.distance,
    c.doc_id,
    c.start_char,
    c.end_char,
    f.name
FROM (
    SELECT chunk_id, distance
    FROM chunk_embeddings
    WHERE embedding MATCH ?
    ORDER BY distance
    LIMIT ?
) ce
JOIN chunks c ON ce.chunk_id = c.id
JOIN files f ON c.doc_id = f.id
            ",
        )?;

        let result = query
            .query_map(
                rusqlite::params![query_vector.as_bytes(), top_k as i64],
                |f| {
                    Ok(SemanticSearchResult {
                        chunk_id: f.get(0)?,
                        distance: f.get(1)?,
                        doc_id: f.get(2)?,
                        start_char: f.get::<_, i64>(3)? as usize,
                        end_char: f.get::<_, i64>(4)? as usize,
                        file_name: f.get(5)?,
                    })
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(result)
    }

    fn add_metadata(
        &self,
        sqlite_store: &SqliteStore,
        file: &types::FileInSQL,
    ) -> Result<(), SqliteError> {
        let mut codex_conn = sqlite_store.codex_conn.lock()?;
        let codex_txn = codex_conn.transaction()?;

        codex_txn.execute(
            "
        insert into files (id, name, extensions, hash, indexed_at, embedded_at)
        values (?1,?2,?3,?4,?5,?6);",
            (
                file.id.clone(),
                file.name.clone(),
                file.extension.clone(),
                file.hash.clone(),
                file.indexed_at.clone(),
                file.embedded_at.clone(),
            ),
        )?;

        codex_txn.commit()?;
        Ok(())
    }

    fn create_base(&self, sqlite_store: &SqliteStore) -> Result<(), SqliteError> {
        // FIX: These are not final stage, we need to think of a concrete database schema
        // For Both codex db and cache db, they should be practical, and be optimized with low
        // powered system...
        //
        // FIX: We have to distribute the commands into different method/functions
        // Have the same order
        //  - Tables
        //  - Indexes
        //  - Set triggers to tables
        // All of the commands should follow a single transaction (1 for codex, 1 for cache)
        // If any one of the process fail, we rollback and none of the
        // (even for codex or cache should not be saved, if either one of them fails)
        // These should be in individual blocks for codex db and cache db

        let mut codex_conn = sqlite_store.codex_conn.lock()?;

        let mut cache_conn = sqlite_store.cache_conn.lock()?;

        let config = match MLConfig::load() {
            Ok(c) => c,
            Err(e) => return Err(SqliteError::Corrupt(e.message())),
        };
        let codex_updates = vec![
            TriggerTables {
                table_name: "files".into(),
                col: "modified_at".into(),
            },
            TriggerTables {
                table_name: "info".into(),
                col: "updated_at".into(),
            },
        ];

        let cache_updates = vec![TriggerTables {
            table_name: "content_cache".into(),
            col: "cached_at".into(),
        }];
        let codex_txn = codex_conn.transaction()?;
        let cache_txn = cache_conn.transaction()?;

        // NOTE: Create tables for codex db
        utils::initialize_tables_codex(&codex_txn)?;
        utils::initialize_tables_embeddings(&codex_txn, &config.dims)?;
        // NOTE: Create tables for cache db
        utils::initialize_tables_cache(&cache_txn)?;

        // FIXME: Implement Indexes Later
        //

        // NOTE: Map tables triggers
        utils::initialize_updated_at_triggers(&codex_txn, codex_updates)?;
        utils::initialize_updated_at_triggers(&cache_txn, cache_updates)?;

        match (cache_txn.commit(), codex_txn.commit()) {
            (Ok(_), Ok(_)) => Ok(()),
            _ => Err(SqliteError::AssertionFail("failed to commit".into())),
        }
    }

    fn write_chunktext(
        &self,
        sqlite_store: &SqliteStore,
        chunk: &[Chunk],
    ) -> Result<(), SqliteError> {
        let mut conn = sqlite_store.codex_conn.lock()?;

        let txn = conn.transaction()?;

        for c in chunk {
            txn.execute(
                "insert into chunks (id, doc_id, chunk_index, start_char, end_char)
            values (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    c.chunk_id,
                    c.doc_id,
                    c.chunk_index as i64,
                    c.start_char as i64,
                    c.end_char as i64,
                ],
            )?;
        }

        txn.commit()?;
        Ok(())
    }

    fn write_embeddings(
        &self,
        sqlite_store: &SqliteStore,
        chunk_embedding: &[ChunkEmbedding],
    ) -> Result<(), SqliteError> {
        let mut conn = sqlite_store.codex_conn.lock()?;

        let txn = conn.transaction()?;

        for emb in chunk_embedding {
            txn.execute(
                "
        insert into chunk_embeddings (chunk_id, embedding)
        values (?1, ?2)
            ",
                rusqlite::params![emb.chunk_id, emb.embedding.as_bytes()],
            )?;
        }

        txn.commit()?;

        Ok(())
    }
    fn reindex_file(
        &self,
        sqlite_store: &SqliteStore,
        file: &types::FileInSQL,
        term_frequencies: &std::collections::HashMap<String, usize>,
    ) -> Result<(), SqliteError> {
        let mut conn = sqlite_store.codex_conn.lock()?;
        let txn = conn.transaction()?;

        txn.execute(
            "
        update keywords set doc_count = doc_count - 1
        where id in (
          select id from files_keywords where id = ?1
        )
            ",
            [&file.id],
        )?;

        txn.execute("delete from keywords where doc_count <= 0", [])?;

        for (term, &count) in term_frequencies {
            txn.execute(
                "insert into keywords (id, name, doc_count)
                values (?1, ?2, 1)
                on conflict(name) do update set doc_count = doc_count + 1",
                [Uuid::new_v4().to_string(), term.clone()],
            )?;

            let keyword_id: String =
                txn.query_row("select id from keywords where name = ?1", [term], |f| {
                    f.get(0)
                })?;

            txn.execute(
                "INSERT OR REPLACE INTO files_keywords (file_id, keyword_id, frequency)
             VALUES (?1, ?2, ?3)",
                (&file.id, &keyword_id, count as i64),
            )?;
        }

        txn.execute(
            "UPDATE files SET indexed_at = current_timestamp WHERE id = ?1",
            [&file.id],
        )?;
        txn.commit()?;

        Ok(())
    }
}
