use chrono::NaiveDate;

/// Strip HTML entities and tags, collapse whitespace, and trim.
#[allow(dead_code)]
pub fn clean_description(text: &str) -> String {
    let mut result = text.to_string();

    // Replace common HTML entities.
    result = result.replace("&amp;", "&");
    result = result.replace("&nbsp;", " ");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&#39;", "'");
    result = result.replace("&apos;", "'");
    result = result.replace("&quot;", "\"");

    // Strip leftover HTML tags.
    let mut cleaned = String::with_capacity(result.len());
    let mut in_tag = false;
    for c in result.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => cleaned.push(c),
            _ => {}
        }
    }

    // Collapse runs of more than two newlines into two.
    let mut collapsed = String::with_capacity(cleaned.len());
    let mut newline_count = 0u32;
    for c in cleaned.chars() {
        if c == '\n' {
            newline_count += 1;
            if newline_count <= 2 {
                collapsed.push(c);
            }
        } else {
            newline_count = 0;
            collapsed.push(c);
        }
    }

    // Trim each line and the whole string.
    collapsed
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Convert an ISO date/timestamp string to a relative human-readable form.
///
/// Returns "3 days ago", "2 weeks ago", etc. Falls back to "Apr 7" format
/// when the date cannot be parsed or is far in the past.
#[allow(dead_code)]
pub fn relative_date(iso: &str) -> String {
    // Try to parse various ISO formats.
    let date = iso
        .get(..10)
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let Some(date) = date else {
        // Fallback: return the input trimmed to 10 chars.
        return iso.chars().take(10).collect();
    };

    let today = chrono::Local::now().date_naive();
    let diff = today.signed_duration_since(date);
    let days = diff.num_days();

    match days {
        0 => "today".to_string(),
        1 => "1 day ago".to_string(),
        2..=6 => format!("{days} days ago"),
        7..=13 => "1 week ago".to_string(),
        14..=27 => format!("{} weeks ago", days / 7),
        28..=59 => "1 month ago".to_string(),
        60..=364 => format!("{} months ago", days / 30),
        _ => date.format("%b %-d").to_string(),
    }
}

/// UTF-8 safe string truncation with ellipsis.
#[allow(dead_code)]
pub fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}
