pub mod types;
pub mod utils;

use uuid::Uuid;

use crate::storage::{
    error::SqliteError,
    sqlite::SqliteStore,
    sqlite_version::{layout::SqliteLayout, v1::types::TriggerTables},
};

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
        let mut codex_conn = sqlite_store.codex_conn.borrow_mut();

        let codex_txn = codex_conn.transaction()?;

        codex_txn.execute("update files set hash = ?2 where id = ?1", (id, hash))?;

        codex_txn.commit()?;

        Ok(())
    }

    fn delete(&self, sqlite_store: &SqliteStore, fileid: String) -> Result<(), SqliteError> {
        let mut codex_conn = sqlite_store.codex_conn.borrow_mut();
        let codex_txn = codex_conn.transaction()?;

        codex_txn.execute("delete from files where id = ?1;", (fileid,))?;

        codex_txn.commit()?;
        Ok(())
    }

    fn get_all_files(
        &self,
        sqlite_store: &SqliteStore,
    ) -> Result<Vec<types::FileInSQL>, SqliteError> {
        // FIXME: Add a batch retrieval, so that it is scalable
        let codex_conn = sqlite_store.codex_conn.borrow_mut();
        let mut query = codex_conn.prepare(
            "
            select 
                id, 
                name, 
                hash,
                extensions, 
                created_at, 
                modified_at 
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
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(files)
    }

    fn add_metadata(
        &self,
        sqlite_store: &SqliteStore,
        file: &types::FileInSQL,
    ) -> Result<(), SqliteError> {
        let mut codex_conn = sqlite_store.codex_conn.borrow_mut();
        let codex_txn = codex_conn.transaction()?;

        codex_txn.execute(
            "
        insert into files (id, name, extensions, hash)
        values (?1,?2,?3,?4);",
            (
                file.id.clone(),
                file.name.clone(),
                file.extension.clone(),
                file.hash.clone(),
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

        let mut codex_conn = sqlite_store.codex_conn.borrow_mut();
        let mut cache_conn = sqlite_store.cache_conn.borrow_mut();
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

    fn reindex_file(
        &self,
        sqlite_store: &SqliteStore,
        file: &types::FileInSQL,
        term_frequencies: &std::collections::HashMap<String, usize>,
    ) -> Result<(), SqliteError> {
        let mut conn = sqlite_store.codex_conn.borrow_mut();
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
