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

/// Build the CSS `background` value used by `.row-lane-accent` to encode a
/// row's lane(s) as a left-edge colour stripe. Returns `None` when there are
/// no lanes (caller skips rendering the accent element entirely).
///
/// The horizontal fade-out is applied via `mask-image` in CSS — this helper
/// only emits the vertical-band colour structure:
///
/// - 1 lane  → solid colour
/// - 2 lanes → two vertical bands at 0–50% / 50–100%
/// - 3+      → equal bands across the height (clamped to first 4 for
///   visual sanity; lane #5+ is hidden but the row's lane chip column still
///   shows the full set).
pub fn lane_accent_gradient(lanes_json: Option<&str>) -> Option<String> {
    let lanes = all_lanes(lanes_json);
    if lanes.is_empty() {
        return None;
    }
    let take: Vec<&str> = lanes.iter().take(4).map(|s| s.as_str()).collect();
    if take.len() == 1 {
        // Solid colour — the horizontal fade is CSS-side via mask-image.
        return Some(lane_hex(take[0]).to_string());
    }
    let n = take.len() as f32;
    let mut stops: Vec<String> = Vec::with_capacity(take.len());
    for (i, key) in take.iter().enumerate() {
        let start = (i as f32 / n * 100.0).round() as i32;
        let end = ((i as f32 + 1.0) / n * 100.0).round() as i32;
        stops.push(format!("{} {start}% {end}%", lane_hex(key)));
    }
    Some(format!("linear-gradient(to bottom, {})", stops.join(", ")))
}
