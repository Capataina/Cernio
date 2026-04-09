use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::Frame;

use crate::tui::app::{App, PipelineCard, PipelineColumn};

pub fn draw(frame: &mut Frame, app: &mut App, area: Rect) {
    // Proportional columns: size by card count, with a minimum width for empty columns.
    let w = app.pipeline_watching.len().max(1) as u32;
    let a = app.pipeline_applied.len().max(1) as u32;
    let i = app.pipeline_interview.len().max(1) as u32;
    // Give empty columns a small fixed width, non-empty columns scale by content.
    let min_col = 20_u16;
    let constraints = if app.pipeline_watching.is_empty() && app.pipeline_interview.is_empty() {
        // Only applied has content — give it most space.
        vec![
            Constraint::Length(min_col),
            Constraint::Fill(1),
            Constraint::Length(min_col),
        ]
    } else if app.pipeline_interview.is_empty() {
        // Watching + Applied have content, Interview empty.
        let total = w + a;
        vec![
            Constraint::Ratio(w, total),
            Constraint::Ratio(a, total),
            Constraint::Length(min_col),
        ]
    } else {
        // All have content — distribute proportionally.
        let total = w + a + i;
        vec![
            Constraint::Ratio(w, total),
            Constraint::Ratio(a, total),
            Constraint::Ratio(i, total),
        ]
    };
    let cols = Layout::horizontal(constraints).split(area);

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

    // Inner height after borders (top + bottom = 2 rows).
    let visible_height = area.height.saturating_sub(2) as usize;

    // Build one-line-per-card format: "  {grade:<3} {title} — {company}"
    // Max content width = column width minus borders (2) minus prefix (2) minus grade (3) minus spacing.
    let inner_width = area.width.saturating_sub(2) as usize; // subtract left+right border

    let mut lines = Vec::with_capacity(cards.len());

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

        // Build the text portion: "{title} — {company}"
        let text_part = format!("{} — {}", card.title, card.company);
        // Available width for text: inner_width - prefix(2) - grade(3) - space(1)
        let max_text = inner_width.saturating_sub(6);
        let truncated = crate::tui::widgets::text_utils::truncate_chars(&text_part, max_text);

        lines.push(Line::from(vec![
            Span::styled(prefix, card_style),
            Span::styled(
                format!("{grade:<3}"),
                if is_selected { card_style } else { grade_style },
            ),
            Span::styled(truncated, card_style),
        ]));
    }

    // Scroll to keep selection visible (centre the selection in the viewport).
    let scroll_offset = if cards.len() > visible_height {
        selection.saturating_sub(visible_height / 2) as u16
    } else {
        0
    };

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset, 0));

    frame.render_widget(para, area);

    // Scrollbar for pipeline column.
    if !cards.is_empty() {
        let mut scrollbar_state = ScrollbarState::new(cards.len())
            .position(selection);
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
