mod tab_list;
mod ui;

use crate::ui::TabUiInfo;
use std::collections::BTreeMap;
use zellij_tile::prelude::*;

use ui::render_prompt;
use ui::Colors;

use tab_list::TabList;

#[derive(Default)]
struct State {
    tabs: TabList,
    tab_name: Option<String>,
    search_term: String,
    colors: Colors,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            // PermissionType::ChangeApplicationState,
        ]);

        subscribe(&[EventType::ModeUpdate, EventType::TabUpdate, EventType::Key]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::Key(key) => {
                should_render = self.handle_key(key);
            }
            Event::TabUpdate(tab_infos) => {
                self.update_tab_infos(tab_infos);
                should_render = true;
            }
            Event::ModeUpdate(mode_info) => {
                self.colors = Colors::new(mode_info.style.colors);
                should_render = true;
            }
            _ => (),
        };
        should_render
    }

    fn render(&mut self, rows: usize, cols: usize) {
        let (x, y, width, height) = self.main_menu_size(rows, cols);

        let search_indication =
            Text::new(format!("Search: {}_", self.search_term)).color_range(2, ..7);
        print_text_with_coordinates(search_indication, x.saturating_sub(1), y + 2, None, None);

        // render_prompt(&self.search_term, self.colors, x, y + 2);

        let room_for_list = height.saturating_sub(6); // search line and controls;
        self.tabs.update_rows(room_for_list);

        let list = self
            .tabs
            .render(room_for_list, width.saturating_sub(7), self.colors); // 7 for various ui
        for (i, line) in list.iter().enumerate() {
            print!("\u{1b}[{};{}H{}", y + i + 5, x, line.render());
        }
    }
}

impl State {
    fn main_menu_size(&self, rows: usize, cols: usize) -> (usize, usize, usize, usize) {
        // x, y, width, height
        let width = cols;
        let x = 0;
        let y = 0;
        let height = rows.saturating_sub(y);

        (x, y, width, height)
    }

    fn handle_key(&mut self, key: Key) -> bool {
        // do we want it?
        // https://github.com/zellij-org/zellij/blob/95dc4d8466acad4e79f4a8d866b757b9fb8e5cd4/default-plugins/session-manager/src/main.rs#L190-L193
        // if self.error.is_some() {
        //     self.error = None;
        //     return true;
        // }

        let mut should_render = false;
        match key {
            Key::Up => {
                self.tabs.move_selection_up();
                should_render = true;
            }
            Key::Down => {
                self.tabs.move_selection_down();
                should_render = true;
            }
            Key::Esc => {
                hide_self();
            }
            _ => {}
        }

        should_render
    }

    fn update_tab_infos(&mut self, tab_infos: Vec<TabInfo>) {
        let tab_infos: Vec<TabUiInfo> = tab_infos
            .iter()
            .map(|tab_info| TabUiInfo::from_tab_info(tab_info))
            .collect();
        let current_tab_name = tab_infos.iter().find_map(|tab_info| {
            if tab_info.is_current_tab {
                Some(tab_info.name.clone())
            } else {
                None
            }
        });

        if let Some(current_tab_name) = current_tab_name {
            self.tab_name = Some(current_tab_name);
        }

        self.tabs.set_tabs(tab_infos);
    }
}
