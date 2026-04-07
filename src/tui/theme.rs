use ratatui::style::{Color, Modifier, Style};

/// Semantic colour palette using the terminal's ANSI colours.
///
/// By mapping everything to the 16-colour ANSI palette and using `Color::Reset`
/// for backgrounds, the TUI inherits the user's terminal theme — including
/// transparency, custom palettes (Catppuccin, Dracula, Nord, etc.), and dark/light
/// mode. No hardcoded hex values.
pub struct Theme {
    pub border: Color,
    pub border_focused: Color,
    pub selected: Style,
    pub selected_unfocused: Style,
    pub header: Style,
    pub title: Style,
    pub grade_ss: Style,
    pub grade_s: Style,
    pub grade_a: Style,
    pub grade_b: Style,
    pub grade_c: Style,
    pub grade_f: Style,
    pub eval_pending: Style,
    pub eval_evaluating: Style,
    pub eval_strong: Style,
    pub eval_weak: Style,
    pub eval_no_fit: Style,
    pub status_resolved: Style,
    pub status_bespoke: Style,
    pub status_potential: Style,
    pub decision_watching: Style,
    pub decision_applied: Style,
    pub decision_rejected: Style,
    pub dim: Style,
    pub bold: Style,
    pub tab_active: Style,
    pub tab_inactive: Style,
    pub help_key: Style,
    pub help_section: Style,
    pub stat_label: Style,
    pub stat_value: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            border: Color::DarkGray,
            border_focused: Color::Cyan,
            selected: Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
            selected_unfocused: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            header: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            title: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),

            // Job grades (SS–F)
            grade_ss: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            grade_s: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            grade_a: Style::default().fg(Color::Cyan),
            grade_b: Style::default().fg(Color::Yellow),
            grade_c: Style::default().fg(Color::Red),
            grade_f: Style::default().fg(Color::DarkGray),

            // Evaluation status
            eval_pending: Style::default().fg(Color::DarkGray),
            eval_evaluating: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            eval_strong: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            eval_weak: Style::default().fg(Color::Yellow),
            eval_no_fit: Style::default().fg(Color::DarkGray),

            // Company status
            status_resolved: Style::default().fg(Color::Green),
            status_bespoke: Style::default().fg(Color::Yellow),
            status_potential: Style::default().fg(Color::DarkGray),

            // User decisions
            decision_watching: Style::default().fg(Color::Cyan),
            decision_applied: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            decision_rejected: Style::default().fg(Color::DarkGray),

            dim: Style::default().fg(Color::DarkGray),
            bold: Style::default().add_modifier(Modifier::BOLD),
            tab_active: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            tab_inactive: Style::default().fg(Color::DarkGray),
            help_key: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            help_section: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            stat_label: Style::default().fg(Color::DarkGray),
            stat_value: Style::default().add_modifier(Modifier::BOLD),
        }
    }
}

impl Theme {
    pub fn grade_style(&self, grade: Option<&str>) -> Style {
        match grade {
            Some("SS") => self.grade_ss,
            Some("S") => self.grade_s,
            Some("A") => self.grade_a,
            Some("B") => self.grade_b,
            Some("C") => self.grade_c,
            Some("F") => self.grade_f,
            _ => self.dim,
        }
    }

    pub fn eval_style(&self, status: &str) -> Style {
        match status {
            "pending" => self.eval_pending,
            "evaluating" => self.eval_evaluating,
            "strong_fit" => self.eval_strong,
            "weak_fit" => self.eval_weak,
            "no_fit" => self.eval_no_fit,
            _ => self.dim,
        }
    }

    pub fn status_style(&self, status: &str) -> Style {
        match status {
            "resolved" => self.status_resolved,
            "bespoke" => self.status_bespoke,
            "potential" => self.status_potential,
            _ => self.dim,
        }
    }

    pub fn decision_style(&self, decision: Option<&str>) -> Style {
        match decision {
            Some("watching") => self.decision_watching,
            Some("applied") => self.decision_applied,
            Some("rejected") => self.decision_rejected,
            _ => self.dim,
        }
    }
}
