use rusqlite::Transaction;

use crate::storage::{error::SqliteError, sqlite_version::v1::types::TriggerTables};

pub fn initialize_updated_at_triggers(
    txn: &Transaction,
    trig_tables: Vec<TriggerTables>,
) -> Result<(), SqliteError> {
    // WARN: this code assumes that the table has a column name `id` which is acts as the primary key
    // for the table, please only use this when that is the case...
    for trig in trig_tables {
        txn.execute(
            &format!(
                "create trigger if not exists update_{table_name}_{col_name}
                    after update on {table_name}
                    begin 
                    update {table_name}
                    set {col_name} = current_timestamp
                    where id = old.id;
                    end;
                ",
                table_name = trig.table_name,
                col_name = trig.col
            ),
            (),
        )?;
    }

    Ok(())
}

pub fn initialize_tables_codex(codex_txn: &Transaction) -> Result<(), SqliteError> {
    codex_txn.execute(
        "create table if not exists info (
                    codex_version text not null,
                    sqlite_version text not null,
                    storage_version text not null,
                    updated_at datetime default current_timestamp
                )",
        (),
    )?;
    codex_txn.execute(
        "create table if not exists files (
                        id text primary key,
                        name text not null,
                        extensions text not null,
                        hash text not null,
                        created_at datetime default current_timestamp,
                        modified_at datetime default current_timestamp,
                        indexed_at datetime,
                        embedded_at datetime,
                        clustered_at datetime
                )",
        (),
    )?;
    codex_txn.execute(
        "create table if not exists keywords (
                        id text primary key,
                        name text not null unique,
                        doc_count integer not null default 1,
                        created_at datetime default current_timestamp
                )",
        (),
    )?;

    codex_txn.execute(
        "create table if not exists files_keywords (
                    file_id text not null,
                    keyword_id text not null,
                    frequency integer not null default 1,
                    foreign key (file_id) references files(id) on delete cascade,
                    foreign key (keyword_id) references keywords(id) on delete cascade
                )",
        (),
    )?;

    Ok(())
}

pub fn initialize_tables_embeddings(
    codex_txn: &Transaction,
    dims: &u32,
) -> Result<(), SqliteError> {
    codex_txn.execute(
        "create table if not exists chunks (
            id TEXT primary key,
            doc_id TEXT NOT NULL,
            chunk_index INTEGER NOT NULL,
            start_char INTEGER NOT NULL,
            end_char INTEGER NOT NULL,
            
            foreign key (doc_id) REFERENCES files(id) on delete cascade
    )",
        [],
    )?;

    codex_txn.execute(
        &format!(
            "
        CREATE VIRTUAL TABLE IF NOT EXISTS chunk_embeddings USING vec0(
            chunk_id TEXT,
            embedding FLOAT[{dims}] distance_metric=cosine,
        );
        "
        ),
        [],
    )?;

    Ok(())
}

pub fn initialize_cluster_tables(codex_txn: &Transaction) -> Result<(), SqliteError> {
    codex_txn.execute(
        "
    create table if not exists clusters (
    id integer primary key,
    name text not null,
    created_at datetime default current_timestamp
    )
        ",
        [],
    )?;

    codex_txn.execute(
        "
    create table if not exists chunk_clusters (
    chunk_id text not null,
    cluster_id integer not null,
    clustered_at datetime default current_timestamp,

    foreign key (cluster_id) references clusters(id)
    )
        ",
        [],
    )?;

    Ok(())
}

pub fn initialize_tables_cache(cache_txn: &Transaction) -> Result<(), SqliteError> {
    cache_txn.execute(
        "create table if not exists content_cache (
                    id text primary key,
                    content text not null,
                    cached_at datetime default current_timestamp
                )",
        (),
    )?;

    Ok(())
}
