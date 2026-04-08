mod companies;
mod dashboard;
mod jobs;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use super::app::{App, Focus, SortMode, View};

/// Main draw entry point — renders tabs, content area, status bar, and overlays.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let bottom_height = if app.search_mode { 2 } else { 1 };

    let areas = Layout::vertical([
        Constraint::Length(3),       // tab bar
        Constraint::Fill(1),         // content
        Constraint::Length(bottom_height), // status bar (+ search bar)
    ])
    .split(frame.area());

    draw_tabs(frame, app, areas[0]);

    match app.view {
        View::Dashboard => dashboard::draw(frame, app, areas[1]),
        View::Companies => companies::draw(frame, app, areas[1]),
        View::Jobs => jobs::draw(frame, app, areas[1]),
    }

    if app.search_mode {
        // Split bottom area: search bar on top, status bar below.
        let bottom = Layout::vertical([
            Constraint::Length(1), // search bar
            Constraint::Length(1), // status bar
        ])
        .split(areas[2]);
        draw_search_bar(frame, app, bottom[0]);
        draw_status_bar(frame, app, bottom[1]);
    } else {
        draw_status_bar(frame, app, areas[2]);
    }

    // Toasts — ephemeral notifications in the bottom-right.
    draw_toasts(frame, app);

    // Grade picker popup.
    if app.show_grade_picker {
        draw_grade_picker(frame, app);
    }

    if app.show_help {
        draw_help_overlay(frame, app);
    }
}

// ── Tab bar ──────────────────────────────────────────────────────

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let focused_indicator = if app.focused_mode { " [FOCUSED]" } else { "" };

    let jobs_label = if app.focused_mode {
        format!(
            " Jobs ({}/{}){} ",
            app.jobs.len(),
            app.total_jobs_unfiltered,
            focused_indicator
        )
    } else {
        format!(" Jobs ({}) ", app.jobs.len())
    };

    let titles = vec![
        " Dashboard ".to_string(),
        format!(" Companies ({}) ", app.companies.len()),
        jobs_label,
    ];

    let tabs = Tabs::new(titles)
        .block(
            Block::bordered()
                .title(" cernio ")
                .title_style(app.theme.title)
                .border_style(Style::default().fg(app.theme.border)),
        )
        .select(app.view.index())
        .style(app.theme.tab_inactive)
        .highlight_style(app.theme.tab_active)
        .divider("│");

    frame.render_widget(tabs, area);
}

// ── Search bar ───────────────────────────────────────────────────

fn draw_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    let query = &app.search_query;
    let cursor_pos = query.len();

    let line = Line::from(vec![
        Span::styled(" / ", Style::default().fg(Color::Cyan)),
        Span::raw(query.as_str()),
        Span::styled("█", Style::default().fg(Color::Cyan)),
    ]);

    let bar = Paragraph::new(line);
    frame.render_widget(bar, area);

    // Set cursor position for blinking cursor effect.
    #[allow(clippy::cast_possible_truncation)]
    let cx = area.x + 3 + cursor_pos as u16;
    if cx < area.x + area.width {
        frame.set_cursor_position((cx, area.y));
    }
}

// ── Status bar ───────────────────────────────────────────────────

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut keys: Vec<(&str, &str)> = match (app.view, app.focus) {
        (View::Dashboard, _) => vec![
            ("1-3", "view"),
            ("j/k", "scroll"),
            ("D", "clean db"),
            ("?", "help"),
            ("q", "quit"),
        ],
        (View::Companies, Focus::List) => vec![
            ("j/k", "navigate"),
            ("Enter", "view jobs"),
            ("Tab", "detail"),
            ("o", "open"),
            ("y", "copy"),
            ("/", "search"),
            ("?", "help"),
            ("q", "quit"),
        ],
        (View::Companies, Focus::Detail) => vec![
            ("j/k", "scroll"),
            ("Tab", "list"),
            ("o", "open"),
            ("y", "copy"),
            ("?", "help"),
            ("q", "quit"),
        ],
        (View::Jobs, Focus::List) => {
            let mut k = vec![
                ("j/k", "navigate"),
                ("w", "watch"),
                ("a", "applied"),
                ("x", "reject"),
                ("o", "open"),
                ("y", "copy"),
                ("s", "sort"),
                ("/", "search"),
                ("g", "grade"),
                ("f", if app.focused_mode { "show all" } else { "focus" }),
                ("Tab", "detail"),
            ];
            if app.job_filter_company.is_some() {
                k.push(("Esc", "back"));
            }
            k.push(("q", "quit"));
            k
        }
        (View::Jobs, Focus::Detail) => vec![
            ("j/k", "scroll"),
            ("w", "watch"),
            ("a", "applied"),
            ("x", "reject"),
            ("o", "open"),
            ("y", "copy"),
            ("Esc", "back"),
            ("q", "quit"),
        ],
    };

    // Build left side (key bindings).
    let mut spans: Vec<Span> = keys
        .drain(..)
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(format!(" {key} "), app.theme.help_key),
                Span::styled(format!("{desc}  "), app.theme.dim),
            ]
        })
        .collect();

    // Build right side indicators.
    let mut right_parts: Vec<Span> = Vec::new();

    // Sort mode indicator (Jobs view only).
    if app.view == View::Jobs {
        let sort_label = match app.sort_mode {
            SortMode::ByGrade => "grade",
            SortMode::ByCompany => "company",
            SortMode::ByDate => "date",
            SortMode::ByLocation => "location",
        };
        right_parts.push(Span::styled(
            format!(" sort:{sort_label} "),
            app.theme.dim,
        ));
    }

    // Search filter indicator.
    if !app.search_query.is_empty() && !app.search_mode {
        let count = app.jobs.len();
        right_parts.push(Span::styled(
            format!(" \"{}\" — {} matches ", app.search_query, count),
            Style::default().fg(Color::Cyan),
        ));
    }

    if !right_parts.is_empty() {
        // Push right-aligned indicators by calculating padding.
        let left_len: usize = spans.iter().map(|s| s.content.len()).sum();
        let right_len: usize = right_parts.iter().map(|s| s.content.len()).sum();
        let padding = (area.width as usize).saturating_sub(left_len + right_len);
        if padding > 0 {
            spans.push(Span::raw(" ".repeat(padding)));
        }
        spans.extend(right_parts);
    }

    let bar = Paragraph::new(Line::from(spans));
    frame.render_widget(bar, area);
}

