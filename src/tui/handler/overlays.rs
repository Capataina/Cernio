use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::App;

pub fn handle_grade_picker(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.show_grade_picker = false;
        }
        KeyCode::Char(c) => {
            let grade = match c {
                '1' => Some("SS"),
                '2' => Some("S"),
                '3' => Some("A"),
                '4' => Some("B"),
                '5' => Some("C"),
                '6' => Some("F"),
                _ => None,
            };
            if let Some(g) = grade {
                app.override_grade(g);
            } else {
                app.show_grade_picker = false;
            }
        }
        _ => {
            app.show_grade_picker = false;
        }
    }
}

pub fn handle_bulk_picker(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.show_bulk_picker = false;
        }
        KeyCode::Char(c) => {
            let grade = match c {
                '1' => Some("SS"),
                '2' => Some("S"),
                '3' => Some("A"),
                '4' => Some("B"),
                '5' => Some("C"),
                '6' => Some("F"),
                _ => None,
            };
            if let Some(g) = grade {
                let action = app.bulk_action.clone();
                app.bulk_decision_by_grade(g, &action);
            }
            app.show_bulk_picker = false;
        }
        _ => {
            app.show_bulk_picker = false;
        }
    }
}

pub fn handle_search_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.search_mode = false;
            app.search_query.clear();
            app.refresh();
        }
        KeyCode::Enter => {
            app.search_mode = false;
            // Keep filter active — do not clear search_query.
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.refresh();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.refresh();
        }
        _ => {}
    }
}
