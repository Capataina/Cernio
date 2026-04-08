use ratatui::layout::Constraint;

/// A block requesting vertical space in a dynamic layout.
pub struct BlockSpec {
    /// Minimum lines of content (not including border).
    pub content_lines: u16,
    /// Minimum height (including border). Block won't shrink below this.
    pub min_height: u16,
    /// Growth priority: higher = gets more surplus space. 0 = never grows.
    pub grow_priority: u16,
}

/// Distribute `available` rows among blocks based on content, minimum height, and grow priority.
///
/// Each block gets at least `max(min_height, content_lines + 2)` (the +2 accounts for borders).
/// Surplus space is distributed proportionally to `grow_priority`.
/// Returns a `Vec<Constraint>` suitable for `Layout::vertical()`.
pub fn distribute(specs: &[BlockSpec], available: u16) -> Vec<Constraint> {
    if specs.is_empty() {
        return vec![];
    }

    // Calculate natural height for each block (content + 2 for border).
    let naturals: Vec<u16> = specs
        .iter()
        .map(|s| s.min_height.max(s.content_lines.saturating_add(2)))
        .collect();

    let total_natural: u16 = naturals.iter().sum();

    if total_natural >= available {
        // Not enough space — give each block its natural height, let last one absorb.
        let mut constraints: Vec<Constraint> = naturals
            .iter()
            .map(|&h| Constraint::Length(h))
            .collect();
        if let Some(last) = constraints.last_mut() {
            *last = Constraint::Fill(1);
        }
        return constraints;
    }

    // Surplus space to distribute.
    let surplus = available - total_natural;
    let total_priority: u16 = specs.iter().map(|s| s.grow_priority).sum();

    if total_priority == 0 {
        // No one wants to grow — give natural heights plus a trailing Fill.
        let mut constraints: Vec<Constraint> = naturals
            .iter()
            .map(|&h| Constraint::Length(h))
            .collect();
        constraints.push(Constraint::Fill(1));
        return constraints;
    }

    // Distribute surplus proportionally.
    let heights: Vec<u16> = specs
        .iter()
        .zip(naturals.iter())
        .map(|(spec, &natural)| {
            if spec.grow_priority == 0 {
                natural
            } else {
                let extra = (surplus as u32 * spec.grow_priority as u32
                    / total_priority as u32) as u16;
                natural + extra
            }
        })
        .collect();

    heights.iter().map(|&h| Constraint::Length(h)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distribute_basic() {
        let specs = vec![
            BlockSpec { content_lines: 5, min_height: 0, grow_priority: 0 },
            BlockSpec { content_lines: 3, min_height: 0, grow_priority: 1 },
        ];
        let result = distribute(&specs, 30);
        // First block: 5 + 2 = 7, no growth.
        // Second block: 3 + 2 = 5 + surplus.
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_distribute_respects_min_height() {
        let specs = vec![
            BlockSpec { content_lines: 2, min_height: 10, grow_priority: 0 },
            BlockSpec { content_lines: 2, min_height: 0, grow_priority: 1 },
        ];
        let result = distribute(&specs, 30);
        assert_eq!(result.len(), 2);
        // First block should be at least min_height=10.
        if let Constraint::Length(h) = result[0] {
            assert!(h >= 10);
        }
    }

    #[test]
    fn test_distribute_overflow() {
        let specs = vec![
            BlockSpec { content_lines: 20, min_height: 0, grow_priority: 0 },
            BlockSpec { content_lines: 20, min_height: 0, grow_priority: 0 },
        ];
        let result = distribute(&specs, 30);
        // Total natural = 44, available = 30. Last becomes Fill.
        assert!(matches!(result.last(), Some(Constraint::Fill(1))));
    }
}
