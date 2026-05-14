use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::{App, FilterAxis};

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

pub fn handle_filter_menu(app: &mut App, key: KeyEvent) {
    let axes = FilterAxis::ALL;
    let current_axis = axes[app.filter_menu_axis.min(axes.len() - 1)];
    let chips = current_axis.chips();

    match key.code {
        KeyCode::Esc | KeyCode::Char('f') | KeyCode::Char('q') => {
            app.show_filter_menu = false;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.filter_menu_axis = (app.filter_menu_axis + 1) % axes.len();
            let new_chips = axes[app.filter_menu_axis].chips();
            app.filter_menu_chip = app.filter_menu_chip.min(new_chips.len().saturating_sub(1));
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.filter_menu_axis = if app.filter_menu_axis == 0 {
                axes.len() - 1
            } else {
                app.filter_menu_axis - 1
            };
            let new_chips = axes[app.filter_menu_axis].chips();
            app.filter_menu_chip = app.filter_menu_chip.min(new_chips.len().saturating_sub(1));
        }
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => {
            if !chips.is_empty() {
                app.filter_menu_chip = (app.filter_menu_chip + 1) % chips.len();
            }
        }
        KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => {
            if !chips.is_empty() {
                app.filter_menu_chip = if app.filter_menu_chip == 0 {
                    chips.len() - 1
                } else {
                    app.filter_menu_chip - 1
                };
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            if let Some(chip) = chips.get(app.filter_menu_chip) {
                let chip = (*chip).to_string();
                let set = filter_set_mut(app, current_axis);
                if set.contains(&chip) {
                    set.remove(&chip);
                } else {
                    set.insert(chip);
                }
                app.refresh();
            }
        }
        KeyCode::Char('c') => {
            // Clear every chip on every axis (including the default "active"
            // on archive). The user can press `r` to restore the default
            // archive=active behaviour.
            app.filters.clear_all();
            app.refresh();
            app.add_toast("All filters cleared".to_string());
        }
        KeyCode::Char('r') => {
            app.filters.reset();
            app.refresh();
            app.add_toast("Filters reset to defaults".to_string());
        }
        _ => {}
    }
}

fn filter_set_mut<'a>(
    app: &'a mut App,
    axis: FilterAxis,
) -> &'a mut std::collections::HashSet<String> {
    match axis {
        FilterAxis::Grade => &mut app.filters.grades,
        FilterAxis::Ats => &mut app.filters.ats,
        FilterAxis::Decision => &mut app.filters.decisions,
        FilterAxis::Package => &mut app.filters.package,
        FilterAxis::Archive => &mut app.filters.archive,
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
