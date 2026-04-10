use std::cmp::max;
#[derive(Clone)]
pub struct Column {
    pub text: String,
    pub spacing: usize,
}

impl Column {
    pub fn new<S: ToString>(text: S) -> Self {
        Self {
            text: text.to_string(),
            spacing: 0,
        }
    }
    pub fn update_spacing(&mut self, word: String) {
        self.spacing = word.len().saturating_sub(self.text.len())
    }
}

#[derive(Clone)]
pub struct Rows {
    pub value: String,
}

impl Rows {
    pub fn new<S: ToString>(value: S) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

pub struct TabularView {
    pub cols: Vec<Column>,
    pub rows: Vec<Vec<Option<Rows>>>,
}

impl TabularView {
    pub fn new(cols: Vec<Column>) -> Self {
        Self {
            cols,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<Option<String>>) -> anyhow::Result<()> {
        if !(row.len() == self.cols.len()) {
            anyhow::bail!("table assertion failed, length of column and row are not the same")
        }

        let rows_infered = row
            .into_iter()
            .map(|s| match s {
                None => None,
                Some(se) => Some(Rows::new(se)),
            })
            .collect::<Vec<Option<Rows>>>();
        self.rows.push(rows_infered.clone());

        self.calculate_spacing_single_row(rows_infered)?;
        Ok(())
    }

    pub fn assertions(&self) -> bool {
        self.rows.iter().all(|row| row.len() == self.cols.len())
    }

    pub fn calculate_spacing(&mut self) -> anyhow::Result<()> {
        if !self.assertions() {
            anyhow::bail!("table assertion failed, length of column and row are not the same")
        }

        let table_row = self.rows.clone();

        for rows in table_row {
            self.calculate_spacing_single_row(rows)?;
        }

        Ok(())
    }

    pub fn calculate_spacing_single_row(&mut self, row: Vec<Option<Rows>>) -> anyhow::Result<()> {
        if !(row.len() == self.cols.len()) {
            anyhow::bail!("table assertion failed, length of column and row are not the same")
        }

        for col_idx in 0..self.cols.len() {
            let cont: String = match &row[col_idx] {
                Some(val) => val.value.clone(),
                None => String::from("none"),
            };

            let current_col = &self.cols[col_idx];

            if current_col.spacing + current_col.text.len() <= cont.len() {
                self.cols[col_idx].update_spacing(cont);
            }
        }
        Ok(())
    }

    pub fn render(&self) -> anyhow::Result<String> {
        if !self.assertions() {
            anyhow::bail!("table assertion failed, length of column and rows are not the same")
        }
        let mut main_string = String::from("");
        let total_width: usize = self.cols.iter().map(|c| c.text.len() + c.spacing).sum();
        main_string.push_str("  ");
        main_string.push_str(&"-".repeat(total_width + 2 * self.cols.len() - 3));

        main_string.push('\n');
        main_string.push_str("  ");
        for col in &self.cols {
            main_string.push_str(&format!("|{}{}", col.text, " ".repeat(col.spacing)));
        }

        main_string.push('|');
        main_string.push('\n');
        main_string.push_str("  ");
        main_string.push_str(&"-".repeat(total_width + 2 * self.cols.len() - 3));
        main_string.push('\n');
        main_string.push_str("  ");
        for row in &self.rows {
            for c_idx in 0..row.len() {
                let content = match &row[c_idx] {
                    Some(c) => c.value.clone(),
                    None => String::from("None"),
                };
                let spacing = max(
                    ((content.len() as isize)
                        - (self.cols[c_idx].text.len() + self.cols[c_idx].spacing) as isize)
                        .abs(),
                    0,
                );

                main_string.push_str(&format!("|{}{}", content, " ".repeat(spacing as usize)));
            }

            main_string.push('|');
            main_string.push('\n');
            main_string.push_str("  ");
        }
        main_string.push_str(&"-".repeat(total_width + 2 * self.cols.len() - 3));
        main_string.push('\n');

        main_string.push_str("  ");
        main_string.push_str(&format!("Showing {} Records", self.rows.len()));
        Ok(main_string)
    }
    pub fn print(&self) {
        match self.render() {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error Recieved, could not create table: {}", e),
        }
    }
}
