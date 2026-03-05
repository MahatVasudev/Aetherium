use core::{
    CURRENT_DIR,
    tfidf::{self, EXAMPLE_STATIC_DIR},
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

    println!("{:?}", counted)
}
