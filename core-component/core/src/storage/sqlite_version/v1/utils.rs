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
                        modified_at datetime default current_timestamp
                )",
        (),
    )?;
    codex_txn.execute(
        "create table if not exists keywords (
                        id text primary key,
                        name text not null,
                        created_at datetime default current_timestamp
                )",
        (),
    )?;

    codex_txn.execute(
        "create table if not exists files_keywords (
                    file_id text not null,
                    keyword_id text not null,

                    foreign key (file_id) references files(id),
                    foreign key (keyword_id) references keywords(id)
                )",
        (),
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
