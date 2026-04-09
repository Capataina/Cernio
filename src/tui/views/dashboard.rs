use std::collections::HashMap;

use chrono::{Datelike, Local, NaiveDate};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;
use rusqlite::Connection;

use crate::tui::app::App;
use crate::tui::widgets::layout::{distribute, BlockSpec};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    // Session summary block — show if file exists.
    let has_summary = std::fs::metadata("state/tui-summary.md").is_ok();
    let summary_height = if has_summary { 5 } else { 3 };

    let rows = Layout::vertical([
        Constraint::Length(summary_height), // summary block
        Constraint::Fill(1),               // stats grid
    ])
    .split(area);

    draw_summary_block(frame, app, rows[0], has_summary);

    // Two-column layout: left (grade dist + pipeline + session stats) and right (action items + top roles).
    let cols = Layout::horizontal([
        Constraint::Ratio(2, 5),
        Constraint::Fill(1),
    ])
    .split(rows[1]);

    // Left column: dynamic layout based on content.
    let grade_content = left_grade_content_lines(app);
    let pipeline_content = left_pipeline_content_lines(app);
    let left_specs = vec![
        BlockSpec { content_lines: grade_content, min_height: 5, grow_priority: 0 },
        BlockSpec { content_lines: pipeline_content, min_height: 4, grow_priority: 0 },
        BlockSpec { content_lines: 12, min_height: 8, grow_priority: 1 }, // session stats grows
        BlockSpec { content_lines: 9, min_height: 9, grow_priority: 0 },  // activity heatmap (7 rows + header + pad)
    ];
    let left_constraints = distribute(&left_specs, cols[0].height);
    let left = Layout::vertical(left_constraints).split(cols[0]);

    draw_grade_distribution(frame, app, left[0]);
    draw_pipeline_health(frame, app, left[1]);
    draw_session_stats(frame, app, left[2]);
    draw_activity_chart(frame, app, left[3]);

    // Right column: action items (dynamic) + top roles (grows to fill).
    let action_content = action_items_content_lines(app);
    let right_specs = vec![
        BlockSpec { content_lines: action_content, min_height: 6, grow_priority: 0 },
        BlockSpec { content_lines: 10, min_height: 5, grow_priority: 1 }, // top roles grows
    ];
    let right_constraints = distribute(&right_specs, cols[1].height);
    let right = Layout::vertical(right_constraints).split(cols[1]);

    draw_action_items(frame, app, right[0]);
    draw_top_roles(frame, app, right[1]);
}

/// Content lines for grade distribution block (excluding border).
fn left_grade_content_lines(app: &App) -> u16 {
    let _s = &app.stats;
    let company_grades = ["S", "A", "B", "C"];
    let job_grades = ["SS", "S", "A", "B", "C", "F"];
    let max_rows = company_grades.len().max(job_grades.len());
    // 1 header line + rows
    (max_rows as u16 + 1).min(10)
}

/// Content lines for pipeline health block (excluding border).
fn left_pipeline_content_lines(app: &App) -> u16 {
    let s = &app.stats;
    let mut lines = s.ats_coverage.len();
    if s.bespoke_count > 0 {
        lines += 1;
    }
    if s.archived_count > 0 {
        lines += 1;
    }
    if lines == 0 {
        lines = 1;
    }
    (lines as u16).min(8)
}

/// Content lines for action items block (excluding border).
fn action_items_content_lines(app: &App) -> u16 {
    let s = &app.stats;
    let mut lines: u16 = 3; // grade action lines (SS, S, A)
    lines += 1; // blank
    lines += 1; // decision counts

    let mut next_count: u16 = 0;
    if s.bespoke_searchable > 0 {
        next_count += 1;
    }
    if s.needs_description > 0 {
        next_count += 1;
    }
    let pending: i64 = s
        .jobs_by_eval
        .iter()
        .filter(|(e, _)| e == "pending")
        .map(|(_, c)| c)
        .sum();
    if pending > 0 {
        next_count += 1;
    }
    if next_count > 0 {
        lines += 2 + next_count;
    }

    let bespoke_names = fetch_bespoke_company_names(app);
    if !bespoke_names.is_empty() {
        lines += 1 + bespoke_names.len() as u16;
    }

    lines.min(18)
}

