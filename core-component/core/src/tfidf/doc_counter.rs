use std::collections::{HashMap, HashSet};

use crate::storage::error::StorageError;

pub struct DocumentCounter;

impl DocumentCounter {
    pub fn unique_terms<R: Iterator<Item = Result<String, StorageError>>>(
        tokens: R,
    ) -> Result<HashSet<String>, StorageError> {
        let mut seen = HashSet::new();
        for token in tokens {
            seen.insert(token?);
        }

        Ok(seen)
    }

    pub fn accumulate(docs_freq: &mut HashMap<String, usize>, unique_terms: HashSet<String>) {
        for term in unique_terms {
            *docs_freq.entry(term).or_insert(0) += 1;
        }
    }
}
