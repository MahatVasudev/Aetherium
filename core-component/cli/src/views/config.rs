use crate::views::types::{AllignmentType, BoldType, Color, ITALLICType};

// ====================================== List Configuration

#[derive(Clone)]
pub struct ContentTableConfig {
    pub row_separator: String,
    pub distance_between_rows: usize,
    pub window_width: u16,
    pub window_height: u16,
}

#[derive(Clone)]
pub struct ContentTableCellConfig {
    pub horizontal_border_style: String,
    pub vertical_border_style: String,
    pub padding: usize,
}

#[derive(Clone)]
pub struct ContentCellConfig {
    pub content_allignment: AllignmentType,
    pub word_allignment: AllignmentType,
    pub max_letters_per_line: usize,
    pub max_letters: usize,
    pub max_lines: usize,
    pub content_color: Color,
}

pub struct MetaDataCellConfig {
    pub title_allignment: AllignmentType,
    pub sub_title_allignment: AllignmentType,
    pub tags_allignment: AllignmentType,
    pub additional_content_allignment: AllignmentType,
    pub title_color: Color,
    pub sub_title_color: Color,
    pub tags_color: Color,
    pub addtional_content_color: Color,
    pub cell_color: Color,
    pub title_bold: BoldType,
    pub tags_bold: BoldType,
    pub addtional_content_bold: BoldType,
    pub sub_title_bold: BoldType,
    pub tile_italic: ITALLICType,
    pub sub_title_italic: ITALLICType,
    pub tags_italic: ITALLICType,
    pub addtional_content_italic: ITALLICType,
    pub max_letters: usize,
}

// ============================= Default of Configuration

impl Default for MetaDataCellConfig {
    fn default() -> Self {
        Self {
            title_allignment: AllignmentType::CENTER,
            sub_title_allignment: AllignmentType::CENTER,
            title_color: Color::DEFAULT,
            sub_title_color: Color::DEFAULT,
            title_bold: BoldType::SOME,
            sub_title_bold: BoldType::NONE,
            tile_italic: ITALLICType::NONE,
            sub_title_italic: ITALLICType::SOME,
            cell_color: Color::TRANSPARENT,
            max_letters: 20,
            addtional_content_bold: BoldType::NONE,
            tags_bold: BoldType::SOME,
            tags_color: Color::RANDOM,
            additional_content_allignment: AllignmentType::CENTER,
            addtional_content_color: Color::DEFAULT,
            addtional_content_italic: ITALLICType::NONE,
            tags_italic: ITALLICType::NONE,
            tags_allignment: AllignmentType::CENTER,
        }
    }
}

impl Default for ContentTableCellConfig {
    fn default() -> Self {
        Self {
            horizontal_border_style: String::from("-"),
            vertical_border_style: String::from("|"),
            padding: 2,
        }
    }
}

impl Default for ContentCellConfig {
    fn default() -> Self {
        Self {
            content_allignment: AllignmentType::CENTER,
            word_allignment: AllignmentType::LEFT,
            max_letters: 500,
            max_letters_per_line: 100,
            max_lines: 10,
            content_color: Color::DEFAULT,
        }
    }
}

impl Default for ContentTableConfig {
    fn default() -> Self {
        Self {
            row_separator: String::from("*"),
            distance_between_rows: 1,
            window_width: 700,
            window_height: 600,
        }
    }
}
