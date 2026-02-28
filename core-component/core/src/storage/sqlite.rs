use crate::storage::{
    CACHE_DB, CODEX_DB, Storage,
    error::{SqliteError, StorageError},
    sqlite_version::{
        SqliteStoreVersion,
        layout::SqliteLayout,
        v1::{
            SQLITESTOREV1,
            types::{self, Files},
        },
    },
    utils,
};
use rusqlite::Connection;
use std::cell::RefCell;

pub struct SqliteStore {
    pub cache_conn: RefCell<Connection>,
    pub codex_conn: RefCell<Connection>,
    layout: Box<dyn SqliteLayout>,
}

impl SqliteStore {
    fn new(codex_conn: Connection, cache_conn: Connection, version: SqliteStoreVersion) -> Self {
        SqliteStore {
            cache_conn: RefCell::new(cache_conn),
            codex_conn: RefCell::new(codex_conn),
            layout: get_sqlite_version(version),
        }
    }

    fn version(&self) -> SqliteStoreVersion {
        self.layout.version()
    }

    pub fn build(
        storage: crate::storage::Storage,
        version: SqliteStoreVersion,
    ) -> Result<SqliteStore, SqliteError> {
        let database_folder = &storage.database_folder;
        let codex_db = Connection::open(database_folder.join(CODEX_DB))?;
        let cache_db = Connection::open(database_folder.join(CACHE_DB))?;

        Ok(SqliteStore::new(codex_db, cache_db, version))
    }

    pub fn open(
        storage: &crate::storage::Storage,
    ) -> Result<crate::storage::sqlite::SqliteStore, SqliteError> {
        if let Ok(read_codex) = utils::read_codex_config(storage.root_folder()) {
            let database_folder = &storage.database_folder;
            let version = SqliteStoreVersion::parse(&read_codex.version.storage_sqlite);
            let codex_db = Connection::open(database_folder.join(CODEX_DB))?;
            let cache_db = Connection::open(database_folder.join(CACHE_DB))?;

            // codex_db.execute("PRAGMA journal_mode=WAL;", [])?;
            // codex_db.execute("PRAGMA busy_timeout=5000;", [])?;
            //
            // cache_db.execute("PRAGMA journal_mode=WAL;", [])?;
            // cache_db.execute("PRAGMA busy_timeout=5000;", [])?;

            return Ok(SqliteStore::new(codex_db, cache_db, version));
        }

        Err(SqliteError::AssertionFail(
            "Failed to open codex file".into(),
        ))
    }

    pub fn create_base(&self) -> Result<(), SqliteError> {
        self.layout.create_base(self)
    }

    pub fn delete(&self, fileid: String) -> Result<(), SqliteError> {
        self.layout.delete(self, fileid)
    }

    pub fn add_metadata(&self, file: types::Files) -> Result<(), SqliteError> {
        self.layout.add_metadata(self, file)
    }

    pub fn update_hash(&self, fileid: String, hash: String) -> Result<(), SqliteError> {
        self.layout.update_hash(self, fileid, hash)
    }

    pub fn get_all_files(&self) -> Result<Vec<Files>, SqliteError> {
        self.layout.get_all_files(self)
    }
}

fn get_sqlite_version(version: SqliteStoreVersion) -> Box<dyn SqliteLayout> {
    match version {
        SqliteStoreVersion::V1 => Box::new(SQLITESTOREV1),
    }
}

#[cfg(test)]
mod testing {
    use std::{collections::HashSet, fs, io::Write, path::Path};

    use rusqlite::fallible_iterator::Unwrap;
    use tempfile::tempdir;
    use uuid::Uuid;

    use crate::{
        codex::{Codex, versions::CodexVersion},
        storage::{
            DATA_FOLDER, sqlite_version::SqliteStoreVersion, storage_types::FileInSystem,
            versions::StorageVersion,
        },
    };

