use crate::ui::{Colors, LineToRender, ListItem, TabUiInfo};

// TODO: Remove macro
macro_rules! render_assets {
    ($assets:expr, $line_count_to_remove:expr, $selected_index:expr, $to_render_until_selected: expr, $to_render_after_selected:expr, $has_deeper_selected_assets:expr, $max_cols:expr, $colors:expr) => {{
        let (start_index, anchor_asset_index, end_index, line_count_to_remove) =
            minimize_lines($assets.len(), $line_count_to_remove, $selected_index);
        let mut truncated_result_count_above = start_index;
        let mut truncated_result_count_below = $assets.len().saturating_sub(end_index);
        let mut current_index = 1;
        if let Some(assets_to_render_before_selected) = $assets.get(start_index..anchor_asset_index)
        {
            for asset in assets_to_render_before_selected {
                let mut asset: LineToRender =
                    asset.as_line_to_render(current_index, $max_cols, $colors);
                asset.add_truncated_results(truncated_result_count_above);
                truncated_result_count_above = 0;
                current_index += 1;
                $to_render_until_selected.push(asset);
            }
        }
        if let Some(selected_asset) = $assets.get(anchor_asset_index) {
            if $selected_index.is_some() && !$has_deeper_selected_assets {
                let mut selected_asset: LineToRender =
                    selected_asset.as_line_to_render(current_index, $max_cols, $colors);
                selected_asset.make_selected(true);
                selected_asset.add_truncated_results(truncated_result_count_above);
                if anchor_asset_index + 1 >= end_index {
                    // no more results below, let's add the more indication if we need to
                    selected_asset.add_truncated_results(truncated_result_count_below);
                }
                current_index += 1;
                $to_render_until_selected.push(selected_asset);
            } else {
                $to_render_until_selected.push(selected_asset.as_line_to_render(
                    current_index,
                    $max_cols,
                    $colors,
                ));
                current_index += 1;
            }
        }
        if let Some(assets_to_render_after_selected) =
            $assets.get(anchor_asset_index + 1..end_index)
        {
            for asset in assets_to_render_after_selected.iter().rev() {
                let mut asset: LineToRender =
                    asset.as_line_to_render(current_index, $max_cols, $colors);
                asset.add_truncated_results(truncated_result_count_below);
                truncated_result_count_below = 0;
                current_index += 1;
                $to_render_after_selected.insert(0, asset.into());
            }
        }
        line_count_to_remove
    }};
}

pub fn minimize_lines(
    total_count: usize,
    line_count_to_remove: usize,
    selected_index: Option<usize>,
) -> (usize, usize, usize, usize) {
    // returns: (start_index, anchor_index, end_index, lines_left_to_remove)
    let (count_to_render, line_count_to_remove) = if line_count_to_remove > total_count {
        (1, line_count_to_remove.saturating_sub(total_count) + 1)
    } else {
        (total_count.saturating_sub(line_count_to_remove), 0)
    };
    let anchor_index = selected_index.unwrap_or(0); // 5
    let mut start_index = anchor_index.saturating_sub(count_to_render / 2);
    let mut end_index = start_index + count_to_render;
    if end_index > total_count {
        start_index = start_index.saturating_sub(end_index - total_count);
        end_index = total_count;
    }
    (start_index, anchor_index, end_index, line_count_to_remove)
}

#[derive(Debug, Default)]
pub struct TabList {
    pub tab_ui_infos: Vec<TabUiInfo>,
    pub selected_index: SelectedIndex,
    pub selected_search_index: Option<usize>,
    pub search_results: Vec<SearchResult>,
    pub is_searching: bool,
}

impl TabList {
    pub fn set_tabs(&mut self, mut tab_ui_infos: Vec<TabUiInfo>) {
        tab_ui_infos.sort_unstable_by(|a, b| {
            if a.is_current_tab {
                std::cmp::Ordering::Greater
            } else if b.is_current_tab {
                std::cmp::Ordering::Less
            } else {
                a.name.cmp(&b.name)
            }
        });
        self.tab_ui_infos = tab_ui_infos;
    }

    pub fn update_rows(&mut self, rows: usize) {
        if let Some(search_result_rows_until_selected) = self.selected_search_index.map(|i| {
            self.search_results
                .iter()
                .enumerate()
                .take(i + 1)
                .fold(0, |acc, s| acc + s.1.lines_to_render())
        }) {
            if search_result_rows_until_selected > rows
                || self.selected_search_index >= Some(self.search_results.len())
            {
                self.selected_search_index = None;
            }
        }
    }

