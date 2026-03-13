// Term Frequency, Inverse Document Frequency
//
//

// Lets work with some sample documents in a folder
//

pub mod chunkreader;
pub mod doc_counter;
pub mod sentence_splitter;
pub mod term_counter;
pub mod text_extractor;
pub mod tokenizer;
pub mod utils;
pub mod sentence_chunker;

use std::{
    collections::HashMap,
    path::{self, Path},
    sync::LazyLock,
};

use crate::{
    CURRENT_DIR,
    storage::{Storage, error::StorageError, sqlite_version::v1::types::FileInSQL},
    tfidf::{
        chunkreader::ChunkReader, doc_counter::DocumentCounter, term_counter::TermCounter,
        tokenizer::Tokenizer,
    },
};

pub static EXAMPLE_STATIC_DIR: LazyLock<Option<path::PathBuf>> = LazyLock::new(|| {
    CURRENT_DIR
        .as_ref()
        .and_then(|p| p.join("example-docs").into())
});

pub struct TFIDFCorpus {
    term_freq: HashMap<String, HashMap<String, usize>>,
    doc_freq: HashMap<String, usize>,
    total_docs: usize,
}

impl TFIDFCorpus {
    pub fn new() -> Self {
        Self {
            term_freq: HashMap::new(),
            doc_freq: HashMap::new(),
            total_docs: 0,
        }
    }

    pub fn compute_tf<P: AsRef<Path>>(
        file_path: P,
        chunk_size: usize,
    ) -> Result<HashMap<String, usize>, StorageError> {
        let chunks = ChunkReader::open(&file_path, chunk_size)?;
        let tokens = Tokenizer::new(chunks);

        TermCounter::count(tokens)
    }

    pub fn add_document(
        &mut self,
        file: &FileInSQL,
        storage: &Storage,
        chunk_size: usize,
    ) -> Result<(), StorageError> {
        let path = storage.data_folder().join(&file.id);

        let tf = {
            let chunks = ChunkReader::open(&path, chunk_size)?;
            let tokens = Tokenizer::new(chunks);

            TermCounter::count(tokens)?
        };

        {
            let chunks = ChunkReader::open(&path, chunk_size)?;
            let tokens = Tokenizer::new(chunks);
            let unique_tokens = DocumentCounter::unique_terms(tokens)?;
            DocumentCounter::accumulate(&mut self.doc_freq, unique_tokens);
        }

        self.term_freq.insert(file.id.clone(), tf);
        self.total_docs += 1;

        Ok(())
    }

    pub fn build_from_storage(storage: &Storage, chunk_size: usize) -> Result<Self, StorageError> {
        let mut corpus = Self::new();
        let files = storage.sqlite()?.get_all_files()?;

        for file in files {
            if !file.extension.starts_with("text/") {
                continue;
            }

            corpus.add_document(&file, storage, chunk_size)?;
        }

        Ok(corpus)
    }

    pub fn score(&self, doc_id: &str, term: &str) -> f64 {
        let tf = self
            .term_freq
            .get(doc_id)
            .and_then(|m| m.get(term))
            .copied()
            .unwrap_or(0);

        if tf == 0 {
            return 0.0;
        }

        let df = self.doc_freq.get(term).copied().unwrap_or(0);

        let idf = ((self.total_docs as f64 + 1.0) / (df as f64 + 1.0)).ln() + 1.0;

        tf as f64 * idf
    }

    pub fn top_terms(&self, doc_id: &str, n: usize) -> Vec<(String, f64)> {
        let Some(tf_map) = self.term_freq.get(doc_id) else {
            return vec![];
        };

        let mut scores: Vec<(String, f64)> = tf_map
            .keys()
            .map(|term| (term.clone(), self.score(doc_id, term)))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(n);
        scores
    }
}
