use std::{collections::HashMap, path::Path};

use crate::{
    storage::error::StorageError,
    tfidf::{bigram::BigramIterator, chunkreader::ChunkReader, tokenizer::Tokenizer},
};

pub struct TermCounter;

impl TermCounter {
    pub fn count<R: Iterator<Item = Result<String, StorageError>>>(
        tokens: R,
    ) -> Result<HashMap<String, usize>, StorageError> {
        let mut counts = HashMap::new();

        for result in tokens {
            let token = result?;
            *counts.entry(token).or_insert(0) += 1
        }

        Ok(counts)
    }

    pub fn count_with_bigrams<R: Iterator<Item = Result<String, StorageError>>>(
        tokens: R,
    ) -> Result<HashMap<String, usize>, StorageError> {
        let bigram_iter = BigramIterator::new(tokens);
        Self::count(bigram_iter)
    }

    pub fn count_from_file_with_bigram<P: AsRef<Path>>(
        path: P,
        buff: usize,
    ) -> Result<HashMap<String, usize>, StorageError> {
        let chunks = ChunkReader::open(path, buff)?;
        let tokens = Tokenizer::new(chunks);
        Self::count_with_bigrams(tokens)
    }

    pub fn count_from_str(tokens: Vec<String>) -> HashMap<String, usize> {
        Self::count(tokens.into_iter().map(|s| Ok(s))).unwrap()
    }
    pub fn count_from_file<P: AsRef<Path>>(
        path: P,
        buff: usize,
    ) -> Result<HashMap<String, usize>, StorageError> {
        // FIX: Because we are streaming the file, we can take in incomplete words, make sure that
        // we have a variable that will keep track of incomplete words, and append it before
        //
        // NOTE: What is an incomplete word??
        // Lets say "Hello world, this is a new program", it may be represented as
        // <SOS>Hello<Space>world,<Space>this<Space>is<Space>a<Space>new<Space>program<EOS>
        // each word can be characterized by or if there are <Space>,<Sos>,<EOS>,\n,punctuatiosn after or before
        // it... so therefore if we go through document and we get "Hello world, th" we can say
        // that th is an incomplete word, and we add it to a list?, string? this means th should
        // not be passed to the tokenizer and only "Hello world," should be
        // is an incomplete word
        // tokenizing the sentence
        let chunks = ChunkReader::open(path, buff)?;
        let tokens = Tokenizer::new(chunks);

        TermCounter::count(tokens)
    }
}
