use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::app::App;

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

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("  Navigation", t.help_section)),
        Line::from(""),
        help_line(t, "  j / k / ↑ / ↓", "Move selection up/down"),
        help_line(t, "  g / G", "Jump to top / bottom (Companies)"),
        help_line(t, "  Enter / l / →", "Drill into company jobs"),
        help_line(t, "  Esc / h / ←", "Go back"),
        help_line(t, "  Tab", "Toggle list / detail focus"),
        help_line(t, "  1  2  3  4", "Dashboard / Companies / Jobs / Pipeline"),
        Line::from(""),
        Line::from(Span::styled("  Actions", t.help_section)),
        Line::from(""),
        help_line(t, "  w", "Mark job as watching"),
        help_line(t, "  a", "Mark job as applied"),
        help_line(t, "  x", "Mark job as rejected"),
        help_line(t, "  i", "Mark job as interview"),
        help_line(t, "  o", "Open URL in browser (+ mark applied)"),
        help_line(t, "  p", "Autofill application form in Chrome"),
        help_line(t, "  y", "Copy URL to clipboard"),
        help_line(t, "  e", "Export current view to markdown"),
        help_line(t, "  D", "Clean database (from dashboard)"),
        help_line(t, "  f", "Toggle focused mode (hide F/C)"),
        help_line(t, "  A", "Toggle show archived items"),
        help_line(t, "  s", "Cycle sort mode"),
        help_line(t, "  /", "Search / filter"),
        help_line(t, "  g", "Override grade (in Jobs view)"),
        help_line(t, "  [ / ]", "Jump to prev/next grade section"),
        Line::from(""),
        Line::from(Span::styled("  Pipeline", t.help_section)),
        Line::from(""),
        help_line(t, "  h / l", "Move between columns"),
        help_line(t, "  w / a / i", "Move card to watching/applied/interview"),
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
        Span::styled(format!("{key:<18}"), t.help_key),
        Span::styled(desc, Style::default()),
    ])
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
