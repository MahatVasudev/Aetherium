use crate::{
    storage::error::StorageError,
    tfidf::{
        sentence_splitter::{Sentence, SentenceSplitter},
        text_extractor::TextExtractor,
    },
};

pub struct SentenceChunkerBatcher<P: Iterator<Item = Result<String, StorageError>>> {
    pub chunker: SentenceChunker<P>,
    pub batch_size: usize,
}

impl<P: Iterator<Item = Result<String, StorageError>>> SentenceChunkerBatcher<P> {
    pub fn new(source: P, batch_size: usize, max_token: usize, overlap: usize) -> Self {
        Self {
            chunker: SentenceChunker::new(source, max_token, 0, overlap),
            batch_size,
        }
    }
}

impl<P: Iterator<Item = Result<String, StorageError>>> Iterator for SentenceChunkerBatcher<P> {
    type Item = Result<Vec<SentenceChunks>, StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut batches: Vec<SentenceChunks> = vec![];
        loop {
            match self.chunker.next() {
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    if !batches.is_empty() {
                        return Some(Ok(batches));
                    }

                    return None;
                }
                Some(Ok(chunks)) => {
                    batches.push(chunks);
                    if batches.len() >= self.batch_size {
                        return Some(Ok(batches));
                    }
                }
            }
        }
    }
}

pub struct SentenceChunks {
    pub chunks: Vec<Sentence>,
    pub index: usize,
    pub start_at: usize,
    pub end_at: usize,
}

pub struct SentenceChunker<P: Iterator<Item = Result<String, StorageError>>> {
    pub text_ext: TextExtractor<P>,
    pub chunk_buff: Vec<Sentence>,
    pub current_pos: usize,
    pub max_tokens: usize,
    pub overlap: usize,
    pub chunk_index: usize,
}

impl<P: Iterator<Item = Result<String, StorageError>>> SentenceChunker<P> {
    pub fn new(source: P, max_tokens: usize, chunk_index: usize, overlap: usize) -> Self {
        Self {
            text_ext: TextExtractor::new(source),
            chunk_buff: Vec::new(),
            current_pos: 0,
            max_tokens,
            chunk_index,
            overlap,
        }
    }
}

impl<P: Iterator<Item = Result<String, StorageError>>> Iterator for SentenceChunker<P> {
    type Item = Result<SentenceChunks, StorageError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.text_ext.next() {
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    if !self.chunk_buff.is_empty() {
                        let chunk_buff = std::mem::take(&mut self.chunk_buff);

                        return Some(Ok(SentenceChunks {
                            index: self.chunk_index,
                            start_at: chunk_buff[0].start_char,
                            end_at: chunk_buff[chunk_buff.len() - 1].end_char,
                            chunks: chunk_buff,
                        }));
                    }

                    return None;
                }
                Some(Ok(text_chunk)) => {
                    let mut split_sentences = SentenceSplitter::split(&text_chunk);
                    self.chunk_buff.append(&mut split_sentences);
                    let (chunked_vec, carry_vec): (Vec<Sentence>, Vec<Sentence>) =
                        split_by_tokens(std::mem::take(&mut self.chunk_buff), self.max_tokens);

                    if !chunked_vec.is_empty() {
                        let c_idx = self.chunk_index;
                        self.chunk_index += 1;

                        self.chunk_buff = chunked_vec
                            .get((chunked_vec.len() - self.overlap)..)
                            .unwrap_or_else(|| &chunked_vec[chunked_vec.len() - 1..])
                            .to_vec();

                        self.chunk_buff.extend(carry_vec);
                        return Some(Ok(SentenceChunks {
                            start_at: chunked_vec[0].start_char,
                            end_at: chunked_vec[chunked_vec.len() - 1].end_char,
                            chunks: chunked_vec,
                            index: c_idx,
                        }));
                    }

                    self.chunk_buff.extend(carry_vec);
                }
            }
        }
    }
}

fn split_by_tokens(
    split_sentences: Vec<Sentence>,
    max_token: usize,
) -> (Vec<Sentence>, Vec<Sentence>) {
    let mut tokens_curr: usize = 0;

    for idx in 0..split_sentences.len() {
        tokens_curr += split_sentences[idx].text.split_whitespace().count();

        if tokens_curr >= max_token {
            return (
                split_sentences[..idx].to_vec(),
                split_sentences[idx..].to_vec(),
            );
        }
    }

    (split_sentences, Vec::new())
}
