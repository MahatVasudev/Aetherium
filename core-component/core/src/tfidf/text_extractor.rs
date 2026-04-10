use crate::storage::error::StorageError;

pub struct TextExtractor<R: Iterator<Item = Result<String, StorageError>>> {
    source: R,
    carry: String,
    position: usize,
}

pub struct TextChunk {
    pub text: String,
    pub start_char: usize,
}

impl<R: Iterator<Item = Result<String, StorageError>>> TextExtractor<R> {
    pub fn new(source: R) -> Self {
        Self {
            source,
            carry: "".into(),
            position: 0,
        }
    }

    fn split_by_para(string_stream: &str) -> (&str, &str) {
        if let Some(n) = string_stream.find("\n\n") {
            return (&string_stream[..n], &string_stream[n + 2..]);
        }

        ("", string_stream)
    }
}

impl<R: Iterator<Item = Result<String, StorageError>>> Iterator for TextExtractor<R> {
    type Item = Result<TextChunk, StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let carry = std::mem::take(&mut self.carry);
            let (complete_p, carry_p) = Self::split_by_para(&carry);

            if !complete_p.is_empty() {
                self.carry = carry_p.to_string();
                let start = self.position;
                self.position += complete_p.chars().count() + 2;
                return Some(Ok(TextChunk {
                    text: complete_p.to_string(),
                    start_char: start,
                }));
            }

            match self.source.next() {
                None => {
                    if !carry_p.is_empty() {
                        let start = self.position;
                        self.position += carry_p.chars().count();

                        return Some(Ok(TextChunk {
                            text: carry,
                            start_char: start,
                        }));
                    }

                    return None;
                }
                Some(result) => match result {
                    Ok(chunk) => {
                        self.carry = carry_p.to_string();
                        self.carry.push_str(&chunk);
                    }
                    Err(e) => return Some(Err(e)),
                },
            }
        }
    }
}
