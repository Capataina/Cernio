pub mod chrome;
pub mod overlays;
mod companies;
mod dashboard;
mod jobs;
mod pipeline;

use ratatui::layout::{Constraint, Layout};
use ratatui::Frame;

use super::app::{App, View};

/// Main draw entry point — renders tabs, content area, status bar, and overlays.
pub fn draw(frame: &mut Frame, app: &mut App) {
    // Store terminal dimensions for responsive layout.
    let frame_area = frame.area();
    app.terminal_width = frame_area.width;
    app.terminal_height = frame_area.height;

    let bottom_height = if app.search_mode { 2 } else { 1 };

    let areas = Layout::vertical([
        Constraint::Length(3),       // tab bar
        Constraint::Fill(1),         // content
        Constraint::Length(bottom_height), // status bar (+ search bar)
    ])
    .split(frame_area);

    chrome::draw_tabs(frame, app, areas[0]);

    match app.view {
        View::Dashboard => dashboard::draw(frame, app, areas[1]),
        View::Companies => companies::draw(frame, app, areas[1]),
        View::Jobs => jobs::draw(frame, app, areas[1]),
        View::Pipeline => pipeline::draw(frame, app, areas[1]),
    }

    if app.search_mode {
        let bottom = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(areas[2]);
        chrome::draw_search_bar(frame, app, bottom[0]);
        chrome::draw_status_bar(frame, app, bottom[1]);
    } else {
        chrome::draw_status_bar(frame, app, areas[2]);
    }

    overlays::draw_toasts(frame, app);

    if app.show_grade_picker {
        overlays::draw_grade_picker(frame, app);
    }

    if app.show_bulk_picker {
        overlays::draw_bulk_picker(frame, app);
    }

    if app.show_help {
        overlays::draw_help_overlay(frame, app);
    }
}
