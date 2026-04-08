use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
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

    // Column headers — Decision column removed, description indicator added.
    let header = Row::new(vec![
        Cell::from(" Gr"),
        Cell::from("D"),  // description indicator
        Cell::from("Title"),
        Cell::from("Company"),
        Cell::from("Location"),
    ])
    .style(t.header)
    .height(1);

    let rows: Vec<Row> = app
        .jobs
        .iter()
        .map(|j| {
            let grade_raw = j.grade.as_deref().unwrap_or("—");
            let grade_style = t.grade_style(j.grade.as_deref());

            // Grade with decision indicator: "SS✓" for watching/applied, "SS·" otherwise.
            let decision_indicator = match j.decision.as_deref() {
                Some("watching") | Some("applied") => "✓",
                Some("rejected") => "✗",
                _ => "·",
            };
            let grade_display = format!("{grade_raw}{decision_indicator}");

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

            Row::new(vec![
                Cell::from(format!(" {grade_display:<4}")).style(grade_style),
                Cell::from(desc_indicator).style(desc_style),
                Cell::from(j.title.as_str()),
                Cell::from(j.company_name.as_str()),
                Cell::from(truncate(location, 16)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(5),  // grade + decision indicator
        Constraint::Length(2),  // description indicator
        Constraint::Fill(1),   // title
        Constraint::Length(16), // company
        Constraint::Length(16), // location
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
        let display = format_relative_date(posted);
        lines.push(detail_row(t, "Posted", Span::raw(display)));
    }

    let grade = j.grade.as_deref().unwrap_or("—");
    let grade_style = t.grade_style(j.grade.as_deref());
    lines.push(detail_row(t, "Grade", Span::styled(grade, grade_style)));

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

    // ── Fit Assessment ──
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

    // ── Full Description ──
    if let Some(desc) = &j.raw_description {
        let cleaned = clean_text(desc);
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

// ── Local helpers (no external widget dependency) ────────────────

/// Clean raw description text: strip HTML entities, collapse blank lines,
/// trim whitespace, and remove residual HTML tags.
fn clean_text(raw: &str) -> String {
    let mut text = raw.to_string();

    // Strip HTML tags.
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                // Add a space after closing tags to prevent word merging.
                result.push(' ');
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    text = result;

    // Decode common HTML entities.
    text = text
        .replace("&amp;", "&")
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&#x27;", "'")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&bull;", "•")
        .replace("&hellip;", "…")
        .replace("&#8226;", "•")
        .replace("&#8211;", "–")
        .replace("&#8212;", "—");

    // Trim each line and collapse multiple blank lines into one.
    let mut output_lines: Vec<String> = Vec::new();
    let mut prev_blank = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_blank && !output_lines.is_empty() {
                output_lines.push(String::new());
                prev_blank = true;
            }
        } else {
            output_lines.push(trimmed.to_string());
            prev_blank = false;
        }
    }

    // Remove trailing blank lines.
    while output_lines.last().map_or(false, |l| l.is_empty()) {
        output_lines.pop();
    }

    output_lines.join("\n")
}

/// Parse an ISO date string and return a human-readable relative time.
/// Falls back to the original string if parsing fails.
fn format_relative_date(date_str: &str) -> String {
    // Try parsing common ISO formats.
    let parsed = parse_date(date_str);
    match parsed {
        Some(days_ago) => {
            if days_ago < 0 {
                format!("in {} days", -days_ago)
            } else if days_ago == 0 {
                "today".to_string()
            } else if days_ago == 1 {
                "yesterday".to_string()
            } else if days_ago < 7 {
                format!("{days_ago} days ago")
            } else if days_ago < 14 {
                "1 week ago".to_string()
            } else if days_ago < 30 {
                format!("{} weeks ago", days_ago / 7)
            } else if days_ago < 60 {
                "1 month ago".to_string()
            } else if days_ago < 365 {
                format!("{} months ago", days_ago / 30)
            } else {
                format!("{} years ago", days_ago / 365)
            }
        }
        None => date_str.to_string(),
    }
}

/// Parse an ISO-ish date string and return the number of days between it and today.
/// Returns None if parsing fails.
fn parse_date(s: &str) -> Option<i64> {
    // Handle "YYYY-MM-DD" or "YYYY-MM-DDT..." formats.
    let date_part = s.split('T').next().unwrap_or(s);
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() < 3 {
        return None;
    }

    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    if month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }

    // Calculate days since epoch for the parsed date and for today.
    let parsed_days = days_from_civil(year, month, day);

    // Get today's date via chrono (already a dependency).
    let now = chrono::Utc::now().date_naive();
    let today_days =
        days_from_civil(now.year() as i32, now.month(), now.day());

    Some(today_days - parsed_days)
}

/// Convert a civil date to a day count (for difference calculation).
/// Algorithm from Howard Hinnant's date algorithms.
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y as i64 - 1 } else { y as i64 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let m = m as u64;
    let d = d as u64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i64 - 719468
}

/// Needed for chrono date access.
use chrono::Datelike;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text_html_entities() {
        let input = "We&amp;re looking for &lt;engineers&gt; who &quot;care&quot;";
        let result = clean_text(input);
        assert!(result.contains("We&re"));
        assert!(result.contains("<engineers>"));
        assert!(result.contains("\"care\""));
    }

    #[test]
    fn test_clean_text_strips_tags() {
        let input = "<p>Hello</p><br><b>World</b>";
        let result = clean_text(input);
        assert!(!result.contains('<'));
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_clean_text_collapses_blanks() {
        let input = "Line 1\n\n\n\n\nLine 2\n\n\nLine 3";
        let result = clean_text(input);
        assert_eq!(result, "Line 1\n\nLine 2\n\nLine 3");
    }

    #[test]
    fn test_format_relative_date() {
        // We can't test exact output since it depends on the current date,
        // but we can test that known formats don't panic.
        let _ = format_relative_date("2026-04-07");
        let _ = format_relative_date("2026-04-07T12:00:00Z");
        let _ = format_relative_date("not-a-date");
        assert_eq!(format_relative_date("not-a-date"), "not-a-date");
    }
}
