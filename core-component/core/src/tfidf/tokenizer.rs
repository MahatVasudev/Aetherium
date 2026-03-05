use crate::storage::error::StorageError;

pub struct Tokenizer<R: Iterator<Item = Result<String, StorageError>>> {
    source: R,
    pending: Vec<String>,
}

impl<R: Iterator<Item = Result<String, StorageError>>> Tokenizer<R> {
    pub fn new(source: R) -> Self {
        Self {
            source,
            pending: Vec::new(),
        }
    }

    pub fn tokenize_chunk(chunk: &str) -> Vec<String> {
        chunk
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .filter(|s| s.chars().all(|c| c.is_alphanumeric()))
            .collect()
    }
}

impl<R: Iterator<Item = Result<String, StorageError>>> Iterator for Tokenizer<R> {
    type Item = Result<String, StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(token) = self.pending.pop() {
                return Some(Ok(token));
            }

            match self.source.next()? {
                Err(e) => return Some(Err(e)),
                Ok(chunk) => {
                    let mut tokens = Self::tokenize_chunk(&chunk);
                    tokens.reverse();
                    self.pending = tokens;
                }
            }
        }
    }
}
