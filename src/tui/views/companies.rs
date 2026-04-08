use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table, Wrap};
use ratatui::Frame;
use rusqlite::Connection;

use crate::tui::app::{App, Focus};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Fill(1)]).split(area);

    draw_list(frame, app, chunks[0]);
    draw_detail(frame, app, chunks[1]);
}

fn draw_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let t = &app.theme;
    let focused = app.focus == Focus::List;

    if app.companies.is_empty() {
        let block = Block::bordered()
            .title(" Companies ")
            .title_style(t.title)
            .border_style(Style::default().fg(if focused {
                t.border_focused
            } else {
                t.border
            }));
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from("  No companies in the database yet."),
            Line::from(""),
            Line::from("  Run populate-db to add companies"),
            Line::from("  from your discovery list."),
        ])
        .block(block)
        .style(t.dim);
        frame.render_widget(msg, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(" Gr"),
        Cell::from("Company"),
        Cell::from("Status"),
        Cell::from("Jobs"),
        Cell::from("ATS"),
    ])
    .style(t.header)
    .height(1);

    let rows: Vec<Row> = app
        .companies
        .iter()
        .map(|c| {
            let grade = c.grade.as_deref().unwrap_or("—");
            let grade_style = t.grade_style(c.grade.as_deref());
            let status_style = t.status_style(&c.status);

            let jobs_display = if c.job_count > 0 {
                if c.fit_count > 0 {
                    format!("{} ({}✓)", c.job_count, c.fit_count)
                } else {
                    format!("{}", c.job_count)
                }
            } else {
                "—".into()
            };

            let ats = c
                .ats_provider
                .as_deref()
                .unwrap_or(if c.status == "bespoke" {
                    "bespoke"
                } else {
                    "—"
                });

            Row::new(vec![
                Cell::from(format!(" {grade:<2}")).style(grade_style),
                Cell::from(c.name.as_str()),
                Cell::from(c.status.as_str()).style(status_style),
                Cell::from(jobs_display),
                Cell::from(ats),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(14),
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
                .title(" Companies ")
                .title_style(t.title)
                .border_style(Style::default().fg(if focused {
                    t.border_focused
                } else {
                    t.border
                })),
        )
        .row_highlight_style(highlight)
        .highlight_symbol("▸");

    frame.render_stateful_widget(table, area, &mut app.company_state);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let focused = app.focus == Focus::Detail;

    let Some(c) = app.selected_company() else {
        let block = Block::bordered()
            .title(" Detail ")
            .title_style(t.title)
            .border_style(Style::default().fg(t.border));
        frame.render_widget(
            Paragraph::new("  Select a company")
                .style(t.dim)
                .block(block),
            area,
        );
        return;
    };

    let mut lines = Vec::new();

    // Company name and website.
    lines.push(Line::from(Span::styled(
        format!("  {}", c.name),
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {}", c.website),
        t.dim,
    )));
    lines.push(Line::from(""));

    // Description.
    for text_line in c.what_they_do.lines() {
        lines.push(Line::from(format!("  {text_line}")));
    }
    lines.push(Line::from(""));

    // Details section.
    lines.push(Line::from(Span::styled("  ── Details ──", t.header)));
    lines.push(Line::from(""));

    let grade = c.grade.as_deref().unwrap_or("—");
    let grade_style = t.grade_style(c.grade.as_deref());
    lines.push(detail_row(t, "Grade", Span::styled(grade, grade_style)));

    let status_style = t.status_style(&c.status);
    lines.push(detail_row(
        t,
        "Status",
        Span::styled(&c.status, status_style),
    ));

    if let (Some(provider), Some(slug)) = (&c.ats_provider, &c.ats_slug) {
        lines.push(detail_row(
            t,
            "ATS",
            Span::raw(format!("{provider} / {slug}")),
        ));
    } else if c.status == "bespoke" {
        lines.push(detail_row(t, "ATS", Span::styled("bespoke", t.dim)));
    }

    if let Some(loc) = &c.location {
        lines.push(detail_row(t, "Location", Span::raw(loc)));
    }

    if let Some(tags) = &c.sector_tags {
        lines.push(detail_row(t, "Sectors", Span::raw(tags)));
    }

    if let Some(url) = &c.careers_url {
        lines.push(detail_row(t, "Careers", Span::styled(url, t.dim)));
    }

    // Relevance section.
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("  ── Relevance ──", t.header)));
    lines.push(Line::from(""));
    for text_line in c.why_relevant.lines() {
        lines.push(Line::from(format!("  {text_line}")));
    }

    // Grade reasoning.
    if let Some(reasoning) = &c.grade_reasoning {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ── Grade Reasoning ──",
            t.header,
        )));
        lines.push(Line::from(""));
        for text_line in reasoning.lines() {
            lines.push(Line::from(format!("  {text_line}")));
        }
    }

    // Job section with grade distribution and full job list.
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  ── Jobs ({}) ──", c.job_count),
        t.header,
    )));
    lines.push(Line::from(""));

    if c.job_count > 0 {
        if let Ok(conn) = Connection::open(&app.db_path) {
            // Grade distribution bars — dynamic width.
            let grade_dist = fetch_company_grade_distribution(&conn, c.id);
            let max_count = grade_dist
                .iter()
                .map(|(_, n)| *n)
                .max()
                .unwrap_or(1)
                .max(1);

            // Calculate bar width dynamically from the detail panel width.
            // Inner width minus: 2 border + 2 pad + 3 grade + 1 space + 4 count = 12
            let bar_width = area.width.saturating_sub(14);

            for (g, count) in &grade_dist {
                let filled = if *count > 0 {
                    ((*count as f64 / max_count as f64) * bar_width as f64)
                        .ceil() as usize
                } else {
                    0
                };
                let filled = filled.max(if *count > 0 { 1 } else { 0 });
                let pad = (bar_width as usize).saturating_sub(filled);
                let bar: String = "█".repeat(filled);
                let gs = t.grade_style(Some(g.as_str()));
                lines.push(Line::from(vec![
                    Span::styled(format!("  {g:<2} "), gs),
                    Span::styled(bar, gs),
                    Span::raw(" ".repeat(pad)),
                    Span::styled(format!("{count:>3}"), t.stat_value),
                ]));
            }

            // Full job list — ALL jobs sorted by grade, scrollable.
            let all_jobs = fetch_company_all_jobs(&conn, c.id);
            if !all_jobs.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled("  ── All Roles ──", t.header)));
                lines.push(Line::from(""));
                for (g, title) in &all_jobs {
                    let gs = t.grade_style(Some(g.as_str()));
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(format!("{g:<3}"), gs),
                        Span::raw(title.clone()),
                    ]));
                }
            }
        }
    } else {
        lines.push(Line::from(Span::styled("  No jobs fetched yet", t.dim)));
    }

    let block = Block::bordered()
        .title(format!(" {} ", c.name))
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

