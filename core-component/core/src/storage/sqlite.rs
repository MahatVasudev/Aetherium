use crate::{
    storage::{
        CACHE_DB, CODEX_DB, Storage,
        error::{SqliteError, StorageError},
        sqlite_version::{
            SqliteStoreVersion,
            layout::SqliteLayout,
            v1::{
                SQLITESTOREV1,
                types::{
                    self, ChunkEmbeddingSql, ChunksSql, ClusteredDocs, FileDetailWithTopCluster,
                    FileInSQL, SemanticSearchResult,
                },
            },
        },
        storage_types::BasicClusterInfo,
        utils,
    },
    storage_assert,
    tfidf::embeddings::{self, Chunk, ChunkEmbedding},
};
use rusqlite::{Connection, fallible_iterator, ffi::sqlite3_auto_extension};
use sqlite_vec::sqlite3_vec_init;
use std::{
    cell::RefCell,
    collections::HashMap,
    path::Path,
    sync::{Mutex, Once},
};

static VEC_INIT: Once = Once::new();
pub struct SqliteStore {
    pub cache_conn: Mutex<Connection>,
    pub codex_conn: Mutex<Connection>,
    layout: Box<dyn SqliteLayout>,
}

impl SqliteStore {
    fn new(codex_conn: Connection, cache_conn: Connection, version: SqliteStoreVersion) -> Self {
        SqliteStore {
            cache_conn: Mutex::new(cache_conn),
            codex_conn: Mutex::new(codex_conn),
            layout: get_sqlite_version(version),
        }
    }

    pub fn version(&self) -> SqliteStoreVersion {
        self.layout.version()
    }

    pub fn build(
        storage: crate::storage::Storage,
        version: SqliteStoreVersion,
    ) -> Result<SqliteStore, SqliteError> {
        let database_folder = &storage.database_folder;
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }
        let codex_db = Connection::open(database_folder.join(CODEX_DB))?;
        let cache_db = Connection::open(database_folder.join(CACHE_DB))?;

        codex_db.execute_batch("PRAGMA foreign_keys = ON;")?;

