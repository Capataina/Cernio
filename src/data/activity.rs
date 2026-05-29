//! Activity timeline aggregation.
//!
//! Collapses runs of similar adjacent events into a single "group entry" so the
//! Activity view stays readable when 1000+ events arrive at once (e.g. a
//! `cernio clean` pass that archives 700 jobs, a console wipe of all jobs, a
//! grade-jobs batch that grades 1000 jobs in a single run).
//!
//! Grouping rules:
//!   - Adjacent events (after timeline ordering) that share
//!     `(event_type, source)` AND occurred within `MAX_GAP_SECONDS` of each
//!     other collapse into one group.
//!   - Groups with fewer than `MIN_GROUP_SIZE` rows render as individual rows
//!     instead — small clusters are not noisy and shouldn't hide detail.
//!
//! These thresholds are tuned to keep "normal" interactivity (one applied
//! decision, one regrade, one company add) visible per-row while collapsing
//! batch operations.

use crate::data::models::ActivityEntry;

const MAX_GAP_SECONDS: i64 = 10;
const MIN_GROUP_SIZE: usize = 3;

#[derive(Debug, Clone)]
pub enum ActivityGroup {
    Single(ActivityEntry),
    Collapsed {
        event_type: String,
        source: String,
        first_at: String,
        last_at: String,
        date: String,
        entries: Vec<ActivityEntry>,
    },
}

impl ActivityGroup {
    pub fn count(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Collapsed { entries, .. } => entries.len(),
        }
    }

    pub fn date(&self) -> &str {
        match self {
            Self::Single(e) => &e.date,
            Self::Collapsed { date, .. } => date,
        }
    }

    /// Stable key for tracking expand/collapse state across refreshes.
    pub fn key(&self) -> String {
        match self {
            Self::Single(e) => format!("single:{}:{}", e.occurred_at, e.event_type),
            Self::Collapsed { event_type, source, first_at, last_at, .. } => {
                format!("group:{event_type}:{source}:{first_at}:{last_at}")
            }
        }
    }
}

/// Group an ordered (newest-first) activity timeline into rendering groups.
pub fn group_timeline(entries: &[ActivityEntry]) -> Vec<ActivityGroup> {
    if entries.is_empty() {
        return Vec::new();
    }

    let mut groups: Vec<ActivityGroup> = Vec::new();
    let mut buffer: Vec<ActivityEntry> = Vec::new();

    let flush = |buffer: &mut Vec<ActivityEntry>, groups: &mut Vec<ActivityGroup>| {
        if buffer.is_empty() {
            return;
        }
        if buffer.len() < MIN_GROUP_SIZE {
            for e in buffer.drain(..) {
                groups.push(ActivityGroup::Single(e));
            }
        } else {
            // Buffer is newest-first; first_at is the OLDEST event time, last_at is newest.
            let newest = &buffer[0];
            let oldest = &buffer[buffer.len() - 1];
            let event_type = newest.event_type.clone();
            let source = newest.source.clone();
            let first_at = oldest.occurred_at.clone();
            let last_at = newest.occurred_at.clone();
            let date = newest.date.clone();
            let entries = buffer.drain(..).collect();
            groups.push(ActivityGroup::Collapsed {
                event_type,
                source,
                first_at,
                last_at,
                date,
                entries,
            });
        }
    };

    for entry in entries {
        if let Some(last) = buffer.last() {
            let same_kind = last.event_type == entry.event_type && last.source == entry.source;
            let in_window = within_window(&last.occurred_at, &entry.occurred_at);
            if same_kind && in_window {
                buffer.push(entry.clone());
                continue;
            }
            flush(&mut buffer, &mut groups);
        }
        buffer.push(entry.clone());
    }
    flush(&mut buffer, &mut groups);

    groups
}

fn within_window(a: &str, b: &str) -> bool {
    let pa = chrono::NaiveDateTime::parse_from_str(a, "%Y-%m-%d %H:%M:%S").ok();
    let pb = chrono::NaiveDateTime::parse_from_str(b, "%Y-%m-%d %H:%M:%S").ok();
    match (pa, pb) {
        (Some(x), Some(y)) => (x - y).num_seconds().abs() <= MAX_GAP_SECONDS,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(at: &str, ty: &str, src: &str, lbl: &str) -> ActivityEntry {
        ActivityEntry {
            occurred_at: at.to_string(),
            date: at[..10].to_string(),
            event_type: ty.to_string(),
            subject_label: lbl.to_string(),
            lane: None,
            grade: None,
            source: src.to_string(),
            detail_json: None,
        }
    }

    #[test]
    fn singles_stay_singles() {
        let entries = vec![
            entry("2026-05-30 14:00:00", "decision.applied", "tui", "Job A"),
        ];
        let groups = group_timeline(&entries);
        assert_eq!(groups.len(), 1);
        assert!(matches!(groups[0], ActivityGroup::Single(_)));
    }

    #[test]
    fn small_runs_stay_individual() {
        // 2 adjacent matching events — under MIN_GROUP_SIZE, stay individual.
        let entries = vec![
            entry("2026-05-30 14:00:02", "decision.applied", "tui", "Job B"),
            entry("2026-05-30 14:00:01", "decision.applied", "tui", "Job A"),
        ];
        let groups = group_timeline(&entries);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn large_runs_collapse() {
        let mut entries = Vec::new();
        for i in 0..50 {
            entries.push(entry(
                &format!("2026-05-30 14:00:{:02}", 50 - i),
                "job.archived",
                "cli:clean",
                &format!("Job {i}"),
            ));
        }
        let groups = group_timeline(&entries);
        assert_eq!(groups.len(), 1);
        if let ActivityGroup::Collapsed { entries, .. } = &groups[0] {
            assert_eq!(entries.len(), 50);
        } else {
            panic!("expected collapsed group");
        }
    }

    #[test]
    fn different_sources_dont_merge() {
        let mut entries = Vec::new();
        for i in 0..5 {
            entries.push(entry(
                &format!("2026-05-30 14:00:{:02}", 20 - i),
                "job.archived",
                "cli:clean",
                &format!("Job {i}"),
            ));
        }
        for i in 0..5 {
            entries.push(entry(
                &format!("2026-05-30 14:00:{:02}", 10 - i),
                "job.archived",
                "tui",
                &format!("Job other {i}"),
            ));
        }
        let groups = group_timeline(&entries);
        assert_eq!(groups.len(), 2, "different sources should not merge");
    }

    #[test]
    fn time_gap_breaks_run() {
        let entries = vec![
            entry("2026-05-30 15:00:00", "job.archived", "cli:clean", "Job B"),
            entry("2026-05-30 14:00:00", "job.archived", "cli:clean", "Job A"),
            // 1 hour gap → should not merge.
        ];
        let groups = group_timeline(&entries);
        assert_eq!(groups.len(), 2);
    }
}
