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

    #[test]
    fn test_distribute_empty_specs() {
        let result = distribute(&[], 30);
        assert!(result.is_empty());
    }

    #[test]
    fn test_distribute_single_spec_zero_priority_gets_fill() {
        // One block, no growth → it gets natural height plus a trailing Fill.
        let specs = vec![BlockSpec {
            content_lines: 5,
            min_height: 0,
            grow_priority: 0,
        }];
        let result = distribute(&specs, 30);
        // 1 block + 1 trailing fill = 2 constraints
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], Constraint::Length(7))); // 5 + 2
        assert!(matches!(result[1], Constraint::Fill(1)));
    }

    #[test]
    fn test_distribute_zero_priority_no_fill_when_growing_blocks_present() {
        // Mixed: one growing, one fixed. No trailing fill should be appended.
        let specs = vec![
            BlockSpec { content_lines: 5, min_height: 0, grow_priority: 0 },
            BlockSpec { content_lines: 5, min_height: 0, grow_priority: 1 },
        ];
        let result = distribute(&specs, 30);
        // Two blocks, both Length constraints (no extra Fill).
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], Constraint::Length(_)));
        assert!(matches!(result[1], Constraint::Length(_)));
    }

    #[test]
    fn test_distribute_proportional_to_priority() {
        let specs = vec![
            BlockSpec { content_lines: 0, min_height: 0, grow_priority: 1 },
            BlockSpec { content_lines: 0, min_height: 0, grow_priority: 3 },
        ];
        let result = distribute(&specs, 18);
        // Naturals are both 2, surplus = 14, priority sum = 4.
        // First gets 2 + (14*1/4) = 2 + 3 = 5.
        // Second gets 2 + (14*3/4) = 2 + 10 = 12.
        if let (Constraint::Length(a), Constraint::Length(b)) = (&result[0], &result[1]) {
            assert!(b > a, "higher priority should get more space: {a}, {b}");
        } else {
            panic!("expected Length constraints, got {result:?}");
        }
    }

    #[test]
    fn test_distribute_natural_height_uses_max_of_min_and_content() {
        // min_height is the floor; if content+2 < min_height, min_height wins.
        let specs = vec![BlockSpec {
            content_lines: 1,
            min_height: 10,
            grow_priority: 0,
        }];
        let result = distribute(&specs, 30);
        if let Constraint::Length(h) = result[0] {
            assert!(h >= 10, "expected ≥ 10, got {h}");
        }
    }

    #[test]
    fn test_distribute_exact_fit_uses_naturals() {
        // total_natural == available exactly. Should still degrade to Fill on last.
        let specs = vec![
            BlockSpec { content_lines: 3, min_height: 0, grow_priority: 0 },
            BlockSpec { content_lines: 3, min_height: 0, grow_priority: 0 },
        ];
        // Naturals: 5 + 5 = 10
        let result = distribute(&specs, 10);
        // Last block becomes Fill in the >= branch.
        assert!(matches!(result.last(), Some(Constraint::Fill(1))));
    }

    #[test]
    fn test_distribute_does_not_panic_on_huge_available() {
        let specs = vec![BlockSpec {
            content_lines: 5,
            min_height: 0,
            grow_priority: 1,
        }];
        let _ = distribute(&specs, u16::MAX);
    }
}
