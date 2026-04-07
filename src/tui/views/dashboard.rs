use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::horizontal([Constraint::Percentage(45), Constraint::Fill(1)]).split(area);

    // Left column: universe + ATS coverage
    let left = Layout::vertical([Constraint::Min(10), Constraint::Fill(1)]).split(cols[0]);
    draw_universe(frame, app, left[0]);
    draw_ats_coverage(frame, app, left[1]);

    // Right column: jobs + top matches
    let right = Layout::vertical([Constraint::Min(10), Constraint::Fill(1)]).split(cols[1]);
    draw_jobs_summary(frame, app, right[0]);
    draw_top_matches(frame, app, right[1]);
}

fn draw_universe(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let mut lines = vec![
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{}", s.total_companies), t.stat_value),
            Span::raw(" companies in universe"),
        ]),
        Line::from(""),
        Line::from(Span::styled("  By Grade", t.header)),
    ];

    for (grade, count) in &s.companies_by_grade {
        let style = t.grade_style(Some(grade.as_str()));
        let bar = "█".repeat((*count as usize).min(20));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("{grade:<3}"), style),
            Span::styled(format!(" {bar} "), style),
            Span::styled(format!("{count}"), t.stat_value),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  By Status", t.header)));

    for (status, count) in &s.companies_by_status {
        let style = t.status_style(status);
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("{status:<12}"), style),
            Span::styled(format!("{count}"), t.stat_value),
        ]));
    }

    let block = Block::bordered()
        .title(" Universe ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}

fn draw_ats_coverage(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let mut lines = Vec::new();

    if s.ats_coverage.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No ATS portals resolved yet",
            t.dim,
        )));
    } else {
        for (provider, count) in &s.ats_coverage {
            let label = if *count == 1 { "company" } else { "companies" };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{provider:<16}"), t.stat_value),
                Span::styled(format!("{count} {label}"), t.dim),
            ]));
        }
    }

    if s.bespoke_count > 0 {
        let label = if s.bespoke_count == 1 {
            "company"
        } else {
            "companies"
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<16}", "bespoke"), t.stat_value),
            Span::styled(format!("{} {label}", s.bespoke_count), t.dim),
        ]));
    }

    let block = Block::bordered()
        .title(" ATS Coverage ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}

fn draw_jobs_summary(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let mut lines = vec![
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{}", s.total_jobs), t.stat_value),
            Span::raw(" total jobs"),
        ]),
        Line::from(""),
        Line::from(Span::styled("  By Grade", t.header)),
    ];

    for (grade, count) in &s.jobs_by_grade {
        let style = t.grade_style(Some(grade.as_str()));
        let bar = "█".repeat((*count as usize).min(20));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("{grade:<3}"), style),
            Span::styled(format!(" {bar} "), style),
            Span::styled(format!("{count}"), t.stat_value),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  By Evaluation", t.header)));

    for (status, count) in &s.jobs_by_eval {
        let display = match status.as_str() {
            "strong_fit" => "strong fit",
            "weak_fit" => "weak fit",
            "no_fit" => "no fit",
            other => other,
        };
        let style = t.eval_style(status);
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(format!("{display:<14}"), style),
            Span::styled(format!("{count}"), t.stat_value),
        ]));
    }

    let block = Block::bordered()
        .title(" Jobs ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}

fn draw_top_matches(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let mut lines = Vec::new();

    if s.top_matches.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No graded jobs yet",
            t.dim,
        )));
    } else {
        for (i, m) in s.top_matches.iter().enumerate() {
            let grade_str = m.grade.as_deref().unwrap_or("—");
            let style = t.grade_style(m.grade.as_deref());
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{:<3}", i + 1), t.dim),
                Span::styled(format!("{grade_str:<3}"), style),
                Span::raw(format!("{} — {}", m.title, m.company)),
            ]));
        }
    }

    let block = Block::bordered()
        .title(" Top Matches ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}