fn draw_summary_block(frame: &mut Frame, app: &App, area: Rect, has_summary: bool) {
    let t = &app.theme;
    let s = &app.stats;

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

    // Search pulse: time since last search with freshness colouring.
    let search_pulse = if let Some(ref ts) = app.last_search_at {
        if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") {
            let now = Local::now().naive_local();
            let hours = (now - parsed).num_hours();
            let (label, style) = if hours < 24 {
                (format!("{hours}h ago"), t.freshness_green)
            } else if hours < 72 {
                (format!("{}d ago", hours / 24), t.freshness_yellow)
            } else if hours < 168 {
                (format!("{}d ago", hours / 24), t.freshness_red)
            } else {
                (format!("{}d ago", hours / 24), t.dim)
            };
            vec![
                Span::raw(" · search: "),
                Span::styled(label, style),
            ]
        } else {
            vec![]
        }
    } else {
        vec![
            Span::raw(" · search: "),
            Span::styled("never", t.dim),
        ]
    };

    // Visa countdown.
    let visa_spans = {
        let expiry = NaiveDate::from_ymd_opt(2027, 8, 31).unwrap();
        let today = Local::now().date_naive();
        let days = (expiry - today).num_days();
        let style = if days > 365 {
            t.countdown_ok
        } else if days >= 180 {
            t.countdown_warn
        } else {
            t.countdown_urgent
        };
        vec![
            Span::raw(" · visa: "),
            Span::styled(format!("{days}d"), style),
        ]
    };

    let mut summary_spans = vec![
        Span::raw("  "),
        Span::styled(format!("{}", s.total_companies), t.stat_value),
        Span::raw(" companies · "),
        Span::styled(format!("{}", s.total_jobs), t.stat_value),
        Span::raw(" jobs · "),
        Span::styled(format!("{strong}"), t.grade_s),
        Span::raw(" strong · "),
        Span::styled(
            format!("{pending}"),
            if pending > 0 { t.eval_evaluating } else { t.dim },
        ),
        Span::raw(" pending"),
    ];
    summary_spans.extend(search_pulse);
    summary_spans.extend(visa_spans);

    let mut lines = vec![Line::from(summary_spans)];

    // Show session summary if the file exists.
    if has_summary {
        if let Ok(content) = std::fs::read_to_string("state/tui-summary.md") {
            lines.push(Line::from(""));
            let summary_lines: Vec<String> = content
                .lines()
                .filter(|l| !l.trim().is_empty())
                .take(2)
                .map(|l| l.trim().to_string())
                .collect();
            for sl in summary_lines {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(sl, t.dim),
                ]));
            }
        }
    }

    let block = Block::bordered().border_style(Style::default().fg(t.border));

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

/// Render a proportional bar using `█` characters scaled to the available width.
fn proportional_bar(count: i64, max_val: i64, max_bar_width: u16) -> String {
    if max_val == 0 || count == 0 {
        return String::new();
    }
    let width =
        ((count as f64 / max_val as f64) * max_bar_width as f64).ceil() as usize;
    let width = width.max(1);
    "█".repeat(width)
}

