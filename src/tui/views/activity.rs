use chrono::NaiveDate;
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::Frame;

use crate::data::activity::{group_timeline, ActivityGroup};
use crate::data::models::ActivityEntry;
use crate::tui::app::App;
use crate::tui::theme::lane_badge;

/// Draw the Activity timeline — aggregated event groups from the append-only
/// `activity_events` log. Adjacent same-(event_type, source) events within a
/// time window collapse into a single row showing the count; press Enter to
/// expand and see the individual events.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let entries = &app.activity_timeline;
    let groups = group_timeline(entries);
    let count = groups.len();

    let block = Block::bordered()
        .title(format!(" Activity ({count} entries · {} events) ", entries.len()))
        .title_style(app.theme.title)
        .border_style(Style::default().fg(app.theme.border));

    let t = &app.theme;
    let mut lines: Vec<Line> = Vec::new();
    let mut current_date: Option<String> = None;

    // Track per-row index for cursor highlighting (rows = group headers + expanded children).
    let mut row_idx = 0usize;

    for group in &groups {
        let date = group.date().to_string();
        if current_date.as_deref() != Some(date.as_str()) {
            current_date = Some(date.clone());
            let formatted = format_date(&date);
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                format!("── {formatted} ──"),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        }

        let cursor_here = row_idx == app.activity_cursor;
        let base_style = if cursor_here {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        match group {
            ActivityGroup::Single(entry) => {
                lines.push(render_entry_line(entry, t, false, base_style));
                row_idx += 1;
            }
            ActivityGroup::Collapsed {
                event_type,
                source,
                first_at,
                last_at,
                entries,
                ..
            } => {
                let key = group.key();
                let expanded = app.activity_expanded.contains(&key);
                let arrow = if expanded { "▾" } else { "▸" };

                let (icon, icon_style) = icon_for_event(event_type, t);
                let label = event_label(event_type);
                let time_first = if first_at.len() >= 16 { &first_at[11..16] } else { "" };
                let time_last = if last_at.len() >= 16 { &last_at[11..16] } else { "" };
                let time_str = if time_first == time_last {
                    time_first.to_string()
                } else {
                    format!("{time_first}–{time_last}")
                };
                let source_str = if source != "tui" {
                    format!("  · {source}")
                } else {
                    String::new()
                };

                let mut spans = vec![
                    Span::styled(format!("  {arrow} "), if expanded { t.stat_value } else { t.dim }),
                    Span::styled(format!("{time_str:<11} "), t.dim),
                    Span::styled(format!("{icon} "), icon_style),
                    Span::styled(
                        format!("{} {label}", entries.len()),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(source_str, t.dim),
                ];
                if cursor_here {
                    for s in &mut spans { s.style = s.style.add_modifier(Modifier::REVERSED); }
                }
                lines.push(Line::from(spans));
                row_idx += 1;

                if expanded {
                    for entry in entries {
                        lines.push(render_entry_line(entry, t, true, Style::default()));
                    }
                }
            }
        }
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

/// Group count at the top-level (used by handlers to clamp cursor).
pub fn group_count(app: &App) -> usize {
    group_timeline(&app.activity_timeline).len()
}

/// Look up the group at `index` (top-level group rows only — children of
/// expanded groups don't count). Used by Enter to toggle expansion.
pub fn group_at(app: &App, index: usize) -> Option<ActivityGroup> {
    group_timeline(&app.activity_timeline).into_iter().nth(index)
}

fn render_entry_line<'a>(
    entry: &'a ActivityEntry,
    t: &'a crate::tui::theme::Theme,
    is_child: bool,
    base_style: Style,
) -> Line<'a> {
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

    let indent = if is_child { "      " } else { "  " };
    let mut spans = vec![
        Span::styled(indent.to_string(), Style::default()),
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

    if entry.source != "tui" {
        spans.push(Span::styled(format!("  · {}", entry.source), t.dim));
    }

    if base_style != Style::default() {
        for s in &mut spans { s.style = s.style.patch(base_style); }
    }

    Line::from(spans)
}

fn icon_for_event(event_type: &str, t: &crate::tui::theme::Theme) -> (&'static str, Style) {
    match event_type {
        "job.added" => ("+", Style::default().fg(Color::Green)),
        "job.deleted" | "job.pruned" => ("−", Style::default().fg(Color::Red)),
        "job.archived" => ("□", t.dim),
        "job.graded" => ("★", Style::default().fg(Color::Yellow)),
        "job.regraded" => ("↺", Style::default().fg(Color::Yellow)),
        "company.added" => ("+", Style::default().fg(Color::Cyan)),
        "company.deleted" => ("−", Style::default().fg(Color::Red)),
        "company.archived" => ("□", t.dim),
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
        "raw.job.inserted" => "raw insert (job)".into(),
        "raw.job.deleted" => "raw delete (job)".into(),
        "raw.company.inserted" => "raw insert (company)".into(),
        "raw.company.deleted" => "raw delete (company)".into(),
        other => other.to_string(),
    }
}

fn format_date(date_str: &str) -> String {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map(|d| d.format("%b %-d, %Y").to_string())
        .unwrap_or_else(|_| date_str.to_string())
}
