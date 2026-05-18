use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::{App, FilterAxis};

pub fn handle_grade_picker(app: &mut App, key: KeyEvent) {
    crate::tel!("grade_picker_key", "code": format!("{:?}", key.code));
    match key.code {
        KeyCode::Esc => {
            crate::tel!("grade_picker_close");
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
                crate::tel!("grade_override_invoke", "grade": g);
                app.override_grade(g);
            } else {
                crate::tel!("grade_picker_close_non_digit");
                app.show_grade_picker = false;
            }
        }
        _ => {
            crate::tel!("grade_picker_close_other_key");
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
    crate::tel!(
        "filter_menu_key",
        "code": format!("{:?}", key.code),
        "axis": current_axis.label(),
        "chip_idx": app.filter_menu_chip,
        "chip_under_cursor": chips.get(app.filter_menu_chip).copied(),
    );

    match key.code {
        KeyCode::Esc | KeyCode::Char('f') | KeyCode::Char('q') => {
            crate::tel!("filter_menu_close");
            app.show_filter_menu = false;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.filter_menu_axis = (app.filter_menu_axis + 1) % axes.len();
            let new_chips = axes[app.filter_menu_axis].chips();
            app.filter_menu_chip = app.filter_menu_chip.min(new_chips.len().saturating_sub(1));
            crate::tel!(
                "filter_menu_axis_change",
                "new_axis": axes[app.filter_menu_axis].label(),
            );
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.filter_menu_axis = if app.filter_menu_axis == 0 {
                axes.len() - 1
            } else {
                app.filter_menu_axis - 1
            };
            let new_chips = axes[app.filter_menu_axis].chips();
            app.filter_menu_chip = app.filter_menu_chip.min(new_chips.len().saturating_sub(1));
            crate::tel!(
                "filter_menu_axis_change",
                "new_axis": axes[app.filter_menu_axis].label(),
            );
        }
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => {
            if !chips.is_empty() {
                app.filter_menu_chip = (app.filter_menu_chip + 1) % chips.len();
                crate::tel!(
                    "filter_menu_chip_focus",
                    "axis": current_axis.label(),
                    "chip": chips[app.filter_menu_chip],
                );
            }
        }
        KeyCode::Char('h') | KeyCode::Left | KeyCode::BackTab => {
            if !chips.is_empty() {
                app.filter_menu_chip = if app.filter_menu_chip == 0 {
                    chips.len() - 1
                } else {
                    app.filter_menu_chip - 1
                };
                crate::tel!(
                    "filter_menu_chip_focus",
                    "axis": current_axis.label(),
                    "chip": chips[app.filter_menu_chip],
                );
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            if let Some(chip) = chips.get(app.filter_menu_chip) {
                let chip = (*chip).to_string();
                let set = filter_set_mut(app, current_axis);
                let was_active = set.contains(&chip);
                if was_active {
                    set.remove(&chip);
                } else {
                    set.insert(chip.clone());
                }
                let now_active = !was_active;
                let active_count_after = app.filters.active_chip_count();
                crate::tel!(
                    "filter_chip_toggled",
                    "axis": current_axis.label(),
                    "chip": chip,
                    "now_active": now_active,
                    "active_chips_total": active_count_after,
                );
                app.refresh();
            }
        }
        KeyCode::Char('c') => {
            crate::tel!("filter_clear_all", "prev_active_chips": app.filters.active_chip_count());
            app.filters.clear_all();
            app.refresh();
            app.add_toast("All filters cleared".to_string());
        }
        KeyCode::Char('r') => {
            crate::tel!("filter_reset_defaults", "prev_active_chips": app.filters.active_chip_count());
            app.filters.reset();
            app.refresh();
            app.add_toast("Filters reset to defaults".to_string());
        }
        _ => {
            crate::tel!("filter_menu_key_unhandled", "code": format!("{:?}", key.code));
        }
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
        FilterAxis::Evidence => &mut app.filters.evidence,
    }
}

pub fn handle_search_input(app: &mut App, key: KeyEvent) {
    crate::tel!(
        "search_key",
        "code": format!("{:?}", key.code),
        "query_before": &app.search_query,
    );
    match key.code {
        KeyCode::Esc => {
            crate::tel!("search_cancel", "had_query": !app.search_query.is_empty());
            app.search_mode = false;
            app.search_query.clear();
            app.refresh();
        }
        KeyCode::Enter => {
            crate::tel!("search_commit", "query": &app.search_query);
            app.search_mode = false;
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            crate::tel!("search_query_change", "query": &app.search_query);
            app.refresh();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            crate::tel!("search_query_change", "query": &app.search_query);
            app.refresh();
        }
        _ => {}
    }
}
