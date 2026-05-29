use chrono::{Local, NaiveDate};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::Frame;
use rusqlite::Connection;

use crate::tui::app::App;
use crate::tui::theme::{lane_badge, primary_lane, LANE_KEYS};

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    // Session summary block — show if file exists.
    let has_summary = std::fs::metadata("state/tui-summary.md").is_ok();
    let summary_height = if has_summary { 5 } else { 3 };

    let rows = Layout::vertical([
        Constraint::Length(summary_height),
        Constraint::Fill(1),
    ])
    .split(area);

    draw_summary_block(frame, app, rows[0], has_summary);

    // Two-column body: left = per-lane distributions + session stats,
    // right = action items + top roles (lane-coloured).
    let cols = Layout::horizontal([
        Constraint::Ratio(2, 5),
        Constraint::Fill(1),
    ])
    .split(rows[1]);

    let left = Layout::vertical([
        Constraint::Length(13),  // companies × lane
        Constraint::Length(13),  // jobs × lane
        Constraint::Fill(1),    // session stats
    ])
    .split(cols[0]);

    draw_companies_per_lane(frame, app, left[0]);
    draw_jobs_per_lane(frame, app, left[1]);
    draw_session_stats(frame, app, left[2]);

    let right = Layout::vertical([
        Constraint::Length(10),
        Constraint::Fill(1),
    ])
    .split(cols[1]);

    draw_action_items(frame, app, right[0]);
    draw_top_roles(frame, app, right[1]);
}

// ── Summary block ───────────────────────────────────────────────

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
            vec![Span::raw(" · search: "), Span::styled(label, style)]
        } else {
            vec![]
        }
    } else {
        vec![Span::raw(" · search: "), Span::styled("never", t.dim)]
    };

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
        vec![Span::raw(" · visa: "), Span::styled(format!("{days}d"), style)]
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
                lines.push(Line::from(vec![Span::raw("  "), Span::styled(sl, t.dim)]));
            }
        }
    }

    // Lane hit-rate strip — % of each lane's jobs that landed SS+S+A.
    if let Some(strip) = lane_hit_rate_strip(app) {
        lines.push(Line::from(strip));
    }

    let block = Block::bordered().border_style(Style::default().fg(t.border));

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn lane_hit_rate_strip(app: &App) -> Option<Vec<Span<'static>>> {
    let Ok(conn) = Connection::open(&app.db_path) else { return None; };
    let rates = fetch_lane_hit_rates(&conn);
    if rates.is_empty() { return None; }
    let t = &app.theme;
    let mut spans = vec![Span::raw("  hit-rate · ")];
    let mut first = true;
    for (lane, pct, total) in &rates {
        if *total == 0 { continue; }
        if !first { spans.push(Span::styled(" · ", t.dim)); }
        first = false;
        let style = t.lane_style(lane);
        spans.push(Span::styled(format!("{} ", lane_badge(lane)), style));
        spans.push(Span::styled(format!("{pct}%"), t.stat_value));
    }
    Some(spans)
}

fn fetch_lane_hit_rates(conn: &Connection) -> Vec<(String, i64, i64)> {
    let mut out = Vec::new();
    for key in LANE_KEYS.iter() {
        let prefix = format!("[\"{key}\"%");
        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM jobs WHERE lanes LIKE ?1 AND evaluation_status != 'archived'",
                rusqlite::params![prefix],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if total == 0 { continue; }
        let strong: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM jobs WHERE lanes LIKE ?1 AND evaluation_status != 'archived' AND grade IN ('SS','S','A')",
                rusqlite::params![prefix],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let pct = if total > 0 { strong * 100 / total } else { 0 };
        out.push((key.to_string(), pct, total));
    }
    out
}

// ── Per-lane distribution blocks ────────────────────────────────

