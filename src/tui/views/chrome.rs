use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Tabs};
use ratatui::Frame;

use crate::tui::app::{App, Focus, SortMode, View};

// ── Tab bar ──────────────────────────────────────────────────────

pub fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
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

    let pipeline_count = app.pipeline_watching.len()
        + app.pipeline_applied.len()
        + app.pipeline_interview.len();

    let titles = vec![
        " Dashboard ".to_string(),
        format!(" Companies ({}) ", app.companies.len()),
        jobs_label,
        format!(" Pipeline ({}) ", pipeline_count),
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

pub fn draw_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    let query = &app.search_query;
    let cursor_pos = query.len();

    let line = Line::from(vec![
        Span::styled(" / ", Style::default().fg(Color::Cyan)),
        Span::raw(query.as_str()),
        Span::styled("█", Style::default().fg(Color::Cyan)),
    ]);

    let bar = Paragraph::new(line);
    frame.render_widget(bar, area);

    #[allow(clippy::cast_possible_truncation)]
    let cx = area.x + 3 + cursor_pos as u16;
    if cx < area.x + area.width {
        frame.set_cursor_position((cx, area.y));
    }
}

// ── Status bar ───────────────────────────────────────────────────

pub fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut keys: Vec<(&str, &str)> = match (app.view, app.focus) {
        (View::Dashboard, _) => vec![
            ("j/k", "scroll"),
            ("D", "clean"),
            ("?", "help"),
            ("q", "quit"),
        ],
        (View::Companies, Focus::List) => vec![
            ("j/k", "navigate"),
            ("Enter", "jobs"),
            ("Tab", "detail"),
            ("/", "search"),
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
            let mut k = vec![
                ("j/k", "navigate"),
                ("w/a/x/i", "decide"),
                ("o", "open"),
                ("/", "search"),
                ("f", if app.focused_mode { "all" } else { "focus" }),
                ("?", "help"),
            ];
            if app.job_filter_company.is_some() {
                k.push(("Esc", "back"));
            }
            k.push(("q", "quit"));
            k
        }
        (View::Jobs, Focus::Detail) => vec![
            ("j/k", "scroll"),
            ("w/a/x/i", "decide"),
            ("o", "open"),
            ("Esc", "back"),
            ("q", "quit"),
        ],
        (View::Pipeline, _) => vec![
            ("j/k", "up/down"),
            ("h/l", "columns"),
            ("w/a/i", "move"),
            ("?", "help"),
            ("q", "quit"),
        ],
    };

    let mut spans: Vec<Span> = keys
        .drain(..)
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(format!(" {key} "), app.theme.help_key),
                Span::styled(format!("{desc}  "), app.theme.dim),
            ]
        })
        .collect();

    let mut right_parts: Vec<Span> = Vec::new();

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

    if !app.search_query.is_empty() && !app.search_mode {
        let count = app.jobs.len();
        right_parts.push(Span::styled(
            format!(" \"{}\" — {} matches ", app.search_query, count),
            Style::default().fg(Color::Cyan),
        ));
    }

    if !right_parts.is_empty() {
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
