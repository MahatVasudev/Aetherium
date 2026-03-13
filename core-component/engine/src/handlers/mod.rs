pub mod grpc_ops;
use std::path::Path;

use aetherium_core::{
    codex::{Codex, versions::CodexVersion},
    storage::{error::StorageError, sqlite_version::SqliteStoreVersion, versions::StorageVersion},
};
use tracing::{error, info, warn};

use crate::{
    EngineResponse,
    types::{EngineEvent, FileDetail},
};

pub fn handler_create_codex<P: AsRef<Path>>(
    path: P,
    codex_version: &str,
    storage_version: &str,
    sqlite_version: &str,
) -> EngineResponse {
    let codex_version = match CodexVersion::parse(&codex_version) {
        None => {
            warn!(
                "Received a non valid Codex Version, Defaulting to {}",
                CodexVersion::latest().as_str()
            );
            CodexVersion::latest()
        }
        Some(v) => v,
    };

    let storage_version = match StorageVersion::parse(&storage_version) {
        None => {
            warn!(
                "Received a non valid Storage Version, Defaulting to {}",
                StorageVersion::latest().as_str()
            );
            StorageVersion::latest()
        }
        Some(v) => v,
    };

    let sqlite_version = match SqliteStoreVersion::parse(&sqlite_version) {
        None => {
            warn!(
                "Received a non valid SqliteStore Version, Defaulting to {}",
                SqliteStoreVersion::latest().as_str()
            );
            SqliteStoreVersion::latest()
        }
        Some(v) => v,
    };
    let full_path = path.as_ref();
    let codex = match Codex::build(&full_path, codex_version, storage_version, sqlite_version) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while building codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    info!(
        "Codex Successfully Created: Details id: {}, name: {}, path: {}, version: {}, storage_version: {}, sqlitestore_version: {}",
        codex.id,
        codex.name,
        codex.storage.root_folder().to_str().unwrap(),
        codex.version().as_str(),
        codex.storage.version().as_str(),
        codex.storage.sqlite().unwrap().version().as_str()
    );

    return EngineResponse::CodexCreated {
        id: codex.id,
        name: codex.name,
    };
}

pub fn handler_open_codex<P: AsRef<Path>>(path: P) -> EngineResponse {
    let codex = match Codex::open(path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    info!(
        "Codex Successfully Opened: Details id: {}, name: {}, path: {}, version: {}, storage_version: {}, sqlitestore_version: {}",
        codex.id,
        codex.name,
        codex.storage.root_folder().to_str().unwrap(),
        codex.version().as_str(),
        codex.storage.version().as_str(),
        codex.storage.sqlite().unwrap().version().as_str()
    );

    EngineResponse::CodexOpened {
        id: codex.id,
        name: codex.name,
    }
}

pub fn handler_sync_live<P: AsRef<Path>>(
    codex_path: P,
    on_progress: &(dyn Fn(EngineEvent) + Send + Sync),
) -> EngineResponse {
    // TODO: Add Logging on each operation done in the codex, added, updated or deleted
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    match codex
        .storage
        .sync(&|event| on_progress(EngineEvent::Sync(event.into())))
    {
        Ok(_) => EngineResponse::Synced,
        Err(e) => EngineResponse::Error {
            message: e.message().into(),
        },
    }
}

pub fn handler_add_file<P: AsRef<Path>>(
    codex_path: P,
    file_path: P,
    file_name: Option<String>,
) -> EngineResponse {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    let codexconfig = match codex.read_config() {
        Ok(conf) => conf,
        Err(e) => {
            error!("Got Error while opening config file: {}", e.message(),);
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    let added_file = match codex.add_file(
        &file_path.as_ref().to_path_buf(),
        file_name,
        codexconfig.settings.write_chunk_size,
    ) {
        Ok(ad) => ad,
        Err(e) => {
            error!(
                "Got Error Adding File in Codex id: {}, name: {}, file path: {}",
                codex.id,
                codex.name,
                file_path.as_ref().to_string_lossy().to_string()
            );
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    EngineResponse::FileAdded {
        file_id: added_file.file_id,
        hash: added_file.file_hash.to_hex().to_string(),
    }
}

pub fn handler_codex_sync<P: AsRef<Path>>(codex_path: P) -> EngineResponse {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    match codex.storage.sync(&|_| {}) {
        Ok(_) => EngineResponse::Synced,
        Err(e) => {
            error!(
                "Error while syncing Codex: Details -> Id: {}, name: {}, error: {}",
                codex.id,
                codex.name,
                e.message()
            );
            EngineResponse::Error {
                message: e.message().into(),
            }
        }
    }
}

pub fn handler_codex_list_files<P: AsRef<Path>>(codex_path: P) -> EngineResponse {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    let sqlite_conn = match codex.storage.sqlite() {
        Err(err) => {
            error!(
                "Could not open sqlite store for codex: Details -> Id: {}, name: {}, error: {}",
                codex.id,
                codex.name,
                err.message()
            );
            return EngineResponse::Error {
                message: err.message().into(),
            };
        }
        Ok(conn) => conn,
    };

    match sqlite_conn.get_all_files() {
        Err(e) => {
            error!(
                "Could not retrieve all files, please check it, codex id: {}, name: {}, error: {}",
                codex.id,
                codex.name,
                e.message()
            );
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
        Ok(list) => EngineResponse::FileList {
            files: list
                .iter()
                .map(|f| -> FileDetail { f.into() })
                .collect::<Vec<FileDetail>>(),
        },
    }
}

pub fn handler_codex_delete_file<P: AsRef<Path>>(codex_path: P, file_id: String) -> EngineResponse {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    match codex.delete_file(file_id.clone()) {
        Ok(_) => {
            info!(
                "File {} deleted from the codex id: {}, name: {}",
                file_id, codex.id, codex.name
            );
            EngineResponse::FileDeleted
        }
        Err(e) => {
            error!(
                "File {} failed to be deleted from the codex id: {}, name: {}, error: {}",
                file_id,
                codex.id,
                codex.name,
                e.message()
            );
            EngineResponse::Error {
                message: e.message().into(),
            }
        }
    }
}

pub fn handler_get_config<P: AsRef<Path>>(codex_path: P, key: String) -> EngineResponse {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    match codex.get_config(key) {
        Ok(cf) => EngineResponse::GotConfig {
            value: Some(cf.to_string()),
        },

        Err(StorageError::NotFound(_)) => EngineResponse::GotConfig { value: None },

        Err(e) => EngineResponse::Error {
            message: e.message().into(),
        },
    }
}

pub fn handler_set_config<P: AsRef<Path>>(codex_path: P, key: &str, value: &str) -> EngineResponse {
    let codex = match Codex::open(codex_path.as_ref().into()) {
        Ok(c) => c,
        Err(e) => {
            error!("Got Error while opening codex: {}", e.message());
            return EngineResponse::Error {
                message: e.message().into(),
            };
        }
    };

    match codex.change_config(key, value) {
        Ok(_) => EngineResponse::SettedConfig {
            key: key.into(),
            val: value.into(),
        },
        Err(e) => EngineResponse::Error {
            message: e.message().into(),
        },
    }
}
