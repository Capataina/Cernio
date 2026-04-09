use chrono::NaiveDate;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::app::App;

/// Draw the Activity timeline view — a scrollable chronological feed.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let entries = &app.activity_timeline;
    let count = entries.len();

    let block = Block::bordered()
        .title(format!(" Activity ({count} entries) "))
        .title_style(app.theme.title)
        .border_style(Style::default().fg(app.theme.border));

    let mut lines: Vec<Line> = Vec::new();
    let mut current_date: Option<&str> = None;

    for entry in entries {
        // Date header when the date changes.
        if current_date != Some(&entry.date) {
            current_date = Some(&entry.date);

            // Format date nicely: "Apr 9, 2026"
            let formatted = format_date(&entry.date);
            let header = format!("── {formatted} ──");

            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                header,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        }

        // Action icon + label + detail.
        let (icon, label, style) = match entry.action.as_str() {
            "applied" => ("✓", "Applied", Style::default().fg(Color::Green)),
            "watching" => ("◉", "Watching", Style::default().fg(Color::Cyan)),
            "rejected" => ("✗", "Rejected", Style::default().fg(Color::DarkGray)),
            "searched" => ("⟳", "Searched", Style::default().fg(Color::Cyan)),
            "graded" => ("★", "Graded", Style::default().fg(Color::Yellow)),
            "discovered" => ("●", "Discovered", Style::default().fg(Color::DarkGray)),
            other => ("·", other, Style::default().fg(Color::DarkGray)),
        };

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{icon} {label:<12}"), style),
            Span::raw(&entry.detail),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No activity recorded yet.",
            app.theme.dim,
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.activity_scroll, 0));

    frame.render_widget(paragraph, area);
}

/// Format "2026-04-09" into "Apr 9, 2026".
fn format_date(date_str: &str) -> String {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map(|d| d.format("%b %-d, %Y").to_string())
        .unwrap_or_else(|_| date_str.to_string())
}
