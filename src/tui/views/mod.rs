pub mod chrome;
pub mod overlays;
mod activity;
mod companies;
mod dashboard;
mod jobs;
mod pipeline;

use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::app::{App, View};

/// Main draw entry point — renders tabs, content area, status bar, and overlays.
pub fn draw(frame: &mut Frame, app: &mut App) {
    // Store terminal dimensions for responsive layout.
    let frame_area = frame.area();
    app.terminal_width = frame_area.width;
    app.terminal_height = frame_area.height;

    let bottom_height = if app.search_mode { 2 } else { 1 };

    // Breadcrumb visible when drilling down from companies to jobs.
    let has_breadcrumb = app.job_filter_company_name.is_some();
    let breadcrumb_height = if has_breadcrumb { 1 } else { 0 };

    let areas = Layout::vertical([
        Constraint::Length(3),              // tab bar
        Constraint::Length(breadcrumb_height), // breadcrumb (0 or 1)
        Constraint::Fill(1),               // content
        Constraint::Length(bottom_height),  // status bar (+ search bar)
    ])
    .split(frame_area);

    chrome::draw_tabs(frame, app, areas[0]);

    // Breadcrumb trail.
    if has_breadcrumb {
        let company_name = app.job_filter_company_name.as_deref().unwrap_or("");
        let crumb = Paragraph::new(Line::from(vec![
            Span::styled("  Companies", app.theme.dim),
            Span::styled(" › ", app.theme.dim),
            Span::styled(company_name, Style::default().fg(ratatui::style::Color::Cyan)),
            Span::styled(" › ", app.theme.dim),
            Span::styled("Jobs", app.theme.dim),
        ]));
        frame.render_widget(crumb, areas[1]);
    }

    let content_area = areas[2]; // after tabs + breadcrumb
    let status_area = areas[3];

    match app.view {
        View::Dashboard => dashboard::draw(frame, app, content_area),
        View::Companies => companies::draw(frame, app, content_area),
        View::Jobs => jobs::draw(frame, app, content_area),
        View::Pipeline => pipeline::draw(frame, app, content_area),
        View::Activity => activity::draw(frame, app, content_area),
    }

    if app.search_mode {
        let bottom = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(status_area);
        chrome::draw_search_bar(frame, app, bottom[0]);
        chrome::draw_status_bar(frame, app, bottom[1]);
    } else {
        chrome::draw_status_bar(frame, app, status_area);
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