        Ok(SqliteStore::new(codex_db, cache_db, version))
    }

    pub fn open(
        storage: &crate::storage::Storage,
    ) -> Result<crate::storage::sqlite::SqliteStore, SqliteError> {
        if let Ok(read_codex) = utils::read_codex_config(storage.root_folder()) {
            let database_folder = &storage.database_folder;
            let Some(version) = SqliteStoreVersion::parse(&read_codex.version.sqlitestore) else {
                return Err(SqliteError::AssertionFail(
                    "invalid SqliteStore Version Received".into(),
                ));
            };

            unsafe {
                sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
            }
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

    pub fn add_metadata(&self, file: &types::FileInSQL) -> Result<(), SqliteError> {
        self.layout.add_metadata(self, file)
    }

    pub fn update_hash(&self, fileid: String, hash: String) -> Result<(), SqliteError> {
        self.layout.update_hash(self, fileid, hash)
    }

    pub fn get_all_files(&self) -> Result<Vec<FileInSQL>, SqliteError> {
        self.layout.get_all_files(self)
    }

    pub fn reindex_file(
        &self,
        file: &FileInSQL,
        term_frequencies: &HashMap<String, usize>,
    ) -> Result<(), SqliteError> {
        self.layout.reindex_file(self, file, term_frequencies)
    }

    pub fn write_chunks(&self, chunks: &[Chunk]) -> Result<(), SqliteError> {
        self.layout.write_chunktext(self, chunks)
    }

    pub fn find_similar_embedding(
        &self,
        query_vector: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SemanticSearchResult>, SqliteError> {
        self.layout
            .find_similar_files_embedding(self, query_vector, top_k)
    }

    pub fn write_embeddings(&self, embeddings: &[ChunkEmbedding]) -> Result<(), SqliteError> {
        self.layout.write_embeddings(self, embeddings)
    }

    pub fn delete_chunks(&self, doc_id: &str) -> Result<(), SqliteError> {
        self.layout.delete_chunk(self, String::from(doc_id))
    }

    pub fn list_embeded_files(&self) -> Result<Vec<FileInSQL>, SqliteError> {
        self.layout.list_embeded_files(self)
    }

    pub fn list_not_embeded_files(&self) -> Result<Vec<FileInSQL>, SqliteError> {
        self.layout.list_not_embeded_files(self)
    }

    pub fn clear_clusters(&self) -> Result<(), SqliteError> {
        self.layout.clear_clusters(self)
    }

    pub fn write_cluster_info(
        &self,
        cluster_id: i32,
        cluster_name: &str,
    ) -> Result<(), SqliteError> {
        self.layout
            .write_cluster_info(self, cluster_id, cluster_name)
    }

    pub fn write_cluster_chunks(&self, assignments: &[(String, i32)]) -> Result<(), SqliteError> {
        self.layout.write_cluster_chunks(self, assignments)
    }

    pub fn get_all_chunks(&self) -> Result<Vec<ChunksSql>, SqliteError> {
        self.layout.get_all_chunks(self)
    }

    pub fn get_all_embeddings(&self) -> Result<Vec<ChunkEmbeddingSql>, SqliteError> {
        self.layout.get_all_embeddings(self)
    }

    pub fn get_doc_cluster(&self, doc_id: &str) -> Result<Vec<ClusteredDocs>, SqliteError> {
        self.layout.get_doc_cluster(self, doc_id)
    }

    pub fn check_embedding_dims(&self) -> Result<u32, SqliteError> {
        self.layout.get_embeds_dim(self)
    }

    pub fn list_files_with_top_clusters(
        &self,
    ) -> Result<Vec<FileDetailWithTopCluster>, SqliteError> {
        self.layout.list_files_with_top_clusters(self)
    }

    pub fn reset_embedding_dims(&self, new_dims: u32) -> Result<(), SqliteError> {
        self.layout.reset_embedding_tables(self, new_dims)
    }

    pub fn basic_describe_clusters(&self) -> Result<Vec<BasicClusterInfo>, SqliteError> {
        self.layout.get_basic_cluster_info(self)
    }
}

fn get_sqlite_version(version: SqliteStoreVersion) -> Box<dyn SqliteLayout> {
    match version {
        SqliteStoreVersion::V1 => Box::new(SQLITESTOREV1),
    }
}

#[cfg(test)]
mod testing {
    use std::{collections::HashSet, fs, path::Path};

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

        let written = codex
            .add_file(&raw_filename.to_path_buf(), None, 512)
            .unwrap();

        let written1 = codex
            .add_file(&raw_filename.to_path_buf(), None, 512)
            .unwrap();

        fs::remove_file(codex.storage.data_folder().join(written1.file_id)).unwrap();

        codex.storage.sync(&mut |_| {}).unwrap();
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
        let written = codex
            .add_file(&raw_filename.to_path_buf(), None, 512)
            .unwrap();
        let mut name = Uuid::new_v4().to_string();

        println!("{name}");
        fs::write(foldername.join(DATA_FOLDER).join(&name), b"hello world").unwrap();

        println!(
            "{:?}",
            fs::read_dir(codex.storage.data_folder())
                .unwrap()
                .map(|f| f.unwrap().path().to_string_lossy().to_string())
                .collect::<Vec<String>>()
        );

        codex.storage.sync(&mut |_| {}).unwrap();

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
        let written = codex
            .add_file(&raw_filename.to_path_buf(), None, 512)
            .unwrap();
        let name = Uuid::new_v4().to_string();
        fs::write(foldername.join(DATA_FOLDER).join(&name), b"hello world").unwrap();

        println!(
            "{:?}",
            fs::read_dir(codex.storage.data_folder())
                .unwrap()
                .map(|f| f.unwrap().path().to_string_lossy().to_string())
                .collect::<Vec<String>>()
        );

        codex.storage.sync(&mut |_| {}).unwrap();

        fs::write(
            foldername.join(DATA_FOLDER).join(&name),
            b"hello world changed",
        )
        .unwrap();

        codex.storage.sync(&mut |_| {}).unwrap();
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
