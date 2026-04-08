use chrono::NaiveDate;

/// Clean raw HTML description text into readable plain text.
///
/// Handles block-level elements (p, br, li, h1-h6, div) by inserting newlines,
/// converts list items to bullet points, strips remaining tags, decodes
/// HTML entities (named and numeric), and collapses excessive whitespace.
pub fn clean_description(raw: &str) -> String {
    let mut text = raw.to_string();

    // Replace <br> variants with newline.
    text = text.replace("<br>", "\n");
    text = text.replace("<br/>", "\n");
    text = text.replace("<br />", "\n");
    text = text.replace("<BR>", "\n");

    // Block-level tags: insert newlines around them.
    let block_tags = [
        "p", "div", "h1", "h2", "h3", "h4", "h5", "h6",
        "tr", "table", "section", "article", "header", "footer",
    ];

    for tag in &block_tags {
        let close_lower = format!("</{tag}>");
        let close_upper = format!("</{}>", tag.to_uppercase());
        text = text.replace(&close_lower, "\n");
        text = text.replace(&close_upper, "\n");

        let open_lower = format!("<{tag}");
        let open_upper = format!("<{}", tag.to_uppercase());
        text = text.replace(&open_lower, &format!("\n<{tag}"));
        text = text.replace(&open_upper, &format!("\n<{}", tag.to_uppercase()));
    }

    // Convert list items to bullet points.
    let mut processed = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            let mut tag = String::from('<');
            for tc in chars.by_ref() {
                tag.push(tc);
                if tc == '>' { break; }
            }
            let tag_lower = tag.to_lowercase();
            if tag_lower.starts_with("<li") && tag_lower.contains('>') {
                processed.push_str("\n  • ");
            } else if tag_lower == "</li>" {
                // skip
            } else if tag_lower == "<ul>" || tag_lower == "</ul>"
                || tag_lower == "<ol>" || tag_lower == "</ol>"
            {
                processed.push('\n');
            } else {
                processed.push_str(&tag);
            }
        } else {
            processed.push(c);
        }
    }
    text = processed;

    // Strip remaining HTML tags.
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                result.push(' ');
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    text = result;

    // Decode named HTML entities.
    text = text
        .replace("&amp;", "&")
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&#x27;", "'")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&bull;", "•")
        .replace("&hellip;", "…")
        .replace("&#8226;", "•")
        .replace("&#8211;", "–")
        .replace("&#8212;", "—")
        .replace("&rsquo;", "\u{2019}")
        .replace("&lsquo;", "\u{2018}")
        .replace("&rdquo;", "\u{201D}")
        .replace("&ldquo;", "\u{201C}")
        .replace("&trade;", "\u{2122}")
        .replace("&reg;", "\u{00AE}")
        .replace("&copy;", "\u{00A9}");

    // Decode numeric HTML entities (&#NNN; and &#xHHH;).
    let mut decoded = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' && chars.peek() == Some(&'#') {
            chars.next(); // consume '#'
            let mut num_str = String::new();
            let is_hex = chars.peek() == Some(&'x') || chars.peek() == Some(&'X');
            if is_hex { chars.next(); }
            for nc in chars.by_ref() {
                if nc == ';' { break; }
                num_str.push(nc);
            }
            let code = if is_hex {
                u32::from_str_radix(&num_str, 16).ok()
            } else {
                num_str.parse::<u32>().ok()
            };
            if let Some(ch) = code.and_then(char::from_u32) {
                decoded.push(ch);
            }
        } else {
            decoded.push(c);
        }
    }
    text = decoded;

    // Trim each line and collapse multiple blank lines into one.
    let mut output_lines: Vec<String> = Vec::new();
    let mut prev_blank = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_blank && !output_lines.is_empty() {
                output_lines.push(String::new());
                prev_blank = true;
            }
        } else {
            output_lines.push(trimmed.to_string());
            prev_blank = false;
        }
    }

    while output_lines.last().is_some_and(|l| l.is_empty()) {
        output_lines.pop();
    }

    output_lines.join("\n")
}

/// Convert an ISO date/timestamp string to a relative human-readable form.
///
/// Returns "3 days ago", "2 weeks ago", etc. Falls back to "Apr 7" format
/// when the date cannot be parsed or is far in the past.
pub fn relative_date(iso: &str) -> String {
    let date = iso
        .get(..10)
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let Some(date) = date else {
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
pub fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}
