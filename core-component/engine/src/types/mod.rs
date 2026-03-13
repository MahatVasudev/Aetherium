use aetherium_core::storage::{sqlite_version::v1::types::FileInSQL, storage_types::SyncEvent};

pub struct FileDetail {
    pub id: String,
    pub name: String,
    pub hash: String,
    pub extension: String,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
}

pub enum EngineEvent {
    Sync(SyncProgress),
    OperationStarted,
    OperationFinished,
}

pub enum SyncProgress {
    FileAdded {
        id: String,
        name: String,
    },
    FileRemoved {
        id: String,
    },
    FileUpdated {
        id: String,
    },
    Done {
        added: usize,
        removed: usize,
        updated: usize,
    },
}

impl From<SyncEvent> for SyncProgress {
    fn from(value: SyncEvent) -> Self {
        match value {
            SyncEvent::FileAdded { id, name } => SyncProgress::FileAdded {
                id: id.to_string(),
                name: name.to_string(),
            },
            SyncEvent::FileRemoved { id } => SyncProgress::FileRemoved { id: id.to_string() },
            SyncEvent::FileUpdated { id } => SyncProgress::FileUpdated { id: id.to_string() },
            SyncEvent::Done {
                added,
                removed,
                updated,
            } => SyncProgress::Done {
                added,
                removed,
                updated,
            },
        }
    }
}

impl From<&FileInSQL> for FileDetail {
    fn from(value: &FileInSQL) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            hash: value.hash.clone(),
            extension: value.extension.clone(),
            created_at: value.created_at.clone(),
            modified_at: value.modified_at.clone(),
        }
    }
}
