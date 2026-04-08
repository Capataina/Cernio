use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::vertical([
        Constraint::Length(3), // at-a-glance summary
        Constraint::Fill(1),  // stats grid
    ])
    .split(area);

    draw_summary_line(frame, app, rows[0]);

    let cols = Layout::horizontal([Constraint::Percentage(45), Constraint::Fill(1)]).split(rows[1]);

    // Left column: grade distribution + pipeline health
    let left = Layout::vertical([Constraint::Min(12), Constraint::Fill(1)]).split(cols[0]);
    draw_grade_distribution(frame, app, left[0]);
    draw_pipeline_health(frame, app, left[1]);

    // Right column: action items + top roles
    let right = Layout::vertical([Constraint::Min(12), Constraint::Fill(1)]).split(cols[1]);
    draw_action_items(frame, app, right[0]);
    draw_top_roles(frame, app, right[1]);
}

fn draw_summary_line(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    // Count strong fits.
    let strong: i64 = s
        .jobs_by_grade
        .iter()
        .filter(|(g, _)| g == "SS" || g == "S")
        .map(|(_, c)| c)
        .sum();

    let pending: i64 = s
        .jobs_by_eval
        .iter()
        .filter(|(e, _)| e == "pending")
        .map(|(_, c)| c)
        .sum();

    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{}", s.total_companies), t.stat_value),
        Span::raw(" companies · "),
        Span::styled(format!("{}", s.total_jobs), t.stat_value),
        Span::raw(" jobs · "),
        Span::styled(format!("{strong}"), t.grade_s),
        Span::raw(" strong matches · "),
        Span::styled(format!("{pending}"), if pending > 0 { t.eval_evaluating } else { t.dim }),
        Span::raw(" pending"),
    ]);

    let block = Block::bordered()
        .border_style(Style::default().fg(t.border));

    frame.render_widget(Paragraph::new(line).block(block), area);
}

/// Render proportional bar using `█` characters scaled to the available width.
/// `max_val` is the largest count in the group; bars are scaled relative to it.
/// `max_bar_width` is the maximum number of `█` characters for the largest bar.
fn proportional_bar(count: i64, max_val: i64, max_bar_width: usize) -> String {
    if max_val == 0 || count == 0 {
        return String::new();
    }
    let width = ((count as f64 / max_val as f64) * max_bar_width as f64)
        .ceil() as usize;
    let width = width.max(1); // at least one block if count > 0
    "█".repeat(width)
}

