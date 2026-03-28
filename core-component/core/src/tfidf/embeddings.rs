pub struct Chunk {
    pub chunk_id: String,
    pub doc_id: String,
    pub chunk_index: usize,
    pub start_char: usize,
    pub end_char: usize,
}

pub struct ChunkEmbedding {
    pub chunk_id: String,
    pub embedding: Vec<f32>,
}
