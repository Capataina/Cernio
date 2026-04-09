pub mod navigation;
pub mod overlays;
pub mod mouse;

use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseEvent,
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
        overlays::handle_grade_picker(app, key);
        return;
    }

    // Bulk action picker — pick a grade to apply bulk action.
    if app.show_bulk_picker {
        overlays::handle_bulk_picker(app, key);
        return;
    }

    // Search mode — route all input to the search handler.
    if app.search_mode {
        overlays::handle_search_input(app, key);
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
        KeyCode::Char('4') => {
            app.view = View::Pipeline;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('f') => {
            if !matches!(app.view, View::Dashboard | View::Pipeline) {
                app.focused_mode = !app.focused_mode;
                app.hide_applied = app.focused_mode;
                app.add_toast(if app.focused_mode {
                    "Focused: hiding F/C + applied".to_string()
                } else {
                    "Showing all grades + applied".to_string()
                });
                app.refresh();
            }
            return;
        }
        KeyCode::Char('A') => {
            if !matches!(app.view, View::Dashboard) {
                app.show_archived = !app.show_archived;
                app.add_toast(if app.show_archived {
                    "Showing archived".to_string()
                } else {
                    "Hiding archived".to_string()
                });
                app.refresh();
            }
            return;
        }
        KeyCode::Tab => {
            if matches!(app.view, View::Companies | View::Jobs) {
                app.focus = match app.focus {
                    Focus::List => Focus::Detail,
                    Focus::Detail => Focus::List,
                };
                app.detail_scroll = 0;
            }
            return;
        }
        KeyCode::Char('/') => {
            if !matches!(app.view, View::Pipeline) {
                app.search_mode = true;
                app.search_query.clear();
            }
            return;
        }
        KeyCode::Char('y') => {
            app.copy_url_to_clipboard();
            app.add_toast("URL copied".to_string());
            return;
        }
        KeyCode::Char('e') => {
            app.export_current_view();
            return;
        }
        _ => {}
    }

    // ── View + focus–specific keys ───────────────────────────────

    match (app.view, app.focus) {
        (View::Dashboard, _) => navigation::handle_dashboard(app, key),
        (View::Companies, Focus::List) => navigation::handle_company_list(app, key),
        (View::Companies, Focus::Detail) => navigation::handle_detail_scroll(app, key),
        (View::Jobs, Focus::List) => navigation::handle_job_list(app, key),
        (View::Jobs, Focus::Detail) => navigation::handle_detail_scroll(app, key),
        (View::Pipeline, _) => navigation::handle_pipeline(app, key),
    }
}

pub fn handle_mouse(app: &mut App, event: MouseEvent) {
    mouse::handle_mouse(app, event);
}