fn draw_grade_distribution(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    // Calculate available width for bars dynamically.
    // Layout: "  S  ████ 123  |  SS ████ 123"
    // Each side: 2 padding + 3 grade + bar + 1 space + count_width
    // We split the inner area (area.width - 2 borders) into two halves.
    let inner_w = area.width.saturating_sub(2) as u16;
    let half_w = inner_w / 2;
    // label = "  XX " = 5 chars, count = " 9999" = 5 chars, separator = 2
    let bar_width = half_w.saturating_sub(5 + 5 + 1);

    let spacer = " ".repeat(half_w.saturating_sub(12) as usize);
    let mut lines = vec![Line::from(vec![
        Span::raw("  "),
        Span::styled("Companies", t.header),
        Span::raw(spacer),
        Span::styled("Jobs", t.header),
    ])];

    let max_company = s
        .companies_by_grade
        .iter()
        .map(|(_, c)| *c)
        .max()
        .unwrap_or(1)
        .max(1);
    let max_job = s
        .jobs_by_grade
        .iter()
        .map(|(_, c)| *c)
        .max()
        .unwrap_or(1)
        .max(1);

    let company_grades = ["S", "A", "B", "C"];
    let job_grades = ["SS", "S", "A", "B", "C", "F"];
    let max_rows = company_grades.len().max(job_grades.len());

    for i in 0..max_rows {
        let mut spans = Vec::new();
        spans.push(Span::raw("  "));

        // Company side.
        if i < company_grades.len() {
            let grade = company_grades[i];
            let count = s
                .companies_by_grade
                .iter()
                .find(|(g, _)| g == grade)
                .map(|(_, c)| *c)
                .unwrap_or(0);
            let style = t.grade_style(Some(grade));
            let bar = proportional_bar(count, max_company, bar_width);
            let bar_len = bar.chars().count();
            let pad = bar_width.saturating_sub(bar_len as u16) as usize;
            spans.push(Span::styled(format!("{grade:<2} "), style));
            spans.push(Span::styled(bar, style));
            spans.push(Span::raw(" ".repeat(pad)));
            spans.push(Span::styled(format!("{count:>4}"), t.stat_value));
        } else {
            // Empty company side: must match the width of a filled row.
            // Filled row = grade label (3) + bar (bar_width) + count (4) = 7 + bar_width
            let pad = (3 + bar_width as usize + 4) as usize;
            spans.push(Span::raw(" ".repeat(pad)));
        }

        spans.push(Span::raw("  "));

        // Job side.
        if i < job_grades.len() {
            let grade = job_grades[i];
            let count = s
                .jobs_by_grade
                .iter()
                .find(|(g, _)| g == grade)
                .map(|(_, c)| *c)
                .unwrap_or(0);
            let style = t.grade_style(Some(grade));
            let bar = proportional_bar(count, max_job, bar_width);
            let bar_len = bar.chars().count();
            let pad = bar_width.saturating_sub(bar_len as u16) as usize;
            spans.push(Span::styled(format!("{grade:<2} "), style));
            spans.push(Span::styled(bar, style));
            spans.push(Span::raw(" ".repeat(pad)));
            spans.push(Span::styled(format!("{count:>4}"), t.stat_value));
        }

        lines.push(Line::from(spans));
    }

    let block = Block::bordered()
        .title(" Grade Distribution ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_pipeline_health(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let inner_w = area.width.saturating_sub(2);
    // "  smartrecruiters ████ 123 (45%)"
    // 2 pad + 18 name + bar + 1 + 3 count + 6 pct = ~30 fixed
    let bar_width = inner_w.saturating_sub(30);

    let mut lines = Vec::new();
    let total_with_ats: i64 = s.ats_coverage.iter().map(|(_, c)| *c).sum();
    let max_ats = s
        .ats_coverage
        .iter()
        .map(|(_, c)| *c)
        .max()
        .unwrap_or(1)
        .max(1);

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
            let bar = proportional_bar(*count, max_ats, bar_width);
            let bar_len = bar.chars().count();
            let pad = bar_width.saturating_sub(bar_len as u16) as usize;
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{provider:<18}"), t.stat_value),
                Span::styled(bar, t.grade_a),
                Span::raw(" ".repeat(pad)),
                Span::styled(format!("{count:>3}"), t.stat_value),
                Span::styled(format!(" ({pct}%)"), t.dim),
            ]));
        }
    }

    if s.bespoke_count > 0 {
        let pad = bar_width as usize;
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<18}", "bespoke"), t.status_bespoke),
            Span::raw(" ".repeat(pad)),
            Span::styled(format!("{:>3}", s.bespoke_count), t.stat_value),
        ]));
    }

    if s.archived_count > 0 {
        let pad = bar_width as usize;
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<18}", "archived"), t.dim),
            Span::raw(" ".repeat(pad)),
            Span::styled(format!("{:>3}", s.archived_count), t.dim),
        ]));
    }

    let block = Block::bordered()
        .title(" Pipeline Health ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

/// Fill the bottom-left area with session-level stats.
fn draw_session_stats(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let mut lines = Vec::new();

    // Application progress bar: Applied ██████░░░░ 22/110 SS+S+A
    let ssa_total: i64 = s
        .jobs_by_grade
        .iter()
        .filter(|(g, _)| g == "SS" || g == "S" || g == "A")
        .map(|(_, c)| *c)
        .sum();
    {
        let inner_w = area.width.saturating_sub(2) as usize; // minus borders
        // "  Applied: " = 11, " 999/999 SS+S+A" = 16 => overhead ~27
        let overhead = 27;
        let bar_width = inner_w.saturating_sub(overhead);
        let filled = if ssa_total > 0 {
            ((s.applied_count as f64 / ssa_total as f64) * bar_width as f64).round() as usize
        } else {
            0
        };
        let filled = filled.min(bar_width);
        let empty = bar_width.saturating_sub(filled);

        lines.push(Line::from(vec![
            Span::raw("  Applied: "),
            Span::styled("█".repeat(filled), t.activity_applied),
            Span::styled("░".repeat(empty), t.dim),
            Span::raw(format!(" {}/{} SS+S+A", s.applied_count, ssa_total)),
        ]));
    }

    // Application pipeline.
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{}", s.applied_count), t.decision_applied),
        Span::raw(" applied · "),
        Span::styled(format!("{}", s.watching_count), t.decision_watching),
        Span::raw(" watching · "),
        Span::styled(format!("{}", s.rejected_count), t.decision_rejected),
        Span::raw(" rejected"),
    ]));

    lines.push(Line::from(""));

    // Coverage stats.
    let resolved: i64 = s.companies_by_status.iter()
        .filter(|(st, _)| st == "resolved")
        .map(|(_, c)| *c)
        .sum();
    let bespoke = s.bespoke_count;
    let total = s.total_companies;
    let coverage_pct = if total > 0 { (resolved + bespoke) * 100 / total } else { 0 };

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{resolved}"), t.stat_value),
        Span::raw(" resolved · "),
        Span::styled(format!("{bespoke}"), t.status_bespoke),
        Span::raw(" bespoke · "),
        Span::styled(format!("{coverage_pct}%"), t.stat_value),
        Span::raw(" coverage"),
    ]));

    // Grade distribution summary.
    lines.push(Line::from(""));
    let ss_count: i64 = s.jobs_by_grade.iter().filter(|(g, _)| g == "SS").map(|(_, c)| *c).sum();
    let s_count: i64 = s.jobs_by_grade.iter().filter(|(g, _)| g == "S").map(|(_, c)| *c).sum();
    let a_count: i64 = s.jobs_by_grade.iter().filter(|(g, _)| g == "A").map(|(_, c)| *c).sum();
    let b_count: i64 = s.jobs_by_grade.iter().filter(|(g, _)| g == "B").map(|(_, c)| *c).sum();
    let f_count: i64 = s.jobs_by_grade.iter().filter(|(g, _)| g == "F").map(|(_, c)| *c).sum();
    let hit_rate = if s.total_jobs > 0 {
        (ss_count + s_count + a_count) * 100 / s.total_jobs
    } else {
        0
    };
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{hit_rate}%"), t.grade_s),
        Span::raw(" hit rate (SS+S+A) · "),
        Span::styled(format!("{f_count}"), t.grade_f),
        Span::raw(" filtered (F)"),
    ]));

    // Conversion funnel.
    let total_graded = ss_count + s_count + a_count + b_count + f_count;
    if total_graded > 0 && s.applied_count > 0 {
        let apply_rate = s.applied_count * 100 / (ss_count + s_count + a_count).max(1);
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{apply_rate}%"), t.decision_applied),
            Span::raw(" of SS+S+A applied to"),
        ]));
    }

    // Per-tier application breakdown.
    let applied_by_grade = fetch_applied_by_grade(app);
    if !applied_by_grade.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  Applied by grade:", t.header)));
        for (grade, count) in &applied_by_grade {
            let style = t.grade_style(Some(grade.as_str()));
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{grade:<3}"), style),
                Span::styled(format!("{count}"), t.stat_value),
            ]));
        }
    }

    // Jobs per company.
    let total_active_companies = s.total_companies.saturating_sub(s.archived_count);
    let avg_jobs = if total_active_companies > 0 {
        s.total_jobs / total_active_companies
    } else {
        0
    };
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{avg_jobs}"), t.stat_value),
        Span::raw(" avg jobs/company"),
    ]));

    // Top companies leaderboard.
    if !app.top_companies_by_hits.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  Top producing companies:", t.header)));
        for (name, count) in &app.top_companies_by_hits {
            // Truncate long names to fit.
            let display_name = if name.len() > 16 {
                format!("{}…", &name[..15])
            } else {
                name.clone()
            };
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{display_name:<16} "), t.stat_value),
                Span::styled(format!("{count}"), t.grade_s),
                Span::styled(" strong", t.dim),
            ]));
        }
    }

    let block = Block::bordered()
        .title(" Session Stats ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

/// Draw a GitHub-style activity heatmap: 7 rows (Mon-Sun) x 12 columns (weeks).
///
/// Each cell is a coloured `█` character representing the highest-priority action
/// that day. Priority: applied > searched > graded > discovered. No activity = dim `░`.
fn draw_activity_chart(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;

    // Build a map: date_string -> list of action types.
    let mut by_date: HashMap<String, Vec<String>> = HashMap::new();
    for (date, action) in &app.activity_data {
        by_date.entry(date.clone()).or_default().push(action.clone());
    }

    let today = Local::now().date_naive();
    // Start 83 days ago (12 weeks = 84 days including today).
    let start = today - chrono::Duration::days(83);

    // Day labels for rows (Monday = 0 through Sunday = 6).
    let day_labels = ["M", "T", "W", "T", "F", "S", "S"];

    let mut rows: Vec<Vec<Span>> = Vec::new();
    for dow in 0..7u32 {
        let mut spans = Vec::new();
        spans.push(Span::styled(format!("  {} ", day_labels[dow as usize]), t.dim));

        // Iterate through each week (column).
        for week in 0..12 {
            // Calculate the date for this cell.
            // The grid starts at `start`. We need to find the date for row=dow, col=week.
            // start's weekday (Mon=0, ..., Sun=6).
            let start_dow = start.weekday().num_days_from_monday(); // 0=Mon
            // First Monday on or before start.
            let grid_origin = start - chrono::Duration::days(start_dow as i64);
            let cell_date = grid_origin + chrono::Duration::days((week * 7 + dow) as i64);

            if cell_date > today || cell_date < start {
                spans.push(Span::styled(" ", t.dim));
            } else {
                let date_str = cell_date.format("%Y-%m-%d").to_string();
                let style = if let Some(actions) = by_date.get(&date_str) {
                    // Priority: applied > searched > graded > discovered.
                    if actions.iter().any(|a| a == "applied") {
                        t.activity_applied
                    } else if actions.iter().any(|a| a == "searched") {
                        t.activity_searched
                    } else if actions.iter().any(|a| a == "graded") {
                        t.activity_graded
                    } else {
                        t.activity_discovered
                    }
                } else {
                    t.dim
                };

                if by_date.contains_key(&cell_date.format("%Y-%m-%d").to_string()) {
                    spans.push(Span::styled("█", style));
                } else {
                    spans.push(Span::styled("░", t.dim));
                }
            }
            spans.push(Span::raw(" "));
        }
        rows.push(spans);
    }

    let lines: Vec<Line> = rows.into_iter().map(Line::from).collect();

    let block = Block::bordered()
        .title(" Activity (12 weeks) ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

/// Fetch count of applied jobs broken down by grade.
fn fetch_applied_by_grade(app: &App) -> Vec<(String, i64)> {
    let Ok(conn) = Connection::open(&app.db_path) else {
        return Vec::new();
    };

    conn.prepare(
        "SELECT j.grade, COUNT(*)
         FROM user_decisions ud
         JOIN jobs j ON j.id = ud.job_id
         WHERE ud.decision = 'applied' AND j.grade IS NOT NULL
         GROUP BY j.grade
         ORDER BY CASE j.grade
             WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3
             WHEN 'B' THEN 4 WHEN 'C' THEN 5 WHEN 'F' THEN 6
         END",
    )
    .and_then(|mut stmt| {
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
    })
    .unwrap_or_default()
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
        let count = s
            .jobs_by_grade
            .iter()
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
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{}", s.applied_count), t.decision_applied),
        Span::raw(" applied · "),
        Span::styled(format!("{}", s.watching_count), t.decision_watching),
        Span::raw(" watching · "),
        Span::styled(format!("{}", s.rejected_count), t.decision_rejected),
        Span::raw(" rejected"),
    ]));

    // Next steps section.
    let mut next_steps: Vec<String> = Vec::new();
    if s.bespoke_searchable > 0 {
        next_steps.push(format!(
            "{} bespoke {} need manual job search",
            s.bespoke_searchable,
            if s.bespoke_searchable == 1 {
                "company"
            } else {
                "companies"
            }
        ));
    }
    if s.needs_description > 0 {
        next_steps.push(format!(
            "{} {} need descriptions",
            s.needs_description,
            if s.needs_description == 1 {
                "job"
            } else {
                "jobs"
            }
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

    // List bespoke company names that need search.
    let bespoke_names = fetch_bespoke_company_names(app);
    if !bespoke_names.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Bespoke — need search:",
            t.header,
        )));
        for name in &bespoke_names {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("• ", t.status_bespoke),
                Span::raw(name.clone()),
            ]));
        }
    }

    let block = Block::bordered()
        .title(" Action Items ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_top_roles(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;

    // Fetch ALL SS, S, and A jobs from DB for the scrollable list.
    let roles = fetch_all_top_roles(app);

    let mut lines = Vec::new();

    if roles.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No SS/S/A graded jobs yet",
            t.dim,
        )));
    } else {
        // Single-line format: "SS  title — company"
        for (grade, title, company) in &roles {
            let style = t.grade_style(Some(grade.as_str()));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{grade:<3}"), style),
                Span::styled(title.clone(), t.stat_value),
                Span::styled(format!(" — {company}"), t.dim),
            ]));
        }
    }

    // Apply scroll offset.
    let scroll = app.dashboard_scroll;

    let block = Block::bordered()
        .title(format!(" Top Roles ({}) ", roles.len()))
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(para, area);
}

