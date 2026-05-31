//! Filter-strip definitions, query parsing, URL builders, and chip rendering.

use maud::{html, Markup};
use std::collections::HashSet;

use crate::data::lane::lane_hex;

use super::page::JobsQuery;

// ── Axis definitions ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(super) struct AxisDef {
    pub(super) key: &'static str,    // URL param name
    pub(super) label: &'static str,  // sidebar label
    pub(super) chips: &'static [(&'static str, &'static str)], // (value, display)
    pub(super) kind: ChipKind,
}

#[derive(Clone, Copy)]
pub(super) enum ChipKind {
    Lane,      // coloured pill with dot
    Grade,     // outline pill, grade colour
    Plain,     // monochrome enum pill
    Segmented, // mutually-exclusive segmented control (active/archived, etc.)
}

pub(super) const AXES: &[AxisDef] = &[
    AxisDef {
        key: "lane",
        label: "Lane",
        chips: &[
            ("big-tech", "Big Tech"),
            ("ai-ml", "AI / ML"),
            ("hft", "HFT"),
            ("crypto-mm", "Crypto MM"),
            ("bank-strats", "Bank Strats"),
            ("systems-infra", "Systems Infra"),
            ("devtools", "Devtools"),
            ("fintech", "Fintech"),
        ],
        kind: ChipKind::Lane,
    },
    AxisDef {
        key: "grade",
        label: "Grade",
        chips: &[
            ("SS", "SS"),
            ("S", "S"),
            ("A", "A"),
            ("B", "B"),
            ("C", "C"),
            ("F", "F"),
            ("?", "ungraded"),
        ],
        kind: ChipKind::Grade,
    },
    AxisDef {
        key: "decision",
        label: "Decision",
        chips: &[
            ("none", "untouched"),
            ("watching", "watching"),
            ("applied", "applied"),
            ("interview", "interview"),
            ("rejected", "rejected"),
        ],
        kind: ChipKind::Plain,
    },
    AxisDef {
        key: "archive",
        label: "Archive",
        chips: &[
            ("active", "active"),
            ("archived", "archived"),
        ],
        kind: ChipKind::Segmented,
    },
    AxisDef {
        key: "ats",
        label: "ATS",
        chips: &[
            ("greenhouse", "greenhouse"),
            ("ashby", "ashby"),
            ("lever", "lever"),
            ("workable", "workable"),
            ("smartrecruiters", "smartrec"),
            ("workday", "workday"),
            ("eightfold", "eightfold"),
            ("bespoke", "bespoke"),
        ],
        kind: ChipKind::Plain,
    },
    AxisDef {
        key: "posted",
        label: "Posted",
        chips: &[
            ("7d", "≤ 7d"),
            ("30d", "7–30d"),
            ("90d", "30–90d"),
            ("old", "> 90d"),
            ("unknown", "unknown"),
        ],
        kind: ChipKind::Plain,
    },
    AxisDef {
        key: "sponsor",
        label: "Sponsor",
        chips: &[
            ("yes", "yes"),
            ("no", "no"),
            ("unknown", "unknown"),
        ],
        kind: ChipKind::Segmented,
    },
];

fn parse_csv(s: &Option<String>) -> HashSet<String> {
    let mut out = HashSet::new();
    if let Some(raw) = s {
        for part in raw.split(',') {
            let t = part.trim();
            if !t.is_empty() {
                out.insert(t.to_string());
            }
        }
    }
    out
}

/// Current set for an axis given the query — applies the archive default.
pub(super) fn axis_set(q: &JobsQuery, axis: &AxisDef) -> HashSet<String> {
    let raw = match axis.key {
        "lane" => &q.lane,
        "grade" => &q.grade,
        "decision" => &q.decision,
        "archive" => &q.archive,
        "ats" => &q.ats,
        "posted" => &q.posted,
        "sponsor" => &q.sponsor,
        _ => &None,
    };
    let mut set = parse_csv(raw);
    // Archive defaults to {active} when not specified.
    if axis.key == "archive" && raw.is_none() {
        set.insert("active".to_string());
    }
    set
}