    #[test]
    fn added_data_consistent() {
        let dir = tempdir().unwrap();

        let foldername = dir.path().join("my_codex");

        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let sqlite_version = SqliteStoreVersion::V1;

        let codex =
            Codex::build(&foldername, codex_version, storage_version, sqlite_version).unwrap();
        let raw_filename = Path::new("/home/clyde/Downloads/sml_importance .pdf");

        let written = codex.add_file(&raw_filename.to_path_buf(), 512).unwrap();

        println!(
            "consistent {:?}",
            fs::read_dir(codex.storage.data_folder())
                .unwrap()
                .map(|f| f.unwrap().path().to_string_lossy().to_string())
                .collect::<Vec<String>>()
        );
        assert_eq!(
            codex
                .storage
                .sqlite()
                .unwrap()
                .get_all_files()
                .unwrap()
                .get(0)
                .unwrap()
                .id,
            written.file_id
        );
        assert_eq!(
            codex
                .storage
                .sqlite()
                .unwrap()
                .get_all_files()
                .unwrap()
                .get(0)
                .unwrap()
                .hash,
            written.file_hash.to_hex().to_string()
        );
    }

    #[test]
    fn added_data_consistent_from_outside() {
        let dir = tempdir().unwrap();

        let foldername = dir.path().join("my_codex");

        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let sqlite_version = SqliteStoreVersion::V1;

        let codex =
            Codex::build(&foldername, codex_version, storage_version, sqlite_version).unwrap();
        let raw_filename = Path::new("/home/clyde/Downloads/sml_importance .pdf");
        let written = codex.add_file(&raw_filename.to_path_buf(), 512).unwrap();
        let mut name = Uuid::new_v4().to_string();
        name.push_str(".txt");
        println!("{name}");
        fs::write(foldername.join(DATA_FOLDER).join(&name), b"hello world").unwrap();

        println!(
            "{:?}",
            fs::read_dir(codex.storage.data_folder())
                .unwrap()
                .map(|f| f.unwrap().path().to_string_lossy().to_string())
                .collect::<Vec<String>>()
        );

        codex.storage.sync().unwrap();

        assert_eq!(
            codex
                .storage
                .sqlite()
                .unwrap()
                .get_all_files()
                .unwrap()
                .into_iter()
                .map(|f| f.id)
                .collect::<HashSet<_>>(),
            vec![name, written.file_id]
                .into_iter()
                .collect::<HashSet<_>>()
        );
    }
    #[test]
    fn added_data_consistent_from_outside_changed() {
        let dir = tempdir().unwrap();

        let foldername = dir.path().join("my_codex");

        let codex_version = CodexVersion::V1;
        let storage_version = StorageVersion::V1;
        let sqlite_version = SqliteStoreVersion::V1;

        let codex =
            Codex::build(&foldername, codex_version, storage_version, sqlite_version).unwrap();
        let raw_filename = Path::new("/home/clyde/Downloads/sml_importance .pdf");
        let written = codex.add_file(&raw_filename.to_path_buf(), 512).unwrap();
        let name = Uuid::new_v4().to_string();
        fs::write(foldername.join(DATA_FOLDER).join(&name), b"hello world").unwrap();

        println!(
            "{:?}",
            fs::read_dir(codex.storage.data_folder())
                .unwrap()
                .map(|f| f.unwrap().path().to_string_lossy().to_string())
                .collect::<Vec<String>>()
        );

        codex.storage.sync().unwrap();

        fs::write(
            foldername.join(DATA_FOLDER).join(&name),
            b"hello world changed",
        )
        .unwrap();

        codex.storage.sync().unwrap();
        assert_eq!(
            codex
                .storage
                .sqlite()
                .unwrap()
                .get_all_files()
                .unwrap()
                .into_iter()
                .map(|f| f.id)
                .collect::<HashSet<_>>(),
            vec![name.clone(), written.file_id]
                .into_iter()
                .collect::<HashSet<_>>()
        );

        assert_eq!(
            codex
                .storage
                .sqlite()
                .unwrap()
                .get_all_files()
                .unwrap()
                .into_iter()
                .map(|f| f.hash)
                .collect::<HashSet<_>>(),
            vec![
                FileInSystem::from(&codex.storage, name.clone())
                    .unwrap()
                    .get_hash(&codex.storage)
                    .unwrap()
                    .to_hex()
                    .to_string(),
                written.file_hash.to_hex().to_string()
            ]
            .into_iter()
            .collect::<HashSet<_>>()
        );
    }
}
