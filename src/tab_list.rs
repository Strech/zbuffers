use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use zellij_tile::prelude::*;

#[derive(Debug, Default)]
pub struct TabList {
    tab_infos: Vec<TabInfo>,
    selected_index: Option<usize>,
    selected_search_index: Option<usize>,
    search_results: Vec<SearchResult>,
    search_term: String,
    is_searching: bool,
}

#[derive(Debug)]
pub struct SearchResult {
    score: i64,
    indices: Vec<usize>,
    tab_name: String,
    tab_position: usize,
    is_current_tab: bool,
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
                        self.selected_index = Some(selected_tab.saturating_add(1));
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
                        self.selected_index = Some(selected_tab.saturating_sub(1));
                    } else {
                        self.selected_index = None;
                    }
                }
            }
        }
    }

    pub fn go_to_selected_tab(&self) {
        if self.is_searching {
            match self.selected_search_index {
                Some(selected_tab) => {
                    if let Some(search_result) = self.search_results.get(selected_tab) {
                        close_focus();
                        go_to_tab(search_result.tab_position as u32)
                    }
                }
                None => {}
            }
        } else {
            match self.selected_index {
                Some(selected_tab) => {
                    if let Some(tab_info) = self.tab_infos.get(selected_tab) {
                        close_focus();
                        go_to_tab(tab_info.position as u32)
                    }
                }
                None => {}
            }
        }
    }

    pub fn push_search_character(&mut self, character: char) {
        self.search_term.push(character);
        self.update_search_term();
    }

    pub fn pop_search_character(&mut self) {
        self.search_term.pop();
        self.update_search_term();
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
                    self.render_tab_info(search_result.is_current_tab),
                    self.render_tab_name(
                        &search_result.tab_name,
                        Some(search_result.indices.clone()),
                    ),
                    self.render_more_indication_as_needed(
                        i,
                        first_row_index_to_render,
                        last_row_index_to_render,
                        self.search_results.len(),
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

    fn render_all_entries(&self, table_rows: usize, _table_columns: usize) -> Table {
        let mut table = Table::new().add_row(vec![" ", " ", " "]); // skip the title row
        let (first_row_index_to_render, last_row_index_to_render) =
            self.range_to_render(table_rows, self.tab_infos.len(), self.selected_index);

        for i in first_row_index_to_render..last_row_index_to_render {
            if let Some(tab_info) = self.tab_infos.get(i) {
                let is_selected = Some(i) == self.selected_index;
                let mut table_cells = vec![
                    self.render_tab_info(tab_info.active),
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

    // https://doc.rust-lang.org/std/fmt/index.html#syntax
    fn render_tab_info(&self, is_current_tab: bool) -> Text {
        let text = if is_current_tab { "\u{2588}" } else { " " };
        Text::new(text).color_range(0, ..)
    }

    fn render_tab_name(&self, tab_name: &str, indices: Option<Vec<usize>>) -> Text {
        let text = Text::new(&format!("{:30}", tab_name));
        match indices {
            Some(indices) => text.color_indices(2, indices),
            None => text,
        }
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

    fn update_search_term(&mut self) {
        let mut matches = vec![];
        let matcher = SkimMatcherV2::default().use_cache(true);

        for tab_info in &self.tab_infos {
            if let Some((score, indices)) = matcher.fuzzy_indices(&tab_info.name, &self.search_term)
            {
                matches.push(SearchResult {
                    tab_name: tab_info.name.clone(),
                    tab_position: tab_info.position,
                    is_current_tab: tab_info.active,
                    score,
                    indices,
                });
            }
        }

        matches.sort_by(|a, b| b.score.cmp(&a.score));
        self.search_results = matches;
        self.is_searching = !self.search_term.is_empty();

        match self.selected_search_index {
            Some(search_index) => {
                if self.search_results.is_empty() {
                    self.selected_search_index = None;
                } else if search_index >= self.search_results.len() {
                    self.selected_search_index = Some(self.search_results.len().saturating_sub(1));
                }
            }
            None => self.selected_search_index = Some(0),
        }
    }
}
