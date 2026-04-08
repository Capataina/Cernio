use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::app::{App, PipelineCard, PipelineColumn};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    let cols = Layout::horizontal([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(area);

    draw_column(
        frame,
        app,
        cols[0],
        "Watching",
        &app.pipeline_watching,
        PipelineColumn::Watching,
        app.pipeline_selections[0],
    );
    draw_column(
        frame,
        app,
        cols[1],
        "Applied",
        &app.pipeline_applied,
        PipelineColumn::Applied,
        app.pipeline_selections[1],
    );
    draw_column(
        frame,
        app,
        cols[2],
        "Interview",
        &app.pipeline_interview,
        PipelineColumn::Interview,
        app.pipeline_selections[2],
    );
}

fn draw_column(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    title: &str,
    cards: &[PipelineCard],
    column: PipelineColumn,
    selection: usize,
) {
    let t = &app.theme;
    let is_active = app.pipeline_column == column;

    let border_color = if is_active {
        t.border_focused
    } else {
        t.border
    };

    let block = Block::bordered()
        .title(format!(" {} ({}) ", title, cards.len()))
        .title_style(if is_active { t.title } else { t.dim })
        .border_style(Style::default().fg(border_color));

    if cards.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("  No jobs here yet", t.dim)),
        ])
        .block(block);
        frame.render_widget(msg, area);
        return;
    }

    let mut lines = Vec::new();

    for (i, card) in cards.iter().enumerate() {
        let grade = card.grade.as_deref().unwrap_or("—");
        let grade_style = t.grade_style(card.grade.as_deref());

        let is_selected = is_active && i == selection;

        let prefix = if is_selected { "▸ " } else { "  " };

        let mut card_style = Style::default();
        if is_selected {
            card_style = card_style
                .bg(ratatui::style::Color::DarkGray)
                .add_modifier(Modifier::BOLD);
        }

        lines.push(Line::from(vec![
            Span::styled(prefix, card_style),
            Span::styled(format!("{grade:<3}"), if is_selected { card_style } else { grade_style }),
            Span::styled(&card.title, card_style),
        ]));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(&card.company, t.dim),
        ]));

        // Separator between cards (except last).
        if i < cards.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(para, area);
}
