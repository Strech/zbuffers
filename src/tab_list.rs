use zellij_tile::prelude::*;

#[derive(Debug, Default)]
pub struct TabList {
    pub tab_infos: Vec<TabInfo>,
    pub selected_index: Option<usize>,
    pub selected_search_index: Option<usize>,
    pub search_results: Vec<SearchResult>,
    pub search_term: String,
    pub is_searching: bool,
}

impl TabList {
    pub fn update(&mut self, mut tab_infos: Vec<TabInfo>) {
        tab_infos.sort_unstable_by(|a, b| {
            if a.active {
                std::cmp::Ordering::Greater
            } else if b.active {
                std::cmp::Ordering::Less
            } else {
                a.name.cmp(&b.name)
            }
        });
        self.tab_infos = tab_infos;
    }

    pub fn render(&self, rows: usize, columns: usize, x: usize, y: usize) {
        let search_indication =
            Text::new(format!("Search: {}_", self.search_term)).color_range(2, ..7);
        let table_rows = rows.saturating_sub(5); // search row, toggle row and some padding
        let table_columns = columns;
        let table = if self.is_searching {
            self.render_search_results(table_rows, columns)
        } else {
            self.render_all_entries(table_rows, columns)
        };

        print_text_with_coordinates(search_indication, x.saturating_sub(1), y + 2, None, None);
        print_table_with_coordinates(table, x, y + 3, Some(table_columns), Some(table_rows));
    }

    fn render_search_results(&self, table_rows: usize, _table_columns: usize) -> Table {
        let mut table = Table::new().add_row(vec![" ", " ", " "]); // skip the title row
        let (first_row_index_to_render, last_row_index_to_render) = self.range_to_render(
            table_rows,
            self.search_results.len(),
            self.selected_search_index,
        );

        for i in first_row_index_to_render..last_row_index_to_render {
            if let Some(search_result) = self.search_results.get(i) {
                let is_selected = Some(i) == self.selected_search_index;
                let mut table_cells = vec![
                    self.render_tab_name(
                        &search_result.tab_name,
                        Some(search_result.indices.clone()),
                    ),
                    Text::new(""), //self.render_ctime(&search_result.ctime),
                    Text::new(""), // self.render_more_indication_or_enter_as_needed(
                                   //     i,
                                   //     first_row_index_to_render,
                                   //     last_row_index_to_render,
                                   //     self.search_results.len(),
                                   //     is_selected,
                                   // ),
                ];

                if is_selected {
                    table_cells = table_cells.drain(..).map(|t| t.selected()).collect();
                }
                table = table.add_styled_row(table_cells);
            }
        }
        table
    }

    fn render_all_entries(&self, table_rows: usize, _table_columns: usize) -> Table {
        let mut table = Table::new().add_row(vec![" ", " ", " "]); // skip the title row
        let (first_row_index_to_render, last_row_index_to_render) =
            self.range_to_render(table_rows, self.tab_infos.len(), self.selected_index);

        for i in first_row_index_to_render..last_row_index_to_render {
            if let Some(tab_info) = self.tab_infos.get(i) {
                let is_selected = Some(i) == self.selected_index;
                let mut table_cells = vec![
                    self.render_tab_info(tab_info),
                    self.render_tab_name(&tab_info.name, None),
                    self.render_more_indication_as_needed(
                        i,
                        first_row_index_to_render,
                        last_row_index_to_render,
                        self.tab_infos.len(),
                        is_selected,
                    ),
                ];
                if is_selected {
                    table_cells = table_cells.drain(..).map(|t| t.selected()).collect();
                }
                table = table.add_styled_row(table_cells);
            }
        }
        table
    }

    fn range_to_render(
        &self,
        table_rows: usize,
        results_len: usize,
        selected_index: Option<usize>,
    ) -> (usize, usize) {
        if table_rows <= results_len {
            let row_count_to_render = table_rows.saturating_sub(1); // 1 for the title
            let first_row_index_to_render = selected_index
                .unwrap_or(0)
                .saturating_sub(row_count_to_render / 2);
            let last_row_index_to_render = first_row_index_to_render + row_count_to_render;
            (first_row_index_to_render, last_row_index_to_render)
        } else {
            let first_row_index_to_render = 0;
            let last_row_index_to_render = results_len;
            (first_row_index_to_render, last_row_index_to_render)
        }
    }

    fn render_tab_name(&self, tab_name: &str, indices: Option<Vec<usize>>) -> Text {
        let text = Text::new(&format!("{:30}", tab_name));
        match indices {
            Some(indices) => text.color_indices(2, indices),
            None => text,
        }
    }

    // https://doc.rust-lang.org/std/fmt/index.html#syntax
    fn render_tab_info(&self, tab_info: &TabInfo) -> Text {
        let text = if tab_info.active { "\u{2588}" } else { " " };
        Text::new(text).color_range(0, ..)
    }

    fn render_more_indication_as_needed(
        &self,
        i: usize,
        first_row_index_to_render: usize,
        last_row_index_to_render: usize,
        results_len: usize,
        is_selected: bool,
    ) -> Text {
        if is_selected {
            Text::new("<Enter> - Focus on Tab").color_range(2, ..7)
        } else if i == first_row_index_to_render && i > 0 {
            Text::new(format!("+ {} more", first_row_index_to_render)).color_range(0, ..)
        } else if i == last_row_index_to_render.saturating_sub(1)
            && last_row_index_to_render < results_len
        {
            Text::new(format!(
                "+ {} more",
                results_len.saturating_sub(last_row_index_to_render)
            ))
            .color_range(0, ..)
        } else {
            Text::new(" ")
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.is_searching {
            match self.selected_search_index.as_mut() {
                Some(search_index) => {
                    *search_index = search_index.saturating_add(1);
                }
                None => {
                    if !self.search_results.is_empty() {
                        self.selected_search_index = Some(0);
                    }
                }
            }
        } else {
            match self.selected_index {
                None => {
                    if !self.tab_infos.is_empty() {
                        self.selected_index = Some(0);
                    }
                }
                Some(selected_tab) => {
                    if self.tab_infos.len() > selected_tab + 1 {
                        self.selected_index = Some(selected_tab + 1);
                    } else {
                        self.selected_index = None;
                    }
                }
            }
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.is_searching {
            match self.selected_search_index.as_mut() {
                Some(search_index) => {
                    *search_index = search_index.saturating_sub(1);
                }
                None => {
                    if !self.search_results.is_empty() {
                        self.selected_search_index = Some(0);
                    }
                }
            }
        } else {
            match self.selected_index {
                None => {
                    if !self.tab_infos.is_empty() {
                        self.selected_index = Some(self.tab_infos.len().saturating_sub(1))
                    }
                }
                Some(selected_tab) => {
                    if selected_tab > 0 {
                        self.selected_index = Some(selected_tab - 1);
                    } else {
                        self.selected_index = None;
                    }
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct SearchResult {
    score: i64,
    indices: Vec<usize>,
    // list_item: ListItem,
    tab_name: String,
    // tab_position: Option<usize>,
    // is_current_tab: bool,
}
