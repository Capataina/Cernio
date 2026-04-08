use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme::Theme;

/// Renders proportional horizontal grade bars, filling the available width dynamically.
///
/// Each bar shows: `label  ████████  count`
/// Bar width is proportional to count relative to the maximum count in the set.
#[allow(dead_code)]
pub fn render_grade_bars(
    grades: &[(String, i64)],
    theme: &Theme,
    area: Rect,
    frame: &mut Frame,
) {
    if grades.is_empty() || area.height == 0 || area.width < 10 {
        return;
    }

    let max_count = grades.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1);

    // Reserve space for label (4 chars) + padding (2) + count (6 chars) + padding (2) = 14.
    let label_width = 4u16;
    let count_width = 6u16;
    let padding = 4u16;
    let overhead = label_width + count_width + padding;
    let bar_max_width = area.width.saturating_sub(overhead);

    let lines: Vec<Line> = grades
        .iter()
        .map(|(label, count)| {
            let bar_width =
                ((bar_max_width as i64) * count / max_count).max(0) as usize;

            let filled: String = "█".repeat(bar_width);
            let count_str = format!("{count}");

            let style = theme.grade_style(Some(label.as_str()));

            Line::from(vec![
                Span::styled(format!("{label:<4}"), style),
                Span::raw("  "),
                Span::styled(filled, style),
                Span::raw("  "),
                Span::styled(count_str, Style::default()),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
