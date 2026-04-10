use crate::tfidf::text_extractor::TextChunk;

#[derive(Clone, Debug)]
pub struct Sentence {
    pub text: String,
    pub start_char: usize,
    pub end_char: usize,
}

pub struct SentenceSplitter;

impl SentenceSplitter {
    pub fn split(para: &TextChunk) -> Vec<Sentence> {
        let mut is_quote = false;
        let mut sentences: Vec<Sentence> = Vec::new();
        let mut curr_sentence = String::from("");
        let mut start_char = para.start_char;
        let mut last_word = String::new();

        for (idx, text) in para.text.char_indices() {
            if text == '\"' {
                is_quote = !is_quote;
            }
            if is_quote {
                curr_sentence.push(text);
            } else if delimiter_checks(text) {
                if text == '.' {
                    if judge_should_split_period(&last_word) {
                        curr_sentence.push(text);

                        last_word.clear();
                        let end_char = start_char + curr_sentence.chars().count();
                        sentences.push(Sentence {
                            text: curr_sentence,
                            start_char,
                            end_char,
                        });

                        curr_sentence = "".into();
                        start_char = end_char + idx;
                    } else {
                        curr_sentence.push(text);
                        last_word.clear();
                    }
                } else {
                    curr_sentence.push(text);
                    last_word.clear();
                    let end_char = start_char + curr_sentence.chars().count();
                    sentences.push(Sentence {
                        text: curr_sentence,
                        start_char,
                        end_char,
                    });

                    curr_sentence = "".into();
                    start_char = end_char;
                }
            } else if text.is_whitespace() {
                last_word.clear();
                curr_sentence.push(text)
            } else {
                last_word.push(text);
                curr_sentence.push(text);
            }
        }

        if !curr_sentence.trim().is_empty() {
            let end_char = start_char + curr_sentence.chars().count();
            sentences.push(Sentence {
                text: curr_sentence,
                start_char,
                end_char,
            });
        }

        return sentences;
    }
}

fn judge_should_split_period(word: &str) -> bool {
    let abreviations = ["Mr", "Mrs", "Ms", "Dr", "Prof", "Sr", "Jr", "vs", "etc"];

    if abreviations.contains(&word) {
        return false;
    }

    if word.len() == 1 && word.chars().all(|f| f.is_uppercase()) {
        return false;
    }

    if word.chars().all(|f| f.is_numeric()) {
        return false;
    }

    true
}

fn delimiter_checks(text: char) -> bool {
    text == '.' || text == '!' || text == '?' || text == ';' || text == '\n'
}
