//! Lane keys, parsing, and presentation-agnostic helpers.

/// Canonical lane keys — display order across every interface.
pub const LANE_KEYS: [&str; 8] = [
    "big-tech",
    "ai-ml",
    "hft",
    "crypto-mm",
    "bank-strats",
    "systems-infra",
    "devtools",
    "fintech",
];

/// Short 3-char lane badge used in compact tables.
pub fn lane_badge(key: &str) -> &'static str {
    match key {
        "big-tech" => "BTH",
        "ai-ml" => "AIM",
        "hft" => "HFT",
        "crypto-mm" => "CMM",
        "bank-strats" => "BNK",
        "systems-infra" => "SYS",
        "devtools" => "DEV",
        "fintech" => "FIN",
        _ => "—  ",
    }
}

/// Long human label.
pub fn lane_label(key: &str) -> &'static str {
    match key {
        "big-tech" => "Big Tech",
        "ai-ml" => "AI / ML",
        "hft" => "HFT",
        "crypto-mm" => "Crypto MM",
        "bank-strats" => "Bank Strats",
        "systems-infra" => "Systems Infra",
        "devtools" => "Devtools",
        "fintech" => "Fintech",
        _ => "—",
    }
}

/// Hex colour for CSS / charts (matches the ANSI lane palette in `tui::theme`).
pub fn lane_hex(key: &str) -> &'static str {
    match key {
        "big-tech" => "#4f7cff",
        "ai-ml" => "#b66cf0",
        "hft" => "#ff5c5c",
        "crypto-mm" => "#ffc94a",
        "bank-strats" => "#3acfe5",
        "systems-infra" => "#52d678",
        "devtools" => "#7ea8ff",
        "fintech" => "#7adf9a",
        _ => "#888888",
    }
}

/// Parse a JSON-array lanes field ("[\"hft\",\"ai-ml\"]") into its primary lane.
pub fn primary_lane(lanes_json: Option<&str>) -> Option<String> {
    let raw = lanes_json?;
    let cleaned = raw.replace(['[', ']', '"'], "");
    let first = cleaned.split(',').next()?.trim();
    if first.is_empty() {
        None
    } else {
        Some(first.to_string())
    }
}

/// Parse a JSON-array lanes field into all its lane keys.
pub fn all_lanes(lanes_json: Option<&str>) -> Vec<String> {
    let Some(raw) = lanes_json else { return Vec::new() };
    raw.replace(['[', ']', '"'], "")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
