mod implementations;
mod macros;

use std::io;

#[derive(Debug, PartialEq, Eq)]
pub enum StorageError {
    NotFound(String),

    Corrupt(String),

    Permission(String),

    InvalidFileId(String),

    Io(String),

    AssertionFail(String),

    SqliteError(SqliteError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SqliteError {
    DeleteFail(String),

    Corrupt(String),

    OpenFail(String),

    NotFound(String),

    InvalidQuery(String),

    AssertionFail(String),

    CreateFail(String),

    LockPoisoned,
}
