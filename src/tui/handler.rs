use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use super::app::{App, Focus, View};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Help overlay swallows all keys — any press dismisses it.
    if app.show_help {
        app.show_help = false;
        return;
    }

    // Grade picker overlay — single-key selection.
    if app.show_grade_picker {
        handle_grade_picker(app, key);
        return;
    }

    // Search mode — route all input to the search handler.
    if app.search_mode {
        handle_search_input(app, key);
        return;
    }

    // ── Global keys (work in every view) ─────────────────────────

    match key.code {
        KeyCode::Char('q') => {
            app.running = false;
            return;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
            return;
        }
        KeyCode::Char('?') => {
            app.show_help = true;
            return;
        }
        KeyCode::Char('1') => {
            app.view = View::Dashboard;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('2') => {
            app.view = View::Companies;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('3') => {
            app.view = View::Jobs;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('f') => {
            if !matches!(app.view, View::Dashboard) {
                app.focused_mode = !app.focused_mode;
                app.add_toast(if app.focused_mode {
                    "Focused mode ON — hiding F/C grades".to_string()
                } else {
                    "Focused mode OFF — showing all grades".to_string()
                });
                app.refresh();
            }
            return;
        }
        KeyCode::Tab => {
            if !matches!(app.view, View::Dashboard) {
                app.focus = match app.focus {
                    Focus::List => Focus::Detail,
                    Focus::Detail => Focus::List,
                };
                app.detail_scroll = 0;
            }
            return;
        }
        KeyCode::Char('/') => {
            app.search_mode = true;
            app.search_query.clear();
            return;
        }
        KeyCode::Char('y') => {
            app.copy_url_to_clipboard();
            app.add_toast("URL copied".to_string());
            return;
        }
        _ => {}
    }

    // ── View + focus–specific keys ───────────────────────────────

    match (app.view, app.focus) {
        (View::Dashboard, _) => handle_dashboard(app, key),
        (View::Companies, Focus::List) => handle_company_list(app, key),
        (View::Companies, Focus::Detail) => handle_detail_scroll(app, key),
        (View::Jobs, Focus::List) => handle_job_list(app, key),
        (View::Jobs, Focus::Detail) => handle_detail_scroll(app, key),
    }
}

fn handle_grade_picker(app: &mut App, key: KeyEvent) {
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

fn handle_search_input(app: &mut App, key: KeyEvent) {
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

fn handle_dashboard(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('D') => app.run_cleanup(),
        _ => {}
    }
}

fn handle_company_list(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.next_in_list(),
        KeyCode::Char('k') | KeyCode::Up => app.prev_in_list(),
        KeyCode::Char('g') => app.go_to_top(),
        KeyCode::Char('G') => app.go_to_bottom(),
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => app.enter_company_jobs(),
        KeyCode::Char('o') => app.open_selected_url(),
        _ => {}
    }
}

fn handle_job_list(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.next_in_list(),
        KeyCode::Char('k') | KeyCode::Up => app.prev_in_list(),
        KeyCode::Char('g') => app.go_to_top(),
        KeyCode::Char('G') => app.go_to_bottom(),
        KeyCode::Char('o') => app.open_selected_url(),
        KeyCode::Char('w') => app.record_decision("watching"),
        KeyCode::Char('a') => app.record_decision("applied"),
        KeyCode::Char('x') => app.record_decision("rejected"),
        KeyCode::Char('s') => app.toggle_sort(),
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
            if app.job_filter_company.is_some() {
                app.clear_job_filter();
                app.view = View::Companies;
                app.detail_scroll = 0;
            }
        }
        _ => {}
    }
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    if app.show_help || app.show_grade_picker {
        return;
    }

    match mouse.kind {
        MouseEventKind::ScrollDown => {
            // Scroll works regardless of focus — detect pane from column.
            let area_width = 80u16; // reasonable default; real width comes from frame
            let in_left_pane = mouse.column < area_width / 2;

            match app.view {
                View::Dashboard => {
                    app.scroll_viewport_down(3);
                }
                _ => {
                    if in_left_pane || app.focus == Focus::List {
                        app.next_in_list();
                        app.next_in_list();
                        app.next_in_list();
                    } else {
                        app.scroll_detail_down();
                        app.scroll_detail_down();
                        app.scroll_detail_down();
                    }
                }
            }
        }
        MouseEventKind::ScrollUp => {
            let area_width = 80u16;
            let in_left_pane = mouse.column < area_width / 2;

            match app.view {
                View::Dashboard => {
                    app.scroll_viewport_up(3);
                }
                _ => {
                    if in_left_pane || app.focus == Focus::List {
                        app.prev_in_list();
                        app.prev_in_list();
                        app.prev_in_list();
                    } else {
                        app.scroll_detail_up();
                        app.scroll_detail_up();
                        app.scroll_detail_up();
                    }
                }
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            // Click on tab bar (rows 0–2).
            if mouse.row < 3 {
                // Rough tab detection from column position.
                let col = mouse.column;
                if col < 15 {
                    app.view = View::Dashboard;
                    app.focus = Focus::List;
                } else if col < 35 {
                    app.view = View::Companies;
                    app.focus = Focus::List;
                } else {
                    app.view = View::Jobs;
                    app.focus = Focus::List;
                }
                app.detail_scroll = 0;
                return;
            }

            // Below tab bar — detect pane from column.
            if matches!(app.view, View::Dashboard) {
                return;
            }

            let area_width = 80u16;
            let in_left_pane = mouse.column < area_width / 2;

            if in_left_pane {
                app.focus = Focus::List;
                // Calculate row index: subtract header offset (tab bar + table header).
                let header_offset = 5u16; // tab(3) + table header(2)
                if mouse.row >= header_offset {
                    let clicked_row = (mouse.row - header_offset) as usize;
                    let len = match app.view {
                        View::Companies => app.companies.len(),
                        View::Jobs => app.jobs.len(),
                        View::Dashboard => 0,
                    };
                    if clicked_row < len {
                        match app.view {
                            View::Companies => app.company_state.select(Some(clicked_row)),
                            View::Jobs => app.job_state.select(Some(clicked_row)),
                            View::Dashboard => {}
                        }
                        app.detail_scroll = 0;
                    }
                }
            } else {
                app.focus = Focus::Detail;
            }
        }
        _ => {}
    }
}

fn handle_detail_scroll(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.scroll_detail_down(),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_detail_up(),
        KeyCode::Char('o') => app.open_selected_url(),
        // Actions still work from detail focus in jobs view.
        KeyCode::Char('w') if app.view == View::Jobs => app.record_decision("watching"),
        KeyCode::Char('a') if app.view == View::Jobs => app.record_decision("applied"),
        KeyCode::Char('x') if app.view == View::Jobs => app.record_decision("rejected"),
        KeyCode::Char('s') if app.view == View::Jobs => app.toggle_sort(),
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
            app.focus = Focus::List;
            app.detail_scroll = 0;
        }
        _ => {}
    }
}
