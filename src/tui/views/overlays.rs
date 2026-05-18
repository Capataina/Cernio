use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::app::{App, FilterAxis};

// ── Grade picker popup ───────────────────────────────────────────

pub fn draw_grade_picker(frame: &mut Frame, app: &App) {
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

// ── Bulk action picker popup ─────────────────────────────────────

pub fn draw_bulk_picker(frame: &mut Frame, app: &App) {
    let area = centered_rect_fixed(36, 5, frame.area());

    let t = &app.theme;
    let grades = ["SS", "S", "A", "B", "C", "F"];

    let spans: Vec<Span> = grades
        .iter()
        .enumerate()
        .flat_map(|(i, g)| {
            let style = t.grade_style(Some(g));
            vec![
                Span::styled(format!(" {}:{} ", i + 1, g), style),
                Span::raw(" "),
            ]
        })
        .collect();

    let action = &app.bulk_action;
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(format!("  {action} all jobs of grade:"), t.header)),
        Line::from(spans),
        Line::from(""),
    ];

    let block = Block::bordered()
        .title(format!(" Bulk {} ", action))
        .title_style(t.title)
        .border_style(Style::default().fg(t.border_focused));

    let popup = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });

    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}

// ── Help overlay ─────────────────────────────────────────────────

pub fn draw_help_overlay(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 70, frame.area());

    let t = &app.theme;
    let section_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let rule_style = Style::default().fg(Color::DarkGray);

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    // ── Navigation section ──
    lines.push(section_header("Navigation", section_style, rule_style));
    lines.push(help_line(t, "  j/k  ↑/↓      ", "Navigate list"));
    lines.push(help_line(t, "  g / G         ", "Top / bottom"));
    help_line_push(&mut lines, t, "  Enter / l / → ", "Drill into company jobs");
    help_line_push(&mut lines, t, "  Esc / h / ←   ", "Go back");
    help_line_push(&mut lines, t, "  Tab           ", "Toggle list ↔ detail");
    help_line_push(&mut lines, t, "  1  2  3  4    ", "Switch view");
    lines.push(Line::from(""));

    // ── Decisions section ──
    lines.push(section_header("Decisions", section_style, rule_style));
    help_line_push(&mut lines, t, "  w             ", "Mark watching");
    help_line_push(&mut lines, t, "  a             ", "Mark applied");
    help_line_push(&mut lines, t, "  x             ", "Mark rejected");
    help_line_push(&mut lines, t, "  i             ", "Mark interview");
    help_line_push(&mut lines, t, "  o             ", "Open URL + apply");
    help_line_push(&mut lines, t, "  p             ", "Autofill application form");
    help_line_push(&mut lines, t, "  y             ", "Copy URL to clipboard");
    lines.push(Line::from(""));

    // ── Filtering section ──
    lines.push(section_header("Filtering", section_style, rule_style));
    help_line_push(&mut lines, t, "  /             ", "Search jobs/companies");
    help_line_push(&mut lines, t, "  f             ", "Open filter menu (grades, ATS, status…)");
    help_line_push(&mut lines, t, "  s             ", "Cycle sort mode");
    help_line_push(&mut lines, t, "  [ / ]         ", "Jump prev/next grade section");
    lines.push(Line::from(""));

    // ── Grading section ──
    lines.push(section_header("Grading", section_style, rule_style));
    help_line_push(&mut lines, t, "  Space         ", "Quick peek popup (Jobs view)");
    help_line_push(&mut lines, t, "  Ctrl+G        ", "Group by company (Jobs view)");
    help_line_push(&mut lines, t, "  g             ", "Override grade (Jobs view)");
    help_line_push(&mut lines, t, "  W             ", "Bulk action by grade");
    help_line_push(&mut lines, t, "  e             ", "Export current view");
    help_line_push(&mut lines, t, "  D             ", "Clean database (Dashboard)");
    lines.push(Line::from(""));

    // ── Pipeline section ──
    lines.push(section_header("Pipeline", section_style, rule_style));
    help_line_push(&mut lines, t, "  h / l         ", "Switch column");
    help_line_push(&mut lines, t, "  w / a / i     ", "Move card");
    help_line_push(&mut lines, t, "  o             ", "Open card URL");
    lines.push(Line::from(""));

    // ── General section ──
    lines.push(section_header("General", section_style, rule_style));
    help_line_push(&mut lines, t, "  ?             ", "Toggle this help");
    help_line_push(&mut lines, t, "  q / Ctrl+C    ", "Quit");
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "  Press any key to close",
        t.dim,
    )));
    lines.push(Line::from(""));

    let block = Block::bordered()
        .title(" Help ")
        .title_style(t.title)
        .border_style(Style::default().fg(t.border_focused));

    let help = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });

    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

/// Build a section header line with box-drawing rule characters.
fn section_header<'a>(title: &'a str, title_style: Style, rule_style: Style) -> Line<'a> {
    Line::from(vec![
        Span::styled(" ─── ", rule_style),
        Span::styled(title, title_style),
        Span::styled(" ─────────────────────────", rule_style),
    ])
}

