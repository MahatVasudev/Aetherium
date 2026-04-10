use std::{
    cmp,
    ops::{Add, Deref},
};

use terminal_size::{Height, Width, terminal_size};

use crate::views::{
    config::{ContentCellConfig, ContentTableCellConfig, ContentTableConfig, MetaDataCellConfig},
    types::{AllignmentType, BoldType, Color, ITALLICType},
    utils::{alignText, saturating_string, wrap_text_single_line},
};

// Main Goal of making a view like this.................
// ---------------| |---------------------------------------------------------------------- |
// |    title     | |       some content                                                    |
// |  sub title   | |        lorem ipsum                                                    |
// _______________| |---------------------------------------------------------------------- |
//
//

pub trait Render {
    fn render(&self) -> String;
}

impl Render for ContentTable {
    fn render(&self) -> String {
        let mut result = String::new();

        for r in &self.data {
            result.push_str(&r.render());
            result.push('\n');
            result.push_str(
                &self
                    .config
                    .row_separator
                    .repeat(self.config.window_width as usize),
            );
            result.push('\n');
            result.push('\n');
        }

        result.push_str(&alignText(
            &format!("...showing {} data", self.data.len()),
            &AllignmentType::RIGHT,
            self.config.window_width as usize,
            ' ',
        ));

        result
    }
}

impl Render for ContentTableCell {
    fn render(&self) -> String {
        let meta_lines: Vec<String> = self
            .meta_data
            .render()
            .lines()
            .map(|s| s.to_string())
            .collect();
        let content_columns: Vec<Vec<String>> = self
            .content_cell
            .iter()
            .map(|c| c.render().lines().map(|s| s.to_string()).collect())
            .collect();

        let meta_height = meta_lines.len();

        let content_height = content_columns
            .iter()
            .map(|col| col.len())
            .max()
            .unwrap_or(0);

        let height = meta_height.max(content_height);
        let meta_width = self.meta_data.config.max_letters + self.config.padding;

        let mut meta_lines = meta_lines;
        while meta_lines.len() < height {
            meta_lines.push(" ".repeat(meta_width - self.config.padding));
        }

        let content_width = self
            .content_cell
            .get(0)
            .map(|c| c.config.max_letters_per_line)
            .unwrap_or(0);

        let mut content_columns = content_columns;

        for col in &mut content_columns {
            while col.len() < height {
                col.push(" ".repeat(content_width));
            }
        }
        let content_widths: Vec<usize> = self
            .content_cell
            .iter()
            .map(|c| c.config.max_letters_per_line)
            .collect();
        let meta_border = " "
            .repeat(self.config.padding)
            .add(&self.config.horizontal_border_style.repeat(meta_width));

        let content_borders: Vec<String> = content_widths
            .iter()
            .map(|w| {
                " ".repeat(2 * self.config.padding)
                    .add(&self.config.horizontal_border_style.repeat(*w))
            })
            .collect();

        let mut border_line = String::new();
        let is_more_cols = content_columns.len() > 1;
        border_line.push_str(&meta_border);

        border_line.push_str(&" ".repeat(self.config.padding));
        for b in &content_borders {
            border_line.push_str(b);
        }

        border_line.push('\n');

        let mut result = String::new();

        result.push_str(&border_line);
        for i in 0..height {
            result.push_str(&self.config.vertical_border_style);

            // meta column
            result.push_str(&" ".repeat(self.config.padding));
            result.push_str(&meta_lines[i]);
            result.push_str(&" ".repeat(meta_width.saturating_sub(meta_lines[i].chars().count())));
            result.push_str(&self.config.vertical_border_style);

            // spacing between meta and content
            result.push_str(&" ".repeat(self.config.padding));

            // content columns
            for (idx, col) in content_columns.iter().enumerate() {
                result.push_str(&self.config.vertical_border_style);
                result.push_str(&col[i]);

                if is_more_cols && idx < content_columns.len() - 1 {
                    result.push_str(&self.config.vertical_border_style);
                    result.push_str(&" ".repeat(self.config.padding));
                }
            }

            result.push_str(&self.config.vertical_border_style);
            result.push('\n');
        }

        result.push_str(&border_line);
        result
    }
}

impl Render for ContentCell {
    fn render(&self) -> String {
        let raw_lines = self.content.split('\n');

        let mut lines = vec![];

        for line in raw_lines {
            lines.extend(wrap_text_single_line(
                line,
                self.config.max_letters_per_line,
            ));
        }

        let more_lines = lines.len().saturating_sub(self.config.max_lines);

        lines.truncate(self.config.max_lines);

        let lines: Vec<_> = lines
            .into_iter()
            .map(|l| {
                alignText(
                    &l,
                    &self.config.content_allignment,
                    self.config.max_letters_per_line,
                    ' ',
                )
            })
            .collect();

        let mut result = lines.join("\n");
        result.push('\n');
        if more_lines > 0 {
            result.push_str(&alignText(
                &format!("...{more_lines} more lines"),
                &AllignmentType::RIGHT,
                self.config.max_letters_per_line,
                ' ',
            ));
        }

        result
    }
}

