use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{App, View};

pub fn handle_dashboard(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.scroll_viewport_down(1),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_viewport_up(1),
        KeyCode::Char('D') => app.run_cleanup(),
        _ => {}
    }
}

pub fn handle_company_list(app: &mut App, key: KeyEvent) {
    use crate::tui::app::CompaniesLayout;

    // Lanes layout overrides most keys.
    if app.companies_layout == CompaniesLayout::Lanes {
        handle_company_lanes(app, key);
        return;
    }

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.next_in_list(),
        KeyCode::Char('k') | KeyCode::Up => app.prev_in_list(),
        KeyCode::Char('g') => app.go_to_top(),
        KeyCode::Char('G') => app.go_to_bottom(),
        KeyCode::Home => app.go_to_top(),
        KeyCode::End => app.go_to_bottom(),
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => app.enter_company_jobs(),
        KeyCode::Char('o') => app.open_selected_url(),
        KeyCode::Char('s') => app.toggle_sort(),
        KeyCode::Char('L') => {
            app.companies_layout = CompaniesLayout::Lanes;
            app.add_toast("Layout: Lanes".to_string());
        }
        KeyCode::Esc => app.clear_multi_select(),
        _ => {}
    }
}

fn handle_company_lanes(app: &mut App, key: KeyEvent) {
    use crate::tui::app::CompaniesLayout;
    use crate::tui::theme::{all_lanes, LANE_KEYS};

    // Compute the count of companies in the focused column for clamping.
    let lane_key = LANE_KEYS[app.companies_lane_col];
    let col_len = app.companies.iter().filter(|c| {
        all_lanes(c.lanes.as_deref()).iter().any(|k| k == lane_key)
    }).count();

    match key.code {
        KeyCode::Char('h') | KeyCode::Left => {
            if app.companies_lane_col > 0 {
                app.companies_lane_col -= 1;
            }
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if app.companies_lane_col < LANE_KEYS.len() - 1 {
                app.companies_lane_col += 1;
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let sel = &mut app.companies_lane_selections[app.companies_lane_col];
            if col_len > 0 && *sel + 1 < col_len {
                *sel += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let sel = &mut app.companies_lane_selections[app.companies_lane_col];
            if *sel > 0 { *sel -= 1; }
        }
        KeyCode::Char('g') | KeyCode::Home => {
            app.companies_lane_selections[app.companies_lane_col] = 0;
        }
        KeyCode::Char('G') | KeyCode::End => {
            if col_len > 0 {
                app.companies_lane_selections[app.companies_lane_col] = col_len - 1;
            }
        }
        KeyCode::Char('L') | KeyCode::Esc => {
            app.companies_layout = CompaniesLayout::Classic;
            app.add_toast("Layout: Classic".to_string());
        }
        KeyCode::Enter => {
            // Drill into the selected company's jobs.
            let lane_key = LANE_KEYS[app.companies_lane_col];
            let sel = app.companies_lane_selections[app.companies_lane_col];
            let idx = app.companies.iter().enumerate()
                .filter(|(_, c)| all_lanes(c.lanes.as_deref()).iter().any(|k| k == lane_key))
                .nth(sel)
                .map(|(i, _)| i);
            if let Some(i) = idx {
                // Mirror enter_company_jobs by setting selection first.
                app.company_state.select(Some(i));
                app.enter_company_jobs();
            }
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let n = c.to_digit(10).unwrap_or(1) as usize;
            if n >= 1 && n <= LANE_KEYS.len() {
                app.companies_lane_col = n - 1;
            }
        }
        _ => {}
    }
}

pub fn handle_job_list(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            app.show_quick_peek = false;
            app.next_in_list();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.show_quick_peek = false;
            app.prev_in_list();
        }
        KeyCode::Char(' ') => {
            app.show_quick_peek = !app.show_quick_peek;
        }
        KeyCode::Char('G') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.group_by_company = !app.group_by_company;
            app.add_toast(if app.group_by_company { "Grouped by company" } else { "Ungrouped" }.to_string());
        }
        KeyCode::Char('g') => {
            app.show_grade_picker = true;
        }
        KeyCode::Char('G') => app.go_to_bottom(),
        KeyCode::Home => app.go_to_top(),
        KeyCode::End => app.go_to_bottom(),
        KeyCode::Char('o') => app.open_selected_url(),
        KeyCode::Char('p') => app.autofill_selected_job(),
        KeyCode::Char('w') => app.record_decision_multi("watching"),
        KeyCode::Char('a') => app.record_decision_multi("applied"),
        KeyCode::Char('x') => app.record_decision_multi("rejected"),
        KeyCode::Char('i') => app.record_decision_multi("interview"),
        KeyCode::Char('W') => {
            app.bulk_action = "watching".to_string();
            app.show_bulk_picker = true;
        }
        KeyCode::Char('s') => app.toggle_sort(),
        KeyCode::Char(']') => app.jump_next_grade_section(),
        KeyCode::Char('[') => app.jump_prev_grade_section(),
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
            if app.show_quick_peek {
                app.show_quick_peek = false;
            } else if app.job_filter_company.is_some() {
                app.clear_job_filter();
                app.view = View::Companies;
                app.detail_scroll = 0;
            } else {
                app.clear_multi_select();
            }
        }
        _ => {}
    }
}

pub fn handle_activity(app: &mut App, key: KeyEvent) {
    let group_count = crate::tui::views::activity::group_count(app);
    let max_scroll = (app.activity_timeline.len() as u16).saturating_sub(10);
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.activity_cursor + 1 < group_count {
                app.activity_cursor += 1;
            }
            app.activity_scroll = app.activity_scroll.saturating_add(1).min(max_scroll);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.activity_cursor > 0 { app.activity_cursor -= 1; }
            app.activity_scroll = app.activity_scroll.saturating_sub(1);
        }
        KeyCode::Home => {
            app.activity_scroll = 0;
            app.activity_cursor = 0;
        }
        KeyCode::End => {
            app.activity_scroll = max_scroll;
            app.activity_cursor = group_count.saturating_sub(1);
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(group) = crate::tui::views::activity::group_at(app, app.activity_cursor) {
                let key = group.key();
                if app.activity_expanded.contains(&key) {
                    app.activity_expanded.remove(&key);
                } else {
                    app.activity_expanded.insert(key);
                }
            }
        }
        _ => {}
    }
}

pub fn handle_detail_scroll(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.scroll_detail_down(),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_detail_up(),
        KeyCode::Home => { app.detail_scroll = 0; }
        KeyCode::Char('o') => app.open_selected_url(),
        KeyCode::Char('p') if app.view == View::Jobs => app.autofill_selected_job(),
        KeyCode::Char('w') if app.view == View::Jobs => app.record_decision_multi("watching"),
        KeyCode::Char('a') if app.view == View::Jobs => app.record_decision_multi("applied"),
        KeyCode::Char('x') if app.view == View::Jobs => app.record_decision_multi("rejected"),
        KeyCode::Char('i') if app.view == View::Jobs => app.record_decision_multi("interview"),
        KeyCode::Char('s') if app.view == View::Jobs => app.toggle_sort(),
        KeyCode::Char(']') if app.view == View::Jobs => app.jump_next_grade_section(),
        KeyCode::Char('[') if app.view == View::Jobs => app.jump_prev_grade_section(),
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
            app.focus = crate::tui::app::Focus::List;
            app.detail_scroll = 0;
        }
        _ => {}
    }
}
