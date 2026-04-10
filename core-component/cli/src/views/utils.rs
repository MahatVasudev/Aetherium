use std::char;

use crate::views::types::AllignmentType;

pub fn saturating_string(content: &str, max_letters: usize) -> String {
    let mut t: String = content.to_string();

    if t.chars().count() > max_letters {
        t = t[..max_letters.saturating_sub(3)].to_string();

        t.push_str(&".".repeat(3));
    }

    t.to_string()
}

pub fn alignText(
    content: &str,
    alignment: &AllignmentType,
    total_width: usize,
    fill_char: char,
) -> String {
    let length = content.chars().count();

    let remaining = total_width.saturating_sub(length);
    let left: usize;
    let right: usize;
    match alignment {
        AllignmentType::LEFT => {
            left = 0;
            right = remaining;
        }
        AllignmentType::RIGHT => {
            left = remaining;
            right = 0;
        }
        AllignmentType::CENTER => {
            left = remaining.saturating_div(2);
            right = remaining.saturating_sub(left);
        }
    }

    format!(
        "{}{}{}",
        fill_char.to_string().repeat(left),
        content,
        fill_char.to_string().repeat(right)
    )
}

pub fn wrap_text_single_line(content: &str, max_per_line: usize) -> Vec<String> {
    let mut lines = vec![];
    let mut current = String::new();

    for word in content.split_whitespace() {
        if word.len() > max_per_line {
            lines.push(word[..max_per_line].to_string())
        }
        if current.len() + word.len() + 1 > max_per_line {
            lines.push(current);
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    return lines;
}
