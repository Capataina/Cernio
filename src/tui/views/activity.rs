use chrono::NaiveDate;
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::Frame;

use crate::tui::app::App;
use crate::tui::theme::lane_badge;

/// Draw the Activity timeline — every event from `activity_events`, grouped
/// by day, lane-coloured, with event-type icons. The data is the append-only
/// event log written at every DB mutation, so events for deleted rows still
/// render with their cached label/lane/grade.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let entries = &app.activity_timeline;
    let count = entries.len();

    let block = Block::bordered()
        .title(format!(" Activity ({count} events) "))
        .title_style(app.theme.title)
        .border_style(Style::default().fg(app.theme.border));

    let t = &app.theme;
    let mut lines: Vec<Line> = Vec::new();
    let mut current_date: Option<&str> = None;

    for entry in entries {
        if current_date != Some(&entry.date) {
            current_date = Some(&entry.date);
            let formatted = format_date(&entry.date);
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                format!("── {formatted} ──"),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        }

        // Per-event row: time · lane badge · icon · label · grade · source
        let time = if entry.occurred_at.len() >= 16 {
            &entry.occurred_at[11..16]
        } else {
            ""
        };

        let (icon, icon_style) = icon_for_event(&entry.event_type, t);
        let lane_str = match entry.lane.as_deref() {
            Some(key) => format!("{} ", lane_badge(key)),
            None => "—   ".to_string(),
        };
        let lane_style = match entry.lane.as_deref() {
            Some(key) => t.lane_style(key),
            None => t.dim,
        };

        let mut spans = vec![
            Span::raw("  "),
            Span::styled(format!("{time} "), t.dim),
            Span::styled(lane_str, lane_style),
            Span::styled(format!("{icon} "), icon_style),
            Span::styled(format!("{:<22}", event_label(&entry.event_type)), icon_style),
            Span::raw(" "),
        ];

        if !entry.subject_label.is_empty() {
            spans.push(Span::raw(entry.subject_label.clone()));
        }

        if let Some(grade) = entry.grade.as_deref() {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(format!("[{grade}]"), t.grade_style(Some(grade))));
        }

        // Source tag — only show if non-default (i.e. not 'tui' which is most common).
        if entry.source != "tui" {
            spans.push(Span::styled(format!("  · {}", entry.source), t.dim));
        }

        lines.push(Line::from(spans));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No activity recorded yet.",
            app.theme.dim,
        )));
    }

    let line_count = lines.len();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.activity_scroll, 0));

    frame.render_widget(paragraph, area);

    if line_count > 0 {
        let mut scrollbar_state = ScrollbarState::new(line_count)
            .position(app.activity_scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin { vertical: 1, horizontal: 0 }),
            &mut scrollbar_state,
        );
    }
}

/// Map event_type to icon + style.
///
/// raw.* events come from the trigger backstop (mutation paths that bypassed
/// the Rust-side emit helpers) and are rendered with a `?` to surface the
/// coverage gap.
fn icon_for_event(event_type: &str, t: &crate::tui::theme::Theme) -> (&'static str, Style) {
    match event_type {
        "job.added" => ("+", Style::default().fg(Color::Green)),
        "job.deleted" | "job.pruned" => ("−", Style::default().fg(Color::Red)),
        "job.archived" => ("🗄", t.dim),
        "job.graded" => ("★", Style::default().fg(Color::Yellow)),
        "job.regraded" => ("↺", Style::default().fg(Color::Yellow)),
        "company.added" => ("+", Style::default().fg(Color::Cyan)),
        "company.deleted" => ("−", Style::default().fg(Color::Red)),
        "company.archived" => ("🗄", t.dim),
        "company.unarchived" => ("↻", Style::default().fg(Color::Cyan)),
        "company.graded" => ("★", Style::default().fg(Color::Yellow)),
        "decision.applied" => ("✓", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        "decision.watching" => ("◉", Style::default().fg(Color::Cyan)),
        "decision.rejected" => ("✗", t.dim),
        "decision.interview" => ("→", Style::default().fg(Color::Magenta)),
        "search.ran" => ("⟳", Style::default().fg(Color::Cyan)),
        t_str if t_str.starts_with("raw.") => ("?", Style::default().fg(Color::Yellow)),
        _ => ("·", t.dim),
    }
}

/// Short human label for the event type column.
fn event_label(event_type: &str) -> String {
    match event_type {
        "job.added" => "job added".into(),
        "job.deleted" => "job deleted".into(),
        "job.pruned" => "job pruned".into(),
        "job.archived" => "job archived".into(),
        "job.graded" => "job graded".into(),
        "job.regraded" => "job regraded".into(),
        "company.added" => "company added".into(),
        "company.deleted" => "company deleted".into(),
        "company.archived" => "company archived".into(),
        "company.unarchived" => "company unarchived".into(),
        "company.graded" => "company graded".into(),
        "decision.applied" => "applied".into(),
        "decision.watching" => "watching".into(),
        "decision.rejected" => "rejected".into(),
        "decision.interview" => "interview".into(),
        "search.ran" => "search".into(),
        other => other.to_string(),
    }
}

fn format_date(date_str: &str) -> String {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map(|d| d.format("%b %-d, %Y").to_string())
        .unwrap_or_else(|_| date_str.to_string())
}