fn draw_companies_per_lane(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let block = Block::bordered()
        .title(" Companies × Lane ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    let Ok(conn) = Connection::open(&app.db_path) else {
        frame.render_widget(Paragraph::new("  (db unavailable)").style(t.dim).block(block), area);
        return;
    };

    let inner_w = area.width.saturating_sub(2);
    let bar_width = inner_w.saturating_sub(20);  // 5 badge + 13 right margin

    let mut max_total: i64 = 1;
    let mut rows: Vec<(String, [i64; 4], i64)> = Vec::new();  // pinnacle / strong / adjacent / borderline / total
    for key in LANE_KEYS.iter() {
        let prefix = format!("[\"{key}\"%");
        let pinnacle: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM companies WHERE lanes LIKE ?1 AND status != 'archived' AND pinnacle_status_per_lane LIKE '%\"pinnacle\"%'",
                rusqlite::params![prefix], |r| r.get(0)).unwrap_or(0);
        let strong: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM companies WHERE lanes LIKE ?1 AND status != 'archived' AND pinnacle_status_per_lane LIKE '%\"strong\"%' AND pinnacle_status_per_lane NOT LIKE '%\"pinnacle\"%'",
                rusqlite::params![prefix], |r| r.get(0)).unwrap_or(0);
        let adjacent: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM companies WHERE lanes LIKE ?1 AND status != 'archived' AND pinnacle_status_per_lane LIKE '%\"adjacent\"%'",
                rusqlite::params![prefix], |r| r.get(0)).unwrap_or(0);
        let borderline: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM companies WHERE lanes LIKE ?1 AND status != 'archived' AND pinnacle_status_per_lane LIKE '%\"borderline\"%'",
                rusqlite::params![prefix], |r| r.get(0)).unwrap_or(0);
        let total = pinnacle + strong + adjacent + borderline;
        if total > max_total { max_total = total; }
        rows.push((key.to_string(), [pinnacle, strong, adjacent, borderline], total));
    }

    let mut lines = Vec::new();
    for (key, parts, total) in &rows {
        let mut spans = vec![
            Span::raw(" "),
            Span::styled(format!("{} ", lane_badge(key)), t.lane_style(key)),
        ];
        let bar = stacked_lane_bar(parts, *total, bar_width as usize, t, "company");
        spans.extend(bar);
        spans.push(Span::raw(" "));
        spans.push(Span::styled(format!("{total:>4}"), t.stat_value));
        lines.push(Line::from(spans));
    }

    // Legend
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("█ ", t.grade_ss),
        Span::styled("pinnacle ", t.dim),
        Span::styled("█ ", t.grade_s),
        Span::styled("strong ", t.dim),
        Span::styled("█ ", t.grade_a),
        Span::styled("adjacent ", t.dim),
        Span::styled("█ ", t.grade_b),
        Span::styled("borderline", t.dim),
    ]));

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_jobs_per_lane(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let block = Block::bordered()
        .title(" Jobs × Lane ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    let Ok(conn) = Connection::open(&app.db_path) else {
        frame.render_widget(Paragraph::new("  (db unavailable)").style(t.dim).block(block), area);
        return;
    };

    let inner_w = area.width.saturating_sub(2);
    let bar_width = inner_w.saturating_sub(20);

    let mut max_total: i64 = 1;
    let grades = ["SS", "S", "A", "B", "C", "F"];
    let mut rows: Vec<(String, [i64; 6], i64)> = Vec::new();
    for key in LANE_KEYS.iter() {
        let prefix = format!("[\"{key}\"%");
        let mut counts = [0i64; 6];
        let mut total = 0i64;
        for (i, g) in grades.iter().enumerate() {
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM jobs WHERE lanes LIKE ?1 AND grade = ?2 AND evaluation_status != 'archived'",
                    rusqlite::params![prefix, g], |r| r.get(0)).unwrap_or(0);
            counts[i] = count;
            total += count;
        }
        if total > max_total { max_total = total; }
        rows.push((key.to_string(), counts, total));
    }

    let mut lines = Vec::new();
    for (key, counts, total) in &rows {
        let mut spans = vec![
            Span::raw(" "),
            Span::styled(format!("{} ", lane_badge(key)), t.lane_style(key)),
        ];
        let bar = stacked_lane_bar(counts, *total, bar_width as usize, t, "job");
        spans.extend(bar);
        spans.push(Span::raw(" "));
        spans.push(Span::styled(format!("{total:>4}"), t.stat_value));
        lines.push(Line::from(spans));
    }

    // Legend
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("█ ", t.grade_ss),
        Span::styled("SS ", t.dim),
        Span::styled("█ ", t.grade_s),
        Span::styled("S ", t.dim),
        Span::styled("█ ", t.grade_a),
        Span::styled("A ", t.dim),
        Span::styled("█ ", t.grade_b),
        Span::styled("B ", t.dim),
        Span::styled("█ ", t.grade_c),
        Span::styled("C ", t.dim),
        Span::styled("█ ", t.grade_f),
        Span::styled("F", t.dim),
    ]));

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn stacked_lane_bar<'a>(
    parts: &[i64],
    total: i64,
    width: usize,
    t: &crate::tui::theme::Theme,
    kind: &str,
) -> Vec<Span<'a>> {
    if total == 0 || width == 0 {
        return vec![Span::styled(" ".repeat(width), t.dim)];
    }
    let styles: Vec<Style> = if kind == "job" {
        vec![t.grade_ss, t.grade_s, t.grade_a, t.grade_b, t.grade_c, t.grade_f]
    } else {
        vec![t.grade_ss, t.grade_s, t.grade_a, t.grade_b]
    };
    let mut spans = Vec::new();
    let mut used = 0usize;
    for (i, count) in parts.iter().enumerate() {
        if *count == 0 { continue; }
        let cells = ((*count as f64 / total as f64) * width as f64).round() as usize;
        let cells = cells.max(1).min(width.saturating_sub(used));
        if cells == 0 { break; }
        spans.push(Span::styled("█".repeat(cells), styles[i]));
        used += cells;
        if used >= width { break; }
    }
    if used < width {
        spans.push(Span::styled(" ".repeat(width - used), t.dim));
    }
    spans
}

