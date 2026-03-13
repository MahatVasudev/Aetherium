use std::{fmt, sync::LockResult};

use crate::storage::error::{SqliteError, StorageError};

impl StorageError {
    pub fn message(&self) -> &str {
        match self {
            StorageError::NotFound(v)
            | StorageError::Corrupt(v)
            | StorageError::Permission(v)
            | StorageError::InvalidFileId(v) => v,
            StorageError::AssertionFail(v) => v,
            StorageError::Io(_) => "io error",
            StorageError::SqliteError(_) => self.message(),
        }
    }

    pub fn is_fatal(&self) -> bool {
        matches!(self, StorageError::Corrupt(_))
    }
}

impl SqliteError {
    pub fn message(&self) -> &str {
        use SqliteError::*;
        match self {
            DeleteFail(v) => v,
            Corrupt(v) => v,
            OpenFail(v) => v,
            NotFound(v) => v,
            InvalidQuery(v) => v,
            AssertionFail(v) => v,
            CreateFail(v) => v,
            LockPoisoned => "Mutex lock poisoned",
        }
    }
}

impl StorageError {
    pub fn not_found(file_id: &str) -> Self {
        StorageError::NotFound(format!("file not found: {file_id}"))
    }
    pub fn corrupt(file_id: &str) -> Self {
        StorageError::NotFound(format!("file corrupted: {file_id}"))
    }
    pub fn permission(file_id: &str) -> Self {
        StorageError::NotFound(format!("permission denied: {file_id}"))
    }
    pub fn invalid_file_id(file_id: &str) -> Self {
        StorageError::NotFound(format!("file not found: {file_id}"))
    }
    pub fn assertion_fail(args: std::fmt::Arguments) -> Self {
        StorageError::AssertionFail(args.to_string())
    }
}

impl std::error::Error for StorageError {}
impl std::error::Error for SqliteError {}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::AssertionFail(v) => write!(f, "storage assertion failed, {v}"),
            StorageError::NotFound(v) => write!(f, "not found: {v}"),
            StorageError::Corrupt(v) => write!(f, "storage corrupted: {v}"),
            StorageError::Permission(v) => write!(f, "permission denied: {v}"),
            StorageError::InvalidFileId(v) => write!(f, "invalid file id: {v}"),
            StorageError::Io(kind) => write!(f, "io error: {kind:?}"),
            StorageError::SqliteError(v) => write!(f, "sqlite error: {v:?}"),
        }
    }
}

impl fmt::Display for SqliteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqliteError::Corrupt(v) => write!(f, "database corrupted, {v}"),
            SqliteError::OpenFail(v) => write!(f, "failed to open database, {v}"),
            SqliteError::DeleteFail(v) => write!(f, "failed to delete, {v}"),
            SqliteError::CreateFail(v) => write!(f, "failed to create, {v}"),
            SqliteError::InvalidQuery(v) => write!(f, "passed an invalid query, {v}"),
            SqliteError::AssertionFail(v) => write!(f, "assertion error, {v}"),
            SqliteError::NotFound(v) => write!(f, "entries not found, {v}"),
            SqliteError::LockPoisoned => write!(f, "mutex lock error"),
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        use std::io::ErrorKind::*;

        match err.kind() {
            NotFound => StorageError::NotFound("file not found".into()),
            PermissionDenied => StorageError::Permission("permission denied".into()),
            _ => StorageError::Io(err.kind()),
        }
    }
}

impl From<SqliteError> for StorageError {
    fn from(err: SqliteError) -> Self {
        StorageError::SqliteError(err)
    }
}

impl From<rusqlite::Error> for SqliteError {
    fn from(err: rusqlite::Error) -> Self {
        use rusqlite::Error::*;

        match err {
            SqliteFailure(_, Some(msg)) => SqliteError::InvalidQuery(msg),

            InvalidQuery => SqliteError::InvalidQuery("invalid query".into()),

            QueryReturnedNoRows => SqliteError::NotFound("query returned no rows".into()),

            DatabaseBusy => SqliteError::OpenFail("database busy".into()),

            DatabaseCorrupt => SqliteError::Corrupt("database corrupt".into()),

            _ => SqliteError::OpenFail(err.to_string()),
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for SqliteError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        SqliteError::LockPoisoned
    }
}