/// Returns grade distribution for a company's jobs as (grade, count) pairs.
/// Only includes grades with count > 0. Excludes archived jobs.
fn fetch_company_grade_distribution(conn: &Connection, company_id: i64) -> Vec<(String, i64)> {
    let sql = "
        SELECT COALESCE(grade, '—'), COUNT(*)
        FROM jobs
        WHERE company_id = ?1
        AND evaluation_status != 'archived'
        GROUP BY grade
        ORDER BY CASE grade
            WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3
            WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6
            ELSE 7
        END";

    let raw: Vec<(String, i64)> = conn
        .prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([company_id], |row| Ok((row.get(0)?, row.get(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default();

    let grades = ["SS", "S", "A", "B", "C", "F"];
    let mut result: Vec<(String, i64)> = grades
        .iter()
        .filter_map(|g| {
            let count = raw
                .iter()
                .find(|(k, _)| k == *g)
                .map(|(_, n)| *n)
                .unwrap_or(0);
            if count > 0 {
                Some((g.to_string(), count))
            } else {
                None
            }
        })
        .collect();

    // Include ungraded jobs if any.
    if let Some((_, n)) = raw.iter().find(|(k, _)| k == "—") {
        if *n > 0 {
            result.push(("—".to_string(), *n));
        }
    }

    result
}

/// Returns ALL jobs for a company sorted by grade, as (grade, title) pairs.
/// Excludes archived jobs.
fn fetch_company_all_jobs(conn: &Connection, company_id: i64) -> Vec<(String, String)> {
    let sql = "
        SELECT COALESCE(grade, '—'), title
        FROM jobs
        WHERE company_id = ?1
        AND evaluation_status != 'archived'
        ORDER BY
            CASE grade
                WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3
                WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6
                ELSE 7
            END,
            title";

    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([company_id], |row| Ok((row.get(0)?, row.get(1)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}
