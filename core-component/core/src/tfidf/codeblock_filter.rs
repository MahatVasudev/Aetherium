use crate::storage::error::StorageError;

pub struct CodeBlockFilter<R: Iterator<Item = Result<String, StorageError>>> {
    source: R,
    in_code_block: bool,
}

impl<R: Iterator<Item = Result<String, StorageError>>> CodeBlockFilter<R> {
    pub fn new(source: R) -> Self {
        Self {
            source,
            in_code_block: false,
        }
    }
}

impl<R: Iterator<Item = Result<String, StorageError>>> CodeBlockFilter<R> {
    pub fn filter_chunk(&mut self, chunk: &str) -> String {
        let mut result = String::new();
        let mut lines = chunk.lines().peekable();

        while let Some(line) = lines.next() {
            let trimmed = line.trim();

            if trimmed.starts_with("```") && !trimmed.starts_with("```ad-") {
                self.in_code_block = !self.in_code_block;
                continue;
            }

            if !self.in_code_block {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }
}

impl<R: Iterator<Item = Result<String, StorageError>>> Iterator for CodeBlockFilter<R> {
    type Item = Result<String, StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.source.next()? {
                Err(e) => return Some(Err(e)),
                Ok(chunk) => {
                    let filtered = self.filter_chunk(&chunk);
                    if !filtered.trim().is_empty() {
                        return Some(Ok(filtered));
                    }
                }
            }
        }
    }
}