    pub fn render(&self, max_rows: usize, max_cols: usize, colors: Colors) -> Vec<LineToRender> {
        if self.is_searching {
            self.render_search_results(max_rows, max_cols)
        } else {
            self.render_list(max_rows, max_cols, colors)
        }
    }

    fn render_search_results(&self, max_rows: usize, max_cols: usize) -> Vec<LineToRender> {
        let mut lines_to_render = vec![];
        for (i, result) in self.search_results.iter().enumerate() {
            if lines_to_render.len() + result.lines_to_render() <= max_rows {
                let mut result_lines = result.render(max_cols);
                if Some(i) == self.selected_search_index {
                    let mut render_arrows = true;
                    for line_to_render in result_lines.iter_mut() {
                        line_to_render.make_selected_as_search(render_arrows);
                        render_arrows = false; // only render arrows on the first search result
                    }
                }
                lines_to_render.append(&mut result_lines);
            } else {
                break;
            }
        }
        lines_to_render
    }

    fn render_list(&self, max_rows: usize, max_cols: usize, colors: Colors) -> Vec<LineToRender> {
        let mut lines_to_render_until_selected = vec![];
        let mut lines_to_render_after_selected = vec![];
        let total_lines_to_render = self.total_lines_to_render();
        let line_count_to_remove = total_lines_to_render.saturating_sub(max_rows);
        let line_count_to_remove = self.render_tabs(
            &mut lines_to_render_until_selected,
            &mut lines_to_render_after_selected,
            line_count_to_remove,
            max_cols,
            colors,
        );
        let mut lines_to_render = lines_to_render_until_selected;
        lines_to_render.append(&mut lines_to_render_after_selected);
        lines_to_render
    }

    fn total_lines_to_render(&self) -> usize {
        self.tab_ui_infos.len()
    }

    fn render_tabs(
        &self,
        to_render_until_selected: &mut Vec<LineToRender>,
        to_render_after_selected: &mut Vec<LineToRender>,
        line_count_to_remove: usize,
        max_cols: usize,
        colors: Colors,
    ) -> usize {
        render_assets!(
            self.tab_ui_infos,
            line_count_to_remove,
            self.selected_index.0,
            to_render_until_selected,
            to_render_after_selected,
            self.selected_index.0.is_some(),
            max_cols,
            colors
        )
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
                SelectedIndex(None) => {
                    if !self.tab_ui_infos.is_empty() {
                        self.selected_index.0 = Some(0);
                    }
                }
                SelectedIndex(Some(selected_tab)) => {
                    if self.tab_ui_infos.len() > selected_tab + 1 {
                        self.selected_index.0 = Some(selected_tab + 1);
                    } else {
                        self.selected_index.0 = None;
                    }
                }
                _ => {}
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
                SelectedIndex(None) => {
                    if !self.tab_ui_infos.is_empty() {
                        self.selected_index.0 = Some(self.tab_ui_infos.len().saturating_sub(1))
                    }
                }
                SelectedIndex(Some(selected_tab)) => {
                    if selected_tab > 0 {
                        self.selected_index.0 = Some(selected_tab - 1);
                    } else {
                        self.selected_index.0 = None;
                    }
                }
                _ => {}
            }
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct SelectedIndex(pub Option<usize>);

// TODO: Remove
// impl SelectedIndex {
//     pub fn tabs_are_visible(&self) -> bool {
//         self.0.is_some()
//     }
//     pub fn selected_tab_index(&self) -> Option<usize> {
//         self.0
//     }
//     pub fn tab_index_is_selected(&self, index: usize) -> bool {
//         self.0 == Some(index)
//     }
//     pub fn result_shrink(&mut self) {
//         match self {
//             SelectedIndex(None) => self.0 = None,
//             _ => {}
//         }
//     }
//     pub fn reset(&mut self) {
//         self.0 = None;
//     }
// }

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct SearchResult {
    score: i64,
    indices: Vec<usize>,
    list_item: ListItem,
    tab_name: String,
    tab_position: Option<usize>,
    is_current_tab: bool,
}

impl SearchResult {
    pub fn new(
        score: i64,
        indices: Vec<usize>,
        list_item: ListItem,
        tab_name: String,
        tab_position: Option<usize>,
        is_current_tab: bool,
    ) -> Self {
        SearchResult {
            score,
            indices,
            list_item,
            tab_name,
            tab_position,
            is_current_tab,
        }
    }
    pub fn lines_to_render(&self) -> usize {
        self.list_item.line_count()
    }
    pub fn render(&self, max_width: usize) -> Vec<LineToRender> {
        self.list_item.render(Some(self.indices.clone()), max_width)
    }
}