// ── Grade picker popup ───────────────────────────────────────────

fn draw_grade_picker(frame: &mut Frame, app: &App) {
    let area = centered_rect_fixed(30, 5, frame.area());

    let t = &app.theme;
    let grades = ["SS", "S", "A", "B", "C", "F"];

    let spans: Vec<Span> = grades
        .iter()
        .flat_map(|g| {
            let style = t.grade_style(Some(g));
            vec![
                Span::styled(format!(" {g} "), style),
                Span::raw(" "),
            ]
        })
        .collect();

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Pick a grade:", t.header)),
        Line::from(spans),
        Line::from(""),
    ];

    let block = Block::bordered()
        .title(" Grade Override ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border_focused));

    let popup = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });

    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}

// ── Help overlay ─────────────────────────────────────────────────

fn draw_help_overlay(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 70, frame.area());

    let t = &app.theme;

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Navigation", t.help_section)),
        Line::from(""),
        help_line(t, "  j / k / ↑ / ↓", "Move selection up/down"),
        help_line(t, "  g / G", "Jump to top / bottom"),
        help_line(t, "  Enter / l / →", "Drill into company jobs"),
        help_line(t, "  Esc / h / ←", "Go back"),
        help_line(t, "  Tab", "Toggle list / detail focus"),
        help_line(t, "  1  2  3", "Dashboard / Companies / Jobs"),
        Line::from(""),
        Line::from(Span::styled("  Actions", t.help_section)),
        Line::from(""),
        help_line(t, "  w", "Mark job as watching"),
        help_line(t, "  a", "Mark job as applied"),
        help_line(t, "  x", "Mark job as rejected"),
        help_line(t, "  o", "Open URL in browser"),
        help_line(t, "  y", "Copy URL to clipboard"),
        help_line(t, "  D", "Clean database (from dashboard)"),
        help_line(t, "  f", "Toggle focused mode (hide F/C)"),
        help_line(t, "  s", "Cycle sort mode"),
        help_line(t, "  /", "Search / filter"),
        help_line(t, "  g", "Override grade (in Jobs view)"),
        Line::from(""),
        Line::from(Span::styled("  General", t.help_section)),
        Line::from(""),
        help_line(t, "  ?", "Toggle this help"),
        help_line(t, "  q / Ctrl+C", "Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "  Press any key to close",
            t.dim,
        )),
        Line::from(""),
    ];

    let block = Block::bordered()
        .title(" Help ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border_focused));

    let help = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });

    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

fn help_line<'a>(
    t: &'a super::theme::Theme,
    key: &'a str,
    desc: &'a str,
) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{key:<18}"), t.help_key),
        Span::styled(desc, Style::default()),
    ])
}

/// Return a centered sub-rect of `area` using percentages.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(v[1])[1]
}

/// Return a centered sub-rect of `area` using fixed dimensions.
fn centered_rect_fixed(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(
        x,
        y,
        width.min(area.width),
        height.min(area.height),
    )
}

// ── Toast notifications ──────────────────────────────────────────

fn draw_toasts(frame: &mut Frame, app: &App) {
    if app.toasts.is_empty() {
        return;
    }

    let area = frame.area();
    let t = &app.theme;

    // Stack toasts from bottom-right, going up.
    for (i, toast) in app.toasts.iter().enumerate() {
        let width = (toast.message.len() as u16 + 4).min(40);
        let x = area.width.saturating_sub(width + 2);
        let y = area.height.saturating_sub(3 + (i as u16 * 3));

        if y < 4 {
            break; // Don't stack into the tab bar.
        }

        let toast_area = Rect::new(x, y, width, 3);
        let block = Block::bordered()
            .border_style(Style::default().fg(t.border_focused));
        let text = Paragraph::new(format!(" {} ", toast.message)).block(block);

        frame.render_widget(Clear, toast_area);
        frame.render_widget(text, toast_area);
    }
}
