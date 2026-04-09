use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, Wrap};
use ratatui::Frame;

use crate::tui::app::{App, Focus};
use crate::tui::widgets::text_utils;

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_compact = area.width < 80;
    let is_narrow = area.width < 100;

    if is_compact {
        // Compact: list only, no detail panel.
        app.list_area = area;
        app.detail_area = Rect::default();
        draw_list(frame, app, area);
    } else if is_narrow {
        // Narrow: stacked vertically.
        let chunks = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Fill(1),
        ]).split(area);
        app.list_area = chunks[0];
        app.detail_area = chunks[1];
        draw_list(frame, app, app.list_area);
        draw_detail(frame, app, app.detail_area);
    } else {
        // Normal: side-by-side.
        let chunks =
            Layout::horizontal([Constraint::Percentage(45), Constraint::Fill(1)]).split(area);
        app.list_area = chunks[0];
        app.detail_area = chunks[1];
        draw_list(frame, app, app.list_area);
        draw_detail(frame, app, app.detail_area);
    }
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

    // Column headers — Decision column removed, description indicator added.
    let header = Row::new(vec![
        Cell::from(" Gr"),
        Cell::from("D"),  // description indicator
        Cell::from("P"),  // application package ready indicator
        Cell::from("Title"),
        Cell::from("Company"),
        Cell::from("Location"),
    ])
    .style(t.header)
    .height(1);

    let rows: Vec<Row> = app
        .jobs
        .iter()
        .enumerate()
        .map(|(idx, j)| {
            let is_multi = app.multi_select_jobs.contains(&idx);

            // When grouped by company, check if this is the first job of a new company group.
            let is_group_start = app.group_by_company && (idx == 0
                || app.jobs.get(idx.wrapping_sub(1))
                    .map_or(true, |prev| prev.company_name != j.company_name));

            // Animated spinner for pending/evaluating jobs, grade otherwise.
            let (grade_display, grade_style) = match j.evaluation_status.as_str() {
                "pending" => {
                    let spinners = ['◐', '◑', '◒', '◓'];
                    let ch = spinners[(app.frame_count / 5 % 4) as usize];
                    let decision_indicator = match j.decision.as_deref() {
                        Some("watching") | Some("applied") => "✓",
                        Some("rejected") => "✗",
                        _ => "·",
                    };
                    (format!("{ch}{decision_indicator}"), t.eval_pending)
                }
                "evaluating" => {
                    let spinners = ['◐', '◑', '◒', '◓'];
                    let ch = spinners[(app.frame_count / 5 % 4) as usize];
                    let decision_indicator = match j.decision.as_deref() {
                        Some("watching") | Some("applied") => "✓",
                        Some("rejected") => "✗",
                        _ => "·",
                    };
                    (format!("{ch}{decision_indicator}"), t.eval_evaluating)
                }
                _ => {
                    let grade_raw = j.grade.as_deref().unwrap_or("—");
                    let style = t.grade_style(j.grade.as_deref());
                    let decision_indicator = match j.decision.as_deref() {
                        Some("watching") | Some("applied") => "✓",
                        Some("rejected") => "✗",
                        _ => "·",
                    };
                    (format!("{grade_raw}{decision_indicator}"), style)
                }
            };

            // Description indicator.
            let has_desc = j
                .raw_description
                .as_ref()
                .map_or(false, |d| d.len() >= 50);
            let desc_indicator = if has_desc { "✓" } else { "·" };
            let desc_style = if has_desc {
                Style::default().fg(Color::Green)
            } else {
                t.dim
            };

            let location = j.location.as_deref().unwrap_or("—");

            // Application package indicator.
            let pkg_indicator = if j.has_package { "●" } else { " " };
            let pkg_style = if j.has_package {
                Style::default().fg(Color::Yellow)
            } else {
                t.dim
            };

            // "New" badge: magenta ● for jobs discovered in last 24 hours.
            let is_new = j.posted_date.as_deref().map_or(false, |d| {
                chrono::NaiveDate::parse_from_str(&d[..10.min(d.len())], "%Y-%m-%d")
                    .map_or(false, |date| {
                        let today = chrono::Local::now().date_naive();
                        (today - date).num_days() <= 1
                    })
            });

            // When grouped: show company name as bold separator on first row, hide on subsequent.
            let company_display = if app.group_by_company {
                if is_group_start {
                    Cell::from(Line::from(Span::styled(
                        j.company_name.as_str(),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )))
                } else {
                    Cell::from(Line::from(Span::styled("  ·", Style::default().fg(Color::DarkGray))))
                }
            } else {
                Cell::from(j.company_name.as_str())
            };

            let mut row = Row::new(vec![
                Cell::from(format!("{}{grade_display:<4}", if is_multi { "▪" } else { " " })).style(grade_style),
                Cell::from(desc_indicator).style(desc_style),
                Cell::from(pkg_indicator).style(pkg_style),
                Cell::from(Line::from(if is_new {
                    vec![
                        Span::styled("● ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                        Span::raw(&j.title),
                    ]
                } else {
                    vec![Span::raw(&j.title)]
                })),
                company_display,
                Cell::from(truncate(location, 16)),
            ]);
            if is_multi {
                row = row.style(Style::default().fg(Color::Cyan));
            }
            // Add top border for group headers when grouped.
            if is_group_start && idx > 0 {
                row = row.top_margin(1);
            }
            row
        })
        .collect();

    // Responsive column widths.
    let is_compact = area.width < 80;
    let widths = if is_compact {
        vec![
            Constraint::Length(5),  // grade
            Constraint::Fill(1),   // title
            Constraint::Length(12), // company (shorter)
        ]
    } else {
        vec![
            Constraint::Length(5),  // grade + decision indicator
            Constraint::Length(2),  // description indicator
            Constraint::Length(2),  // package indicator
            Constraint::Fill(1),   // title
            Constraint::Length(16), // company
            Constraint::Length(16), // location
        ]
    };

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

    // Scrollbar indicator.
    if !app.jobs.is_empty() {
        let mut scrollbar_state = ScrollbarState::new(app.jobs.len())
            .position(app.job_state.selected().unwrap_or(0));
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

    // ── Title ──
    lines.push(Line::from(Span::styled(
        format!("  {}", j.title),
        Style::default().add_modifier(Modifier::BOLD),
    )));

    // ── Company ──
    lines.push(Line::from(Span::styled(
        format!("  {}", j.company_name),
        t.dim,
    )));
    lines.push(Line::from(""));

    // ── Details (location, posted date, grade) ──
    lines.push(Line::from(Span::styled("  ── Details ──", t.header)));
    lines.push(Line::from(""));

    if let Some(loc) = &j.location {
        lines.push(detail_row(t, "Location", Span::raw(loc)));
    }
    if let Some(remote) = &j.remote_policy {
        lines.push(detail_row(t, "Remote", Span::raw(remote)));
    }
    if let Some(posted) = &j.posted_date {
        let display = text_utils::relative_date(posted);
        lines.push(detail_row(t, "Posted", Span::raw(display)));
    }

    let grade = j.grade.as_deref().unwrap_or("—");
    let grade_style = t.grade_style(j.grade.as_deref());
    lines.push(detail_row(t, "Grade", Span::styled(grade, grade_style)));

    // Decision history: show full journey, not just latest.
    {
        let history = fetch_decision_history(app, j.id);
        if !history.is_empty() {
            let mut spans = vec![
                Span::styled("  Decision  ", t.stat_label),
            ];
            for (i, (decision, date)) in history.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::styled(" → ", t.dim));
                }
                let dec_style = t.decision_style(Some(decision));
                let short_date = if date.len() >= 10 { &date[5..10] } else { date };
                spans.push(Span::styled(decision.clone(), dec_style));
                spans.push(Span::styled(format!(" ({short_date})"), t.dim));
            }
            lines.push(Line::from(spans));
        } else if let Some(decision) = &j.decision {
            let dec_style = t.decision_style(Some(decision));
            lines.push(detail_row(
                t,
                "Decision",
                Span::styled(decision, dec_style),
            ));
        }
    }

    if let Some(score) = j.fit_score {
        lines.push(detail_row(
            t,
            "Fit Score",
            Span::raw(format!("{score:.1}")),
        ));
    }

    // ── Fit Assessment ──
    if let Some(assessment) = &j.fit_assessment {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ── Fit Assessment ──",
            t.header,
        )));
        lines.push(Line::from(""));
        let section_header_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        let bold_style = Style::default().add_modifier(Modifier::BOLD);
        for text_line in assessment.lines() {
            let trimmed = text_line.trim_start();
            let style = if trimmed.starts_with("Q1")
                || trimmed.starts_with("Q2")
                || trimmed.starts_with("Q3")
                || trimmed.starts_with("Q4")
                || trimmed.starts_with("Q5")
                || trimmed.starts_with("Overall:")
                || trimmed.starts_with("Strengths:")
                || trimmed.starts_with("Weaknesses:")
                || trimmed.starts_with("Summary:")
                || trimmed.starts_with("Achievability:")
                || trimmed.starts_with("CV Signal:")
                || trimmed.starts_with("Background:")
                || trimmed.starts_with("Engagement:")
                || trimmed.starts_with("Constraints:")
            {
                section_header_style
            } else if trimmed.starts_with("Grade:") || trimmed.contains("Grade:") {
                bold_style
            } else {
                Style::default()
            };
            lines.push(Line::from(Span::styled(
                format!("  {text_line}"),
                style,
            )));
        }
    }

    // ── Full Description ──
    if let Some(desc) = &j.raw_description {
        let cleaned = text_utils::clean_description(desc);
        if !cleaned.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  ── Description ──",
                t.header,
            )));
            lines.push(Line::from(""));
            for text_line in cleaned.lines() {
                lines.push(Line::from(format!("  {text_line}")));
            }
        }
    }

    // ── Link (at bottom) ──
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

    let line_count = lines.len();

    let detail = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));

    frame.render_widget(detail, area);

    // Detail scrollbar.
    if line_count > 0 {
        let mut scrollbar_state = ScrollbarState::new(line_count)
            .position(app.detail_scroll as usize);
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
    text_utils::truncate_chars(s, max)
}