// ── Filter menu popup ────────────────────────────────────────────

pub fn draw_filter_menu(frame: &mut Frame, app: &App) {
    let t = &app.theme;
    let active_chip_fg = Color::Black;
    let active_chip_bg = t.title.fg.unwrap_or(Color::Cyan);
    let inactive_chip = t.dim;
    let cursor_chip_modifier = Modifier::REVERSED | Modifier::BOLD;

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    for (axis_idx, axis) in FilterAxis::ALL.iter().enumerate() {
        let is_focused_row = axis_idx == app.filter_menu_axis;
        let axis_label_style = if is_focused_row {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            t.header
        };

        let mut row_spans: Vec<Span> = vec![
            Span::raw("  "),
            Span::styled(format!("{:<10}", axis.label()), axis_label_style),
            Span::raw(" "),
        ];

        let chips = axis.chips();
        let chip_focused_idx = if is_focused_row { app.filter_menu_chip } else { usize::MAX };
        let active_set = active_set_for(app, *axis);

        for (chip_idx, chip) in chips.iter().enumerate() {
            let is_active = active_set.contains(*chip);
            let is_cursor = chip_idx == chip_focused_idx;

            let base_style = if is_active {
                Style::default()
                    .fg(active_chip_fg)
                    .bg(active_chip_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                inactive_chip
            };
            let style = if is_cursor {
                base_style.add_modifier(cursor_chip_modifier)
            } else {
                base_style
            };

            row_spans.push(Span::styled(format!(" {chip} "), style));
            row_spans.push(Span::raw(" "));
        }

        lines.push(Line::from(row_spans));
        // Wrap-row spacer between axes for readability.
        lines.push(Line::from(""));
    }

    // ── Footer hint line ──
    let footer = Line::from(vec![
        Span::raw("  "),
        Span::styled("j/k", t.help_key),
        Span::styled(" row · ", t.dim),
        Span::styled("h/l", t.help_key),
        Span::styled(" chip · ", t.dim),
        Span::styled("space", t.help_key),
        Span::styled(" toggle · ", t.dim),
        Span::styled("c", t.help_key),
        Span::styled(" clear · ", t.dim),
        Span::styled("r", t.help_key),
        Span::styled(" reset · ", t.dim),
        Span::styled("Esc/f", t.help_key),
        Span::styled(" close ", t.dim),
    ]);
    lines.push(footer);
    lines.push(Line::from(""));

    // Compute popup size based on content width — wide enough for the
    // ATS axis row (the widest), constrained to terminal.
    let frame_area = frame.area();
    let popup_width = 78u16.min(frame_area.width.saturating_sub(4));
    let popup_height = (lines.len() as u16 + 2).min(frame_area.height.saturating_sub(2));
    let area = centered_rect_fixed(popup_width, popup_height, frame_area);

    let active_count = app.filters.active_chip_count();
    let title = if app.filters.is_default() {
        " Filters ".to_string()
    } else {
        format!(" Filters · {active_count} active ")
    };

    let block = Block::bordered()
        .title(title)
        .title_style(t.title)
        .border_style(Style::default().fg(t.border_focused));

    let popup = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });

    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}

fn active_set_for<'a>(app: &'a App, axis: FilterAxis) -> &'a std::collections::HashSet<String> {
    match axis {
        FilterAxis::Grade => &app.filters.grades,
        FilterAxis::Ats => &app.filters.ats,
        FilterAxis::Decision => &app.filters.decisions,
        FilterAxis::Package => &app.filters.package,
        FilterAxis::Archive => &app.filters.archive,
        FilterAxis::Evidence => &app.filters.evidence,
    }
}

// ── Toast notifications ──────────────────────────────────────────

pub fn draw_toasts(frame: &mut Frame, app: &App) {
    if app.toasts.is_empty() {
        return;
    }

    let area = frame.area();
    let t = &app.theme;

    for (i, toast) in app.toasts.iter().enumerate() {
        let width = (toast.message.len() as u16 + 4).min(40);
        let x = area.width.saturating_sub(width + 2);
        let y = area.height.saturating_sub(3 + (i as u16 * 3));

        if y < 4 {
            break;
        }

        let toast_area = Rect::new(x, y, width, 3);
        let block = Block::bordered()
            .border_style(Style::default().fg(t.border_focused));
        let text = Paragraph::new(format!(" {} ", toast.message)).block(block);

        frame.render_widget(Clear, toast_area);
        frame.render_widget(text, toast_area);
    }
}

// ── Helper functions ─────────────────────────────────────────────

fn help_line<'a>(
    t: &'a crate::tui::theme::Theme,
    key: &'a str,
    desc: &'a str,
) -> Line<'a> {
    Line::from(vec![
        Span::styled(key, t.help_key),
        Span::styled(desc, Style::default()),
    ])
}

fn help_line_push<'a>(
    lines: &mut Vec<Line<'a>>,
    t: &'a crate::tui::theme::Theme,
    key: &'a str,
    desc: &'a str,
) {
    lines.push(help_line(t, key, desc));
}

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
