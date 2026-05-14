pub mod navigation;
pub mod overlays;
pub mod mouse;

use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseEvent,
};

use super::app::{App, Focus, View};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Telemetry: every keystroke logs with a full state snapshot.
    // This is the primary trace input — replaying the keystroke sequence
    // against the snapshots reproduces any reported issue.
    crate::tel!(
        "key_press",
        "code": format!("{:?}", key.code),
        "modifiers": format!("{:?}", key.modifiers),
        "state": app.snapshot(),
    );

    // Help overlay swallows all keys — any press dismisses it.
    if app.show_help {
        crate::tel!("help_dismissed");
        app.show_help = false;
        return;
    }

    // Quick-peek popup — dismiss on any key except Space (which toggles it).
    if app.show_quick_peek && key.code != KeyCode::Char(' ') && key.code != KeyCode::Esc {
        app.show_quick_peek = false;
        // Fall through to normal handling so the key still works.
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

    // Filter menu — multi-axis toggle picker.
    if app.show_filter_menu {
        overlays::handle_filter_menu(app, key);
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
            crate::tel!("quit_requested", "trigger": "q");
            app.running = false;
            return;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            crate::tel!("quit_requested", "trigger": "ctrl_c");
            app.running = false;
            return;
        }
        KeyCode::Char('?') => {
            crate::tel!("help_open");
            app.show_help = true;
            return;
        }
        KeyCode::Char('1') => {
            crate::tel!("view_change", "from": format!("{:?}", app.view), "to": "Dashboard", "trigger": "key_1");
            app.view = View::Dashboard;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('2') => {
            crate::tel!("view_change", "from": format!("{:?}", app.view), "to": "Companies", "trigger": "key_2");
            app.view = View::Companies;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('3') => {
            crate::tel!("view_change", "from": format!("{:?}", app.view), "to": "Jobs", "trigger": "key_3");
            app.view = View::Jobs;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('4') => {
            crate::tel!("view_change", "from": format!("{:?}", app.view), "to": "Pipeline", "trigger": "key_4");
            app.view = View::Pipeline;
            app.focus = Focus::List;
            app.detail_scroll = 0;
            return;
        }
        KeyCode::Char('5') => {
            app.view = View::Activity;
            app.focus = Focus::List;
            app.activity_scroll = 0;
            return;
        }
        KeyCode::Char('f') => {
            if !matches!(app.view, View::Dashboard | View::Pipeline | View::Activity) {
                crate::tel!("filter_menu_open", "view": format!("{:?}", app.view));
                app.show_filter_menu = true;
                app.filter_menu_axis = 0;
                app.filter_menu_chip = 0;
            } else {
                crate::tel!("filter_menu_open_blocked_view", "view": format!("{:?}", app.view));
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
            if !matches!(app.view, View::Pipeline | View::Activity) {
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
        (View::Activity, _) => navigation::handle_activity(app, key),
    }
}

pub fn handle_mouse(app: &mut App, event: MouseEvent) {
    mouse::handle_mouse(app, event);
}
