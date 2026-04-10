use std::fmt::format;

use crate::{storage::error::StorageError, tfidf::stopwords::stopwords};

pub struct BigramIterator<R: Iterator<Item = Result<String, StorageError>>> {
    source: R,
    prev: Option<String>,
    pending: Option<String>,
}

impl<R: Iterator<Item = Result<String, StorageError>>> BigramIterator<R> {
    pub fn new(source: R) -> Self {
        Self {
            source,
            prev: None,
            pending: None,
        }
    }
}

impl<R: Iterator<Item = Result<String, StorageError>>> Iterator for BigramIterator<R> {
    type Item = Result<String, StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(bigram) = self.pending.take() {
            return Some(Ok(bigram));
        }

        match self.source.next()? {
            Err(e) => return Some(Err(e)),
            Ok(token) => {
                if let Some(prev) = &self.prev {
                    let bigram = format!("{} {}", prev, token);

                    if !stopwords().contains(prev.as_str()) || !stopwords().contains(token.as_str())
                    {
                        self.pending = Some(bigram);
                    }
                }

                self.prev = Some(token.clone());

                if !stopwords().contains(&token) {
                    return self.next();
                }
                Some(Ok(token))
            }
        }
    }
}
