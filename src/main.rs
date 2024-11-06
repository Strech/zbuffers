mod tab_list;

use std::collections::BTreeMap;
use tab_list::TabList;
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    tabs: TabList,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);

        subscribe(&[EventType::TabUpdate, EventType::Key]);
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

    fn handle_key(&mut self, key: KeyWithModifier) -> bool {
        let mut should_render = false;

        match key.bare_key {
            BareKey::Esc if key.has_no_modifiers() => {
                hide_self();
            }
            BareKey::Up if key.has_no_modifiers() => {
                self.tabs.move_selection_up();
                should_render = true;
            }
            BareKey::Down if key.has_no_modifiers() => {
                self.tabs.move_selection_down();
                should_render = true;
            }
            BareKey::Backspace if key.has_no_modifiers() => {
                self.tabs.pop_search_character();
                should_render = true;
            }
            BareKey::Enter if key.has_no_modifiers() => {
                self.tabs.go_to_selected_tab();
                should_render = true;
            }
            BareKey::Char('g' | 'c') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                hide_self();
            }
            BareKey::Char('p' | 'k') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                self.tabs.move_selection_up();
                should_render = true;
            }
            BareKey::Char('n' | 'j') if key.has_modifiers(&[KeyModifier::Ctrl]) => {
                self.tabs.move_selection_down();
                should_render = true;
            }
            BareKey::Char(character) if key.has_no_modifiers() => {
                self.tabs.push_search_character(character);
                should_render = true;
            }
            _ => {}
        }

        should_render
    }
}
