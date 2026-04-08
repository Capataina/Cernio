use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::{App, Focus, View};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Help overlay swallows all keys — any press dismisses it.
    if app.show_help {
        app.show_help = false;
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

fn handle_detail_scroll(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => app.scroll_detail_down(),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_detail_up(),
        KeyCode::Char('o') => app.open_selected_url(),
        // Actions still work from detail focus in jobs view.
        KeyCode::Char('w') if app.view == View::Jobs => app.record_decision("watching"),
        KeyCode::Char('a') if app.view == View::Jobs => app.record_decision("applied"),
        KeyCode::Char('x') if app.view == View::Jobs => app.record_decision("rejected"),
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
            app.focus = Focus::List;
            app.detail_scroll = 0;
        }
        _ => {}
    }
}