/// Build the URL produced by toggling `value` in `axis` against the current query.
fn toggle_href(q: &JobsQuery, axis: &AxisDef, value: &str) -> String {
    // Re-emit every axis; replace the one being toggled with the new CSV.
    let mut parts: Vec<(&str, String)> = Vec::new();
    for a in AXES {
        let current = axis_set(q, a);
        let new_set: HashSet<String> = if a.key == axis.key {
            let mut s = current.clone();
            if s.contains(value) {
                s.remove(value);
            } else {
                s.insert(value.to_string());
            }
            s
        } else {
            current
        };
        // Skip default archive (single "active") to keep URLs clean.
        let is_default = a.key == "archive"
            && new_set.len() == 1
            && new_set.contains("active");
        if !new_set.is_empty() && !is_default {
            let mut vals: Vec<&str> = new_set.iter().map(|s| s.as_str()).collect();
            vals.sort();
            parts.push((a.key, vals.join(",")));
        }
    }
    if parts.is_empty() {
        "/jobs".to_string()
    } else {
        let qs = parts
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencode(v)))
            .collect::<Vec<_>>()
            .join("&");
        format!("/jobs?{qs}")
    }
}

/// Build a click-navigation URL that *sets* a fixed set of axes (rather than
/// toggling). Used by heatmap cells / funnel rows / top-list rows.
pub(super) fn set_href(pairs: &[(&str, &str)]) -> String {
    if pairs.is_empty() {
        return "/jobs".to_string();
    }
    let qs = pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencode(v)))
        .collect::<Vec<_>>()
        .join("&");
    format!("/jobs?{qs}")
}

fn urlencode(s: &str) -> String {
    // Only commas + safe ASCII; encode comma → %2C, space → %20, ? → %3F.
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ',' => "%2C".to_string(),
            ' ' => "%20".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

pub(super) fn render_axis(q: &JobsQuery, axis: &AxisDef) -> Markup {
    let active = axis_set(q, axis);
    match axis.kind {
        ChipKind::Segmented => {
            // Render as a segmented control rather than a flexbox of chips.
            // Picks one of seg-good / seg-bad styling based on axis identity.
            let group_class = match axis.key {
                "sponsor" => "seg-group seg-good",
                _ => "seg-group",
            };
            html! {
                div.filter-axis {
                    span.filter-axis-label { (axis.label) }
                    div class=(group_class) {
                        @for (val, disp) in axis.chips.iter() {
                            @let is_active = active.contains(*val);
                            @let href = toggle_href(q, axis, val);
                            a class="seg" href=(href) data-active=(is_active) { (disp) }
                        }
                    }
                }
            }
        }
        _ => {
            html! {
                div.filter-axis {
                    span.filter-axis-label { (axis.label) }
                    div.chips {
                        @for (val, disp) in axis.chips.iter() {
                            @let is_active = active.contains(*val);
                            @let href = toggle_href(q, axis, val);
                            @let (klass, style) = chip_class_and_style(axis.kind, val, is_active);
                            a class=(klass) href=(href) style=(style) data-active=(is_active) {
                                (disp)
                            }
                        }
                    }
                }
            }
        }
    }
}

fn chip_class_and_style(kind: ChipKind, val: &str, active: bool) -> (String, String) {
    let base = if active { "chip chip-active" } else { "chip" };
    match kind {
        ChipKind::Lane => {
            let kind_class = format!("{base} chip-lane");
            let style = format!("--lane-color: {}", lane_hex(val));
            (kind_class, style)
        }
        ChipKind::Grade => {
            let grade_class = match val {
                "SS" => "chip-grade-ss",
                "S"  => "chip-grade-s",
                "A"  => "chip-grade-a",
                "B"  => "chip-grade-b",
                "C"  => "chip-grade-c",
                "F"  => "chip-grade-f",
                _    => "chip-grade-ungraded",
            };
            (format!("{base} chip-grade {grade_class}"), String::new())
        }
        ChipKind::Plain => (format!("{base} chip-plain"), String::new()),
        ChipKind::Segmented => (base.to_string(), String::new()),
    }
}
