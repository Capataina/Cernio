mod companies;
mod dashboard;
mod jobs;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use super::app::{App, Focus, View};

/// Main draw entry point — renders tabs, content area, status bar, and overlays.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let areas = Layout::vertical([
        Constraint::Length(3), // tab bar
        Constraint::Fill(1),  // content
        Constraint::Length(1), // status bar
    ])
    .split(frame.area());

    draw_tabs(frame, app, areas[0]);

    match app.view {
        View::Dashboard => dashboard::draw(frame, app, areas[1]),
        View::Companies => companies::draw(frame, app, areas[1]),
        View::Jobs => jobs::draw(frame, app, areas[1]),
    }

    draw_status_bar(frame, app, areas[2]);

    if app.show_help {
        draw_help_overlay(frame, app);
    }
}

// ── Tab bar ──────────────────────────────────────────────────────

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec![
        format!(" Dashboard "),
        format!(" Companies ({}) ", app.stats.total_companies),
        format!(" Jobs ({}) ", app.stats.total_jobs),
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

// ── Status bar ───────────────────────────────────────────────────

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let keys = match (app.view, app.focus) {
        (View::Dashboard, _) => vec![
            ("1-3", "view"),
            ("?", "help"),
            ("q", "quit"),
        ],
        (View::Companies, Focus::List) => vec![
            ("j/k", "navigate"),
            ("Enter", "view jobs"),
            ("Tab", "detail"),
            ("o", "open"),
            ("?", "help"),
            ("q", "quit"),
        ],
        (View::Companies, Focus::Detail) => vec![
            ("j/k", "scroll"),
            ("Tab", "list"),
            ("o", "open"),
            ("?", "help"),
            ("q", "quit"),
        ],
        (View::Jobs, Focus::List) => {
            let mut keys = vec![
                ("j/k", "navigate"),
                ("w", "watch"),
                ("a", "applied"),
                ("x", "reject"),
                ("o", "open"),
                ("Tab", "detail"),
            ];
            if app.job_filter_company.is_some() {
                keys.push(("Esc", "back"));
            }
            keys.push(("q", "quit"));
            keys
        }
        (View::Jobs, Focus::Detail) => vec![
            ("j/k", "scroll"),
            ("w", "watch"),
            ("a", "applied"),
            ("x", "reject"),
            ("o", "open"),
            ("Esc", "back"),
            ("q", "quit"),
        ],
    };

    let spans: Vec<Span> = keys
        .into_iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(format!(" {key} "), app.theme.help_key),
                Span::styled(format!("{desc}  "), app.theme.dim),
            ]
        })
        .collect();

    let bar = Paragraph::new(Line::from(spans));
    frame.render_widget(bar, area);
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

/// Return a centered sub-rect of `area`.
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