impl Render for MetaDataCell {
    fn render(&self) -> String {
        let title = saturating_string(&self.title, self.config.max_letters);

        let sub_title = saturating_string(&self.sub_title, self.config.max_letters);

        let tags = saturating_string(&self.tags.join("/"), self.config.max_letters);

        let additional_string = wrap_text_single_line(
            &self
                .additional_description
                .to_owned()
                .unwrap_or("".to_string()),
            self.config.max_letters,
        );

        let mut result = String::from("");

        let max_width = self.config.max_letters;

        result.push_str(&alignText(
            &title,
            &self.config.title_allignment,
            max_width,
            ' ',
        ));
        result.push('\n');

        result.push('\n');
        result.push_str(&alignText(
            &sub_title,
            &self.config.sub_title_allignment,
            max_width,
            ' ',
        ));

        result.push('\n');
        result.push('\n');

        result.push_str(&alignText(
            &tags,
            &self.config.tags_allignment,
            max_width,
            ' ',
        ));

        result.push('\n');
        result.push('\n');
        result.push_str(
            &additional_string
                .into_iter()
                .map(|s| {
                    alignText(
                        &s,
                        &self.config.additional_content_allignment,
                        max_width,
                        ' ',
                    )
                })
                .collect::<Vec<String>>()
                .join("\n"),
        );
        result
    }
}

pub struct ContentTable {
    data: Vec<ContentTableCell>,
    config: ContentTableConfig,
}

impl ContentTable {
    pub fn new(contents: Vec<ContentTableCell>, config: Option<ContentTableConfig>) -> Self {
        Self {
            data: contents,
            config: config.unwrap_or_default(),
        }
    }

    pub fn build_relative_window(
        contents: Vec<(MetaDataCell, Vec<String>)>,
        table_config: Option<ContentTableConfig>,
        configs: (Option<ContentTableCellConfig>, Option<ContentCellConfig>),
        window_width: u16,
        window_height: u16,
    ) -> Self {
        // Try not to add max_per_line config in the ContentCellConfig
        //
        let mut data: Vec<ContentTableCell> = Vec::new();
        let padding = configs.0.to_owned().unwrap_or_default().padding;
        for (mtd, ctd) in contents {
            let mtd_width = mtd.config.max_letters + padding;

            let num_contents = ctd.len();

            let individual_width = window_width
                .saturating_sub(mtd_width as u16)
                .saturating_sub(2 * ((1 + ctd.len()) * padding) as u16)
                .saturating_div(num_contents as u16);

            let contents = ctd
                .into_iter()
                .map(|f| {
                    ContentCell::new(
                        f,
                        Some(ContentCellConfig {
                            max_letters_per_line: individual_width as usize,
                            ..configs.1.to_owned().unwrap_or_default()
                        }),
                    )
                })
                .collect::<Vec<ContentCell>>();

            data.push(ContentTableCell::new(mtd, contents, configs.0.clone()));
        }

        Self {
            data,
            config: ContentTableConfig {
                window_width,
                window_height,
                ..table_config.unwrap_or_default()
            },
        }
    }

    pub fn build(
        contents: Vec<(MetaDataCell, Vec<String>)>,
        table_config: Option<ContentTableConfig>,
        configs: (Option<ContentTableCellConfig>, Option<ContentCellConfig>),
    ) -> Self {
        let width: u16;
        let height: u16;
        if let Some((Width(w), Height(h))) = terminal_size() {
            width = w;
            height = h;
        } else {
            println!("Got error during width and height");
            width = 200;
            height = 160;
        }

        Self::build_relative_window(contents, table_config, configs, width, height)
    }
}

pub struct ContentTableCell {
    pub meta_data: MetaDataCell, // can only have a single metadata cell
    pub content_cell: Vec<ContentCell>, // a cell can have multiple content cell
    pub config: ContentTableCellConfig, //
}

impl ContentTableCell {
    pub fn new(
        meta_data: MetaDataCell,
        content_cell: Vec<ContentCell>,
        config: Option<ContentTableCellConfig>,
    ) -> Self {
        Self {
            meta_data,
            content_cell,
            config: config.clone().unwrap_or_default(),
        }
    }
}

pub struct MetaDataCell {
    pub title: String,
    pub sub_title: String,
    pub tags: Vec<String>,
    pub additional_description: Option<String>,
    pub config: MetaDataCellConfig,
}

#[derive(Clone)]
pub struct ContentCell {
    content: String,
    config: ContentCellConfig,
}

impl MetaDataCell {
    pub fn new(
        title: &str,
        sub_title: &str,
        tags: Vec<String>,
        additional_description: Option<String>,
        config: Option<MetaDataCellConfig>,
    ) -> Self {
        Self {
            title: title.to_string(),
            sub_title: sub_title.to_string(),
            tags,
            additional_description,
            config: config.unwrap_or_default(),
        }
    }
}

impl ContentCell {
    pub fn new(content: String, config: Option<ContentCellConfig>) -> Self {
        Self {
            content,
            config: config.unwrap_or_default(),
        }
    }
}
