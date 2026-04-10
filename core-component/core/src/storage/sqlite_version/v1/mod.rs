pub mod types;
pub mod utils;
use std::{
    collections::{HashMap, HashSet},
    result,
};

use crate::{
    ml_server::config::MLConfig,
    storage::{
        error::SqliteError,
        sqlite::SqliteStore,
        sqlite_version::{
            layout::SqliteLayout,
            v1::types::{
                ChunkEmbeddingSql, ChunksSql, ClusterFile, FileDetailWithTopCluster,
                SemanticSearchResult, TriggerTables,
            },
        },
        storage_types,
    },
    tfidf::embeddings::{Chunk, ChunkEmbedding},
};
use uuid::Uuid;
use zerocopy::IntoBytes;

use zerocopy;

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

        let files = query
            .query_map((), |row| {
                Ok(types::FileInSQL {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    hash: row.get(2)?,
                    extension: row.get(3)?,
                    created_at: Some(row.get(4)?),
                    modified_at: Some(row.get(5)?),
                    indexed_at: row.get(6)?, // FIXME: HOT FIX, Need to Change
                    embedded_at: row.get(7)?, // FIXME: HOT FIX, Need to Change
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
                select id from chunks where doc_id = ?1
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

    fn list_embeded_files(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<types::FileInSQL>, SqliteError> {
        let conn = sqlite_store.codex_conn.lock()?;
        let mut query = conn.prepare(
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
            from files where id in (SELECT DISTINCT doc_id from chunks)
            ",
        )?;

        let files = query
            .query_map([], |row| {
                Ok(types::FileInSQL {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    hash: row.get(2)?,
                    extension: row.get(3)?,
                    created_at: None,  // FIXME: HOT FIX, Need to Change
                    modified_at: None, // FIXME: HOT FIX, Need to Change
                    indexed_at: None,  // FIXME: HOT FIX, Need to Change
                    embedded_at: None, // FIXME: HOT FIX, Need to Change
                })
            })?
            .collect::<Result<Vec<types::FileInSQL>, _>>();
        Ok(files?)
    }

    fn list_not_embeded_files(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<types::FileInSQL>, SqliteError> {
        let conn = sqlite_store.codex_conn.lock()?;
        let mut query = conn.prepare(
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
            from files where id not in (SELECT DISTINCT doc_id from chunks)
            ",
        )?;

        let files = query
            .query_map([], |row| {
                Ok(types::FileInSQL {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    hash: row.get(2)?,
                    extension: row.get(3)?,
                    created_at: None,  // FIXME: HOT FIX, Need to Change
                    modified_at: None, // FIXME: HOT FIX, Need to Change
                    indexed_at: None,  // FIXME: HOT FIX, Need to Change
                    embedded_at: None, // FIXME: HOT FIX, Need to Change
                })
            })?
            .collect::<Result<Vec<types::FileInSQL>, _>>();
        Ok(files?)
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
    f.name,
    ct.name
FROM (
    SELECT chunk_id, distance
    FROM chunk_embeddings
    WHERE embedding MATCH ?
    ORDER BY distance
    LIMIT ?
) ce
JOIN chunks c ON ce.chunk_id = c.id
JOIN files f ON c.doc_id = f.id
LEFT JOIN chunk_clusters cct ON c.id = cct.chunk_id
LEFT JOIN clusters ct on cct.cluster_id = ct.id
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
                        cluster: f.get(6)?,
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
        utils::initialize_cluster_tables(&codex_txn)?;
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

    fn get_doc_cluster(
        &self,
        sqlite_store: &SqliteStore,
        doc_id: &str,
    ) -> Result<Vec<types::ClusteredDocs>, SqliteError> {
        todo!()
    }

    fn get_tfidf_chunks(
        &self,
        sqlite_store: &SqliteStore,
        chunk_id: &str,
    ) -> Result<Vec<(String, String, Vec<f32>)>, SqliteError> {
        todo!()
    }

    fn write_cluster_info(
        &self,
        sqlite_store: &SqliteStore,
        cluster_id: i32,
        name: &str,
    ) -> Result<(), SqliteError> {
        let conn = sqlite_store.codex_conn.lock()?;

        conn.execute(
            "INSERT INTO clusters (id, name) values (?1, ?2)",
            rusqlite::params![cluster_id, name],
        )?;

        Ok(())
    }

    fn write_cluster_chunks(
        &self,
        sqlite_store: &SqliteStore,
        assignments: &[(String, i32)],
    ) -> Result<(), SqliteError> {
        let mut conn = sqlite_store.codex_conn.lock()?;
        let txn = conn.transaction()?;
        let mut stmt =
            txn.prepare("INSERT into chunk_clusters (chunk_id, cluster_id) values (?1, ?2)")?;

        for (chunk_id, cluster_id) in assignments {
            stmt.execute(rusqlite::params![chunk_id, cluster_id])?;
        }

        drop(stmt);

        txn.commit()?;

        Ok(())
    }

    fn get_all_chunks(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<types::ChunksSql>, SqliteError> {
        let conn = sqlite_store.codex_conn.lock()?;
        let mut stmt = conn.prepare("SELECT id, doc_id, start_char, end_char FROM chunks")?;

        let chunks = stmt
            .query_map([], |row| {
                Ok(ChunksSql {
                    id: row.get(0)?,
                    doc_id: row.get(1)?,
                    start_char: row.get::<_, i64>(2)? as usize,
                    end_char: row.get::<_, i64>(3)? as usize,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(chunks)
    }

    fn get_all_embeddings(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<ChunkEmbeddingSql>, SqliteError> {
        let conn = sqlite_store.codex_conn.lock()?;
        let mut stmt = conn.prepare(
            "SELECT 
            ce.chunk_id, 
            c.doc_id, 
            ce.embedding 
            FROM chunk_embeddings ce
            join chunks c
            on c.id = ce.chunk_id",
        )?;
        let embeddings = stmt
            .query_map([], |row| {
                let chunk_id: String = row.get(0)?;
                let doc_id: String = row.get(1)?;
                let bytes: Vec<u8> = row.get(2)?;
                Ok((chunk_id, doc_id, bytes))
            })?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|(chunk_id, doc_id, bytes)| {
                let embedding: Vec<f32> = bytes
                    .chunks_exact(4)
                    .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                    .collect();
                ChunkEmbeddingSql {
                    chunk_id,
                    doc_id,
                    embedding,
                }
            })
            .collect::<Vec<ChunkEmbeddingSql>>();

        Ok(embeddings)
    }

    fn clear_clusters(&self, sqlite_store: &SqliteStore) -> Result<(), SqliteError> {
        let mut conn = sqlite_store.codex_conn.lock()?;
        let txn = conn.transaction()?;

        txn.execute("delete from chunk_clusters", [])?;
        txn.execute("delete from clusters", [])?;

        txn.commit()?;

        Ok(())
    }

    fn list_files_with_top_clusters(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<types::FileDetailWithTopCluster>, SqliteError> {
        let mut conn = sqlite_store.codex_conn.lock()?;
        let mut stmt = conn.prepare(
            "
        SELECT 
            f.id, 
            f.name, 
            f.extensions,
            f.created_at,
            c.name as cluster_name,
            COUNT(cc.chunk_id) * 100.0 / SUM(COUNT(cc.chunk_id)) OVER (PARTITION by f.id) as pct
        from files f 
        left join chunks ch on ch.doc_id = f.id 
        left join chunk_clusters cc on cc.chunk_id = ch.id 
        left join clusters c on c.id = cc.cluster_id
        group by f.id, c.id 
        order by f.id, pct desc
            ",
        )?;

        let rows = stmt
            .query_map([], |row| {
                Ok(FileDetailWithTopCluster {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    extension: row.get(2)?,
                    created_at: row.get(3)?,
                    cluster_name: row.get(4)?,
                    top_cluster_pct: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut seen: HashSet<String> = HashSet::new();

        let files = rows
            .into_iter()
            .filter(|f| seen.insert(f.id.clone()))
            .collect();

        return Ok(files);
    }

    fn get_cluster_files(
        &self,
        sqlite_store: &SqliteStore,
        cluster_id: i32,
    ) -> Result<Vec<ClusterFile>, SqliteError> {
        let mut conn = sqlite_store.codex_conn.lock()?;

        let mut stmt = conn.prepare(
            "
        select
            f.id,
            f.name,
            COUNT(cc.chunk_id) as chunk_count,
            COUNT(cc.chunk_id) * 100.0 / total.total_chunks as pct
        from files f
        join chunks ch on ch.doc_id = f.id
        join chunk_clusters cc on cc.chunk_id = ch.id
        join (
            select c2.doc_id, COUNT(*) as total_chunks
            from chunks c2
            group by c2.doc_id
        ) total on total.doc_id = f.id
        where cc.cluster_id = ?1
        group by f.id
        order by pct DESC

        ",
        )?;

        let files = stmt
            .query_map(rusqlite::params![cluster_id], |row| {
                Ok(ClusterFile {
                    file_id: row.get(0)?,
                    file_name: row.get(1)?,
                    chunk_count: row.get::<_, i64>(2)? as usize,
                    cluster_match: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(files)
    }

    fn get_embeds_dim(&self, sqlite_store: &SqliteStore) -> Result<u32, SqliteError> {
        let mut conn = sqlite_store.codex_conn.lock()?;

        let schema: String = conn.query_row(
            "select sql from sqlite_master where name = 'chunk_embeddings';",
            [],
            |row| row.get(0),
        )?;

        let current_dims = schema
            .split("FLOAT[")
            .nth(1)
            .and_then(|s| s.split(']').next())
            .and_then(|d| d.parse::<u32>().ok());

        match current_dims {
            Some(curr) => Ok(curr),
            None => Err(SqliteError::Corrupt(
                "not able to extract dims from 'chunk_embeddings'".to_string(),
            )),
        }
    }

    fn reset_embedding_tables(
        &self,
        sqlite_store: &SqliteStore,
        dims: u32,
    ) -> Result<(), SqliteError> {
        let mut conn = sqlite_store.codex_conn.lock()?;

        let txn = conn.transaction()?;

        txn.execute("drop table chunk_embeddings;", [])?;

        txn.execute("drop table chunks;", [])?;

        utils::initialize_tables_embeddings(&txn, &dims)?;
        txn.commit()?;

        Ok(())
    }

    fn get_basic_cluster_info(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<storage_types::BasicClusterInfo>, SqliteError> {
        let mut conn = sqlite_store.codex_conn.lock()?;

        let mut stmt = conn.prepare(
            "SELECT
        c.id,
        c.name,
        COUNT(DISTINCT cc.chunk_id) as chunk_count,
        COUNT(DISTINCT ch.doc_id) as file_count,
        c.created_at
        from clusters c
        left join chunk_clusters cc ON cc.cluster_id = c.id
        left join chunks ch ON ch.id = cc.chunk_id
        group by c.id
        order by chunk_count DESC
        ",
        )?;

        let mut stats = stmt
            .query_map([], |row| {
                Ok(storage_types::BasicClusterInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    chunk_count: row.get::<_, i64>(2)? as usize,
                    file_count: row.get::<_, i64>(3)? as usize,
                    created_at: row.get(4)?,
                    top_files: Vec::new(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // query 2 — top 3 files per cluster
        let mut top_stmt = conn.prepare(
            "
        SELECT
            cc.cluster_id,
            f.name,
            COUNT(cc.chunk_id) as chunk_count
        FROM chunk_clusters cc
        JOIN chunks ch ON cc.chunk_id = ch.id
        JOIN files f ON ch.doc_id = f.id
        GROUP BY cc.cluster_id, f.id
        ORDER BY cc.cluster_id, chunk_count DESC
    ",
        )?;

        // collect into HashMap<cluster_id, Vec<file_name>>
        let mut top_files: HashMap<i64, Vec<String>> = HashMap::new();
        let rows = top_stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        for (cluster_id, file_name) in rows {
            let files = top_files.entry(cluster_id).or_default();
            if files.len() < 3 {
                files.push(file_name);
            }
        }

        // merge top_files into stats
        for stat in &mut stats {
            if let Some(files) = top_files.get(&stat.id) {
                stat.top_files = files.clone();
            }
        }

        Ok(stats)
    }
}
