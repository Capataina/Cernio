use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;

use crate::tui::app::{App, Focus};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Fill(1)]).split(area);

    draw_list(frame, app, chunks[0]);
    draw_detail(frame, app, chunks[1]);
}

fn draw_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = &app.theme;
    let focused = app.focus == Focus::List;

    let title = if let Some(name) = &app.job_filter_company_name {
        format!(" Jobs — {name} ")
    } else {
        " Jobs — All ".to_string()
    };

    if app.jobs.is_empty() {
        let block = Block::bordered()
            .title(title)
            .title_style(t.title)
            .border_style(Style::default().fg(if focused {
                t.border_focused
            } else {
                t.border
            }));
        let msg = if app.job_filter_company.is_some() {
            "  No jobs fetched for this company yet.\n\n  Run a search to populate jobs."
        } else {
            "  No jobs in the database yet.\n\n  Search companies to fetch job listings."
        };
        frame.render_widget(
            Paragraph::new(msg).style(t.dim).block(block),
            area,
        );
        return;
    }

    let header = Row::new(vec![
        Cell::from(" Gr"),
        Cell::from("Title"),
        Cell::from("Company"),
        Cell::from("Location"),
        Cell::from("Decision"),
    ])
    .style(t.header)
    .height(1);

    let rows: Vec<Row> = app
        .jobs
        .iter()
        .map(|j| {
            let grade = j.grade.as_deref().unwrap_or("—");
            let grade_style = t.grade_style(j.grade.as_deref());

            let location = j.location.as_deref().unwrap_or("—");

            let decision_display = match j.decision.as_deref() {
                Some("watching") => "👁 watching",
                Some("applied") => "✓ applied",
                Some("rejected") => "✗ rejected",
                _ => "",
            };
            let decision_style = t.decision_style(j.decision.as_deref());

            Row::new(vec![
                Cell::from(format!(" {grade:<2}")).style(grade_style),
                Cell::from(j.title.as_str()),
                Cell::from(j.company_name.as_str()),
                Cell::from(truncate(location, 16)),
                Cell::from(decision_display).style(decision_style),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(16),
        Constraint::Length(16),
        Constraint::Length(12),
    ];

    let highlight = if focused {
        t.selected
    } else {
        t.selected_unfocused
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::bordered()
                .title(title)
                .title_style(t.title)
                .border_style(Style::default().fg(if focused {
                    t.border_focused
                } else {
                    t.border
                })),
        )
        .row_highlight_style(highlight)
        .highlight_symbol("▸");

    frame.render_stateful_widget(table, area, &mut app.job_state);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let focused = app.focus == Focus::Detail;

    let Some(j) = app.selected_job() else {
        let block = Block::bordered()
            .title(" Detail ")
            .title_style(t.title)
            .border_style(Style::default().fg(t.border));
        frame.render_widget(
            Paragraph::new("  Select a job").style(t.dim).block(block),
            area,
        );
        return;
    };

    let mut lines = Vec::new();

    // Title and company
    lines.push(Line::from(Span::styled(
        format!("  {}", j.title),
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {}", j.company_name),
        t.dim,
    )));
    lines.push(Line::from(""));

    // Details
    lines.push(Line::from(Span::styled("  ── Details ──", t.header)));
    lines.push(Line::from(""));

    if let Some(loc) = &j.location {
        lines.push(detail_row(t, "Location", Span::raw(loc)));
    }
    if let Some(remote) = &j.remote_policy {
        lines.push(detail_row(t, "Remote", Span::raw(remote)));
    }
    if let Some(posted) = &j.posted_date {
        lines.push(detail_row(t, "Posted", Span::raw(posted)));
    }

    let grade = j.grade.as_deref().unwrap_or("—");
    let grade_style = t.grade_style(j.grade.as_deref());
    lines.push(detail_row(t, "Grade", Span::styled(grade, grade_style)));

    let eval_display = match j.evaluation_status.as_str() {
        "strong_fit" => "strong fit",
        "weak_fit" => "weak fit",
        "no_fit" => "no fit",
        other => other,
    };
    let eval_style = t.eval_style(&j.evaluation_status);
    lines.push(detail_row(
        t,
        "Evaluation",
        Span::styled(eval_display, eval_style),
    ));

    if let Some(decision) = &j.decision {
        let dec_style = t.decision_style(Some(decision));
        lines.push(detail_row(
            t,
            "Decision",
            Span::styled(decision, dec_style),
        ));
    }

    if let Some(score) = j.fit_score {
        lines.push(detail_row(
            t,
            "Fit Score",
            Span::raw(format!("{score:.1}")),
        ));
    }

    // Fit assessment
    if let Some(assessment) = &j.fit_assessment {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ── Fit Assessment ──",
            t.header,
        )));
        lines.push(Line::from(""));
        for text_line in assessment.lines() {
            lines.push(Line::from(format!("  {text_line}")));
        }
    }

    // URL
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  ── Link ──", t.header)));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(format!("  {}", j.url), t.dim)));

    let block = Block::bordered()
        .title(format!(" {} ", j.title))
        .title_style(t.title)
        .border_style(Style::default().fg(if focused {
            t.border_focused
        } else {
            t.border
        }));

    let detail = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));

    frame.render_widget(detail, area);
}

fn detail_row<'a>(
    t: &'a super::super::theme::Theme,
    label: &'a str,
    value: Span<'a>,
) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {label:<12}"), t.stat_label),
        value,
    ])
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max - 1).collect();
        format!("{truncated}…")
    }
}
