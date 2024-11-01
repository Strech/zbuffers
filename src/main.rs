mod tab_list;
mod ui;

use std::collections::BTreeMap;
use zellij_tile::prelude::*;

use ui::Colors;

use tab_list::TabList;

#[derive(Default)]
struct State {
    tabs: TabList,
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
                self.tabs.update(tab_infos);
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

        self.tabs.render(height, width, x, y);
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
}