/// Fetch full decision history for a job (watching → applied → interview, etc.).
fn fetch_decision_history(app: &App, job_id: i64) -> Vec<(String, String)> {
    let Ok(conn) = rusqlite::Connection::open(&app.db_path) else {
        return Vec::new();
    };
    conn.prepare(
        "SELECT decision, decided_at FROM user_decisions
         WHERE job_id = ?1 ORDER BY decided_at ASC",
    )
    .and_then(|mut stmt| {
        stmt.query_map([job_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
    })
    .unwrap_or_default()
}


#[cfg(test)]
mod tests {
    use crate::tui::widgets::text_utils;

    #[test]
    fn test_clean_text_html_entities() {
        let input = "We&amp;re looking for &lt;engineers&gt; who &quot;care&quot;";
        let result = text_utils::clean_description(input);
        assert!(result.contains("We&re"));
        assert!(result.contains("<engineers>"));
        assert!(result.contains("\"care\""));
    }

    #[test]
    fn test_clean_text_strips_tags() {
        let input = "<p>Hello</p><br><b>World</b>";
        let result = text_utils::clean_description(input);
        assert!(!result.contains('<'));
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_clean_text_collapses_blanks() {
        let input = "Line 1\n\n\n\n\nLine 2\n\n\nLine 3";
        let result = text_utils::clean_description(input);
        assert_eq!(result, "Line 1\n\nLine 2\n\nLine 3");
    }

    #[test]
    fn test_format_relative_date() {
        let _ = text_utils::relative_date("2026-04-07");
        let _ = text_utils::relative_date("2026-04-07T12:00:00Z");
        let _ = text_utils::relative_date("not-a-date");
        assert_eq!(text_utils::relative_date("not-a-date"), "not-a-date");
    }
}
