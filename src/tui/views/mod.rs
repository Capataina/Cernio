pub mod chrome;
pub mod overlays;
mod activity;
mod companies;
mod dashboard;
mod jobs;
mod pipeline;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
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

    // Quick-peek popup for jobs view.
    if app.show_quick_peek && app.view == View::Jobs {
        if let Some(job) = app.selected_job() {
            draw_quick_peek(frame, app, job, frame_area);
        }
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

fn draw_quick_peek(
    frame: &mut Frame,
    app: &App,
    job: &super::app::state::JobRow,
    frame_area: Rect,
) {
    let t = &app.theme;

    // Centred popup: 50% width, 60% height.
    let popup_w = frame_area.width * 50 / 100;
    let popup_h = frame_area.height * 60 / 100;
    let x = frame_area.x + (frame_area.width.saturating_sub(popup_w)) / 2;
    let y = frame_area.y + (frame_area.height.saturating_sub(popup_h)) / 2;
    let area = Rect::new(x, y, popup_w.min(frame_area.width), popup_h.min(frame_area.height));

    let mut lines = Vec::new();

    // Title (bold).
    lines.push(Line::from(Span::styled(
        format!("  {}", job.title),
        Style::default().add_modifier(Modifier::BOLD),
    )));

    // Company (dim).
    lines.push(Line::from(Span::styled(
        format!("  {}", job.company_name),
        t.dim,
    )));

    // Grade (coloured).
    let grade_str = job.grade.as_deref().unwrap_or("—");
    let grade_style = t.grade_style(job.grade.as_deref());
    lines.push(Line::from(vec![
        Span::styled("  Grade: ", t.stat_label),
        Span::styled(grade_str, grade_style),
    ]));

    // Location if present.
    if let Some(loc) = &job.location {
        lines.push(Line::from(vec![
            Span::styled("  Location: ", t.stat_label),
            Span::raw(loc.as_str()),
        ]));
    }

    lines.push(Line::from(""));

    // Fit assessment (wrapped).
    if let Some(assessment) = &job.fit_assessment {
        for text_line in assessment.lines() {
            lines.push(Line::from(format!("  {text_line}")));
        }
    } else {
        lines.push(Line::from(Span::styled("  No fit assessment yet.", t.dim)));
    }

    let block = Block::bordered()
        .title(" Quick Peek ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border_focused));

    let popup = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}