// ── Session stats ───────────────────────────────────────────────

fn draw_session_stats(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let s = &app.stats;

    let mut lines = Vec::new();

    let ssa_total: i64 = s
        .jobs_by_grade
        .iter()
        .filter(|(g, _)| g == "SS" || g == "S" || g == "A")
        .map(|(_, c)| *c)
        .sum();
    {
        let inner_w = area.width.saturating_sub(2) as usize;
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

    lines.push(Line::from(""));
    let hit_rate = if s.total_jobs > 0 { (ssa_total) * 100 / s.total_jobs } else { 0 };
    let f_count: i64 = s.jobs_by_grade.iter().filter(|(g, _)| g == "F").map(|(_, c)| *c).sum();
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{hit_rate}%"), t.grade_s),
        Span::raw(" hit rate (SS+S+A) · "),
        Span::styled(format!("{f_count}"), t.grade_f),
        Span::raw(" filtered (F)"),
    ]));

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

    let block = Block::bordered()
        .title(" Session Stats ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn fetch_applied_by_grade(app: &App) -> Vec<(String, i64)> {
    let Ok(conn) = Connection::open(&app.db_path) else { return Vec::new(); };
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

// ── Action items — per-lane fanout ─────────────────────────────

fn draw_action_items(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let block = Block::bordered()
        .title(" Action Items ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    let Ok(conn) = Connection::open(&app.db_path) else {
        frame.render_widget(Paragraph::new("  (db unavailable)").style(t.dim).block(block), area);
        return;
    };

    let mut lines = Vec::new();

    // SS / S / A fanout per lane.
    for grade in &["SS", "S", "A"] {
        let style = t.grade_style(Some(grade));
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM jobs WHERE grade = ?1 AND evaluation_status != 'archived'",
            rusqlite::params![grade], |r| r.get(0)).unwrap_or(0);
        if total == 0 { continue; }

        let mut spans = vec![
            Span::raw("  "),
            Span::styled(format!("{total:>3} {grade:<3}"), style),
            Span::styled(" — ", t.dim),
        ];
        for key in LANE_KEYS.iter() {
            let prefix = format!("[\"{key}\"%");
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM jobs WHERE grade = ?1 AND lanes LIKE ?2 AND evaluation_status != 'archived'",
                rusqlite::params![grade, prefix], |r| r.get(0)).unwrap_or(0);
            if count == 0 { continue; }
            spans.push(Span::styled(format!("{count}", ), t.stat_value));
            spans.push(Span::styled(format!(" {} ", lane_badge(key)), t.lane_style(key)));
        }
        lines.push(Line::from(spans));
    }

    lines.push(Line::from(""));
    let s = &app.stats;
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{}", s.applied_count), t.decision_applied),
        Span::raw(" applied · "),
        Span::styled(format!("{}", s.watching_count), t.decision_watching),
        Span::raw(" watching · "),
        Span::styled(format!("{}", s.rejected_count), t.decision_rejected),
        Span::raw(" rejected"),
    ]));

    let pending: i64 = s.jobs_by_eval.iter().filter(|(e, _)| e == "pending").map(|(_, c)| c).sum();
    if pending > 0 || s.bespoke_searchable > 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("  Next steps:", t.header)));
        if s.bespoke_searchable > 0 {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("• ", t.status_bespoke),
                Span::raw(format!(
                    "{} bespoke {} need search",
                    s.bespoke_searchable,
                    if s.bespoke_searchable == 1 { "company" } else { "companies" }
                )),
            ]));
        }
        if pending > 0 {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("• ", t.dim),
                Span::raw(format!("{pending} jobs pending evaluation")),
            ]));
        }
    }

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