/// Fetch bespoke company names that need manual job search.
/// Shows S/A bespoke companies that have never been searched or
/// haven't been searched in 7+ days.
fn fetch_bespoke_company_names(app: &App) -> Vec<String> {
    let Ok(conn) = Connection::open(&app.db_path) else {
        return Vec::new();
    };

    let sql = "
        SELECT name FROM companies
        WHERE status = 'bespoke'
        AND grade IN ('S', 'A')
        AND (last_searched_at IS NULL
             OR last_searched_at < datetime('now', '-7 days'))
        ORDER BY
            CASE grade WHEN 'S' THEN 1 WHEN 'A' THEN 2 END,
            name";

    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |row| row.get::<_, String>(0))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}

/// Fetch ALL SS, S, and A graded jobs for the scrollable top roles list.
fn fetch_all_top_roles(app: &App) -> Vec<(String, String, String)> {
    let Ok(conn) = Connection::open(&app.db_path) else {
        return Vec::new();
    };

    let sql = "
        SELECT j.grade, j.title, c.name
        FROM jobs j
        JOIN companies c ON c.id = j.company_id
        WHERE j.grade IN ('SS', 'S', 'A')
        AND j.evaluation_status != 'archived'
        AND c.status != 'archived'
        ORDER BY
            CASE j.grade WHEN 'SS' THEN 1 WHEN 'S' THEN 2 WHEN 'A' THEN 3 END,
            j.title";

    conn.prepare(sql)
        .and_then(|mut stmt| {
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}