fn draw_grade_distribution(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let mut lines = vec![
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Companies", t.header),
            Span::raw("           "),
            Span::styled("Jobs", t.header),
        ]),
    ];

    // Determine max values for proportional bars.
    let max_company = s.companies_by_grade.iter().map(|(_, c)| *c).max().unwrap_or(1);
    let max_job = s.jobs_by_grade.iter().map(|(_, c)| *c).max().unwrap_or(1);

    // Build a unified grade list. Company grades are S/A/B/C; job grades are SS/S/A/B/C/F.
    let company_grades = ["S", "A", "B", "C"];
    let job_grades = ["SS", "S", "A", "B", "C", "F"];

    // We want to show max(company_grades.len(), job_grades.len()) rows.
    let max_rows = company_grades.len().max(job_grades.len());

    for i in 0..max_rows {
        let mut spans = Vec::new();
        spans.push(Span::raw("  "));

        // Company side
        if i < company_grades.len() {
            let grade = company_grades[i];
            let count = s.companies_by_grade.iter()
                .find(|(g, _)| g == grade)
                .map(|(_, c)| *c)
                .unwrap_or(0);
            let style = t.grade_style(Some(grade));
            let bar = proportional_bar(count, max_company, 6);
            spans.push(Span::styled(format!("{grade:<3}"), style));
            spans.push(Span::styled(format!("{bar:<7}"), style));
            spans.push(Span::styled(format!("{count:<4}"), t.stat_value));
        } else {
            spans.push(Span::raw("              "));
        }

        spans.push(Span::raw("  "));

        // Job side
        if i < job_grades.len() {
            let grade = job_grades[i];
            let count = s.jobs_by_grade.iter()
                .find(|(g, _)| g == grade)
                .map(|(_, c)| *c)
                .unwrap_or(0);
            let style = t.grade_style(Some(grade));
            let bar = proportional_bar(count, max_job, 6);
            spans.push(Span::styled(format!("{grade:<3}"), style));
            spans.push(Span::styled(format!("{bar:<7}"), style));
            spans.push(Span::styled(format!("{count}"), t.stat_value));
        }

        lines.push(Line::from(spans));
    }

    let block = Block::bordered()
        .title(" Grade Distribution ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}

fn draw_pipeline_health(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let mut lines = Vec::new();

    // Total active (non-archived, non-bespoke) companies with portals.
    let total_with_ats: i64 = s.ats_coverage.iter().map(|(_, c)| *c).sum();

    if s.ats_coverage.is_empty() && s.bespoke_count == 0 {
        lines.push(Line::from(Span::styled(
            "  No ATS portals resolved yet",
            t.dim,
        )));
    } else {
        for (provider, count) in &s.ats_coverage {
            let pct = if total_with_ats > 0 {
                (*count as f64 / total_with_ats as f64 * 100.0).round() as i64
            } else {
                0
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{provider:<12}"), t.stat_value),
                Span::styled(format!("{count:>3}"), t.stat_value),
                Span::styled(format!(" ({pct}%)"), t.dim),
            ]));
        }
    }

    if s.bespoke_count > 0 {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<12}", "bespoke"), t.status_bespoke),
            Span::styled(format!("{:>3}", s.bespoke_count), t.stat_value),
        ]));
    }

    if s.archived_count > 0 {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<12}", "archived"), t.dim),
            Span::styled(format!("{:>3}", s.archived_count), t.dim),
        ]));
    }

    let block = Block::bordered()
        .title(" Pipeline Health ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}

fn draw_action_items(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let mut lines = Vec::new();

    // Grade action breakdown.
    let grade_actions: Vec<(&str, &str, Style)> = vec![
        ("SS", "apply immediately", t.grade_ss),
        ("S", "strong candidates", t.grade_s),
        ("A", "worth applying", t.grade_a),
    ];

    for (grade, label, style) in &grade_actions {
        let count = s.jobs_by_grade.iter()
            .find(|(g, _)| g == grade)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        if count > 0 || *grade == "SS" || *grade == "S" {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{count:>3} {grade:<3}"), *style),
                Span::styled(format!("— {label}"), t.dim),
            ]));
        }
    }

    lines.push(Line::from(""));

    // Decision counts.
    let mut decision_parts = Vec::new();
    decision_parts.push(Span::raw("  "));
    decision_parts.push(Span::styled(format!("{}", s.applied_count), t.decision_applied));
    decision_parts.push(Span::raw(" applied · "));
    decision_parts.push(Span::styled(format!("{}", s.watching_count), t.decision_watching));
    decision_parts.push(Span::raw(" watching · "));
    decision_parts.push(Span::styled(format!("{}", s.rejected_count), t.decision_rejected));
    decision_parts.push(Span::raw(" rejected"));
    lines.push(Line::from(decision_parts));

    // Next steps section.
    let mut next_steps: Vec<String> = Vec::new();
    if s.bespoke_searchable > 0 {
        next_steps.push(format!(
            "{} bespoke {} need manual job search",
            s.bespoke_searchable,
            if s.bespoke_searchable == 1 { "company" } else { "companies" }
        ));
    }
    if s.needs_description > 0 {
        next_steps.push(format!(
            "{} {} need descriptions",
            s.needs_description,
            if s.needs_description == 1 { "job" } else { "jobs" }
        ));
    }

    let pending: i64 = s
        .jobs_by_eval
        .iter()
        .filter(|(e, _)| e == "pending")
        .map(|(_, c)| c)
        .sum();
    if pending > 0 {
        next_steps.push(format!(
            "{} {} pending evaluation",
            pending,
            if pending == 1 { "job" } else { "jobs" }
        ));
    }

    if !next_steps.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  Next Steps:", t.header)));
        for step in &next_steps {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("• ", t.dim),
                Span::raw(step.clone()),
            ]));
        }
    }

    let block = Block::bordered()
        .title(" Action Items ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}

fn draw_top_roles(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let mut lines = Vec::new();

    if s.top_matches.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No SS/S graded jobs yet",
            t.dim,
        )));
    } else {
        for m in &s.top_matches {
            let grade_str = m.grade.as_deref().unwrap_or("—");
            let style = t.grade_style(m.grade.as_deref());

            // First line: grade + title
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{grade_str:<3}"), style),
                Span::styled(m.title.clone(), t.stat_value),
            ]));

            // Second line: company + location
            let location_str = m.location.as_deref().unwrap_or("");
            let detail = if location_str.is_empty() {
                m.company.clone()
            } else {
                format!("{} · {}", m.company, location_str)
            };
            lines.push(Line::from(vec![
                Span::raw("      "),
                Span::styled(detail, t.dim),
            ]));
        }
    }

    let block = Block::bordered()
        .title(" Top Roles ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), area);
}