// ── Top roles — lane-coloured ──────────────────────────────────

fn draw_top_roles(frame: &mut Frame, app: &App, area: Rect) {
    let t = &app.theme;
    let roles = fetch_all_top_roles(app);

    let mut lines = Vec::new();

    if roles.is_empty() {
        lines.push(Line::from(Span::styled("  No SS/S/A graded jobs yet", t.dim)));
    } else {
        for (grade, title, company, lanes_json) in &roles {
            let g_style = t.grade_style(Some(grade.as_str()));
            let lane_key = primary_lane(lanes_json.as_deref());
            let (lane_str, lane_style) = match lane_key.as_deref() {
                Some(key) => (format!("{} ", lane_badge(key)), t.lane_style(key)),
                None => ("—   ".to_string(), t.dim),
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(lane_str, lane_style),
                Span::styled(format!("{grade:<3}"), g_style),
                Span::styled(title.clone(), t.stat_value),
                Span::styled(format!(" — {company}"), t.dim),
            ]));
        }
    }

    let scroll = app.dashboard_scroll;
    let block = Block::bordered()
        .title(format!(" Top Roles ({}) ", roles.len()))
        .title_style(t.title)
        .border_style(Style::default().fg(t.border));

    let line_count = lines.len();
    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(para, area);

    if line_count > 0 {
        let mut scrollbar_state = ScrollbarState::new(line_count)
            .position(app.dashboard_scroll as usize);
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

fn fetch_all_top_roles(app: &App) -> Vec<(String, String, String, Option<String>)> {
    let Ok(conn) = Connection::open(&app.db_path) else { return Vec::new(); };
    let sql = "
        SELECT j.grade, j.title, c.name, j.lanes
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
            stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
                .map(|rows| rows.filter_map(|r| r.ok()).collect())
        })
        .unwrap_or_default()
}
