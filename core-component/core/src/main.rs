use aetherium_core::{
    CURRENT_DIR,
    tfidf::{
        self, EXAMPLE_STATIC_DIR,
        chunkreader::ChunkReader,
        sentence_chunker::{SentenceChunker, SentenceChunkerBatcher},
    },
};

fn main() {
    println!(
        "Current Directory {}",
        CURRENT_DIR.as_ref().unwrap().to_str().unwrap()
    );
    println!(
        "Example docs Directory {}",
        EXAMPLE_STATIC_DIR.as_ref().unwrap().to_str().unwrap()
    );

    let counted = tfidf::term_counter::TermCounter::count_from_file(
        EXAMPLE_STATIC_DIR.as_ref().unwrap().join("poem.txt"),
        30,
    )
    .unwrap();

    let chunk =
        ChunkReader::open(EXAMPLE_STATIC_DIR.as_ref().unwrap().join("poem.txt"), 20).unwrap();

    let sentence_chunks = SentenceChunker::new(chunk, 512, 0, 2);
    sentence_chunks.for_each(|f| {
        let m = f.unwrap();
        println!("Chunk {}: {:?}", m.index, m.chunks)
    });

    let chunk =
        ChunkReader::open(EXAMPLE_STATIC_DIR.as_ref().unwrap().join("poem.txt"), 20).unwrap();
    let sentence_chunks_batch = SentenceChunkerBatcher::new(chunk, 3, 512, 2);

    let mut batch_idx = 0;
    sentence_chunks_batch.for_each(|f| {
        let m = f.unwrap();
        batch_idx += 1;
        println!("Batch {}", batch_idx);
        for i in m {
            println!("Chunk {}: {:?}", i.index, i.chunks)
        }
    });
}
