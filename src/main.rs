mod ui;

use std::collections::BTreeMap;
use zellij_tile::prelude::*;

use ui::render_prompt;
use ui::Colors;

#[derive(Default)]
struct State {
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

        subscribe(&[
            EventType::ModeUpdate,
            // EventType::TabUpdate,
            // EventType::Key
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
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

        render_prompt(&self.search_term, self.colors, x, y + 2);
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
}
