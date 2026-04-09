use rusqlite::Connection;

/// Format all job descriptions and fit assessments in the database.
///
/// Converts raw HTML descriptions to clean, readable plain text and
/// normalises whitespace in fit assessments. Operates on ALL jobs
/// (active and archived) since the raw data should be clean everywhere.
pub fn run(conn: &Connection, dry_run: bool) {
    println!("── Format job descriptions ──\n");

    let (total, desc_changed, assess_changed) = format_all(conn, dry_run);

    let prefix = if dry_run { "[dry-run] " } else { "" };
    println!("{prefix}Scanned {total} jobs.");
    println!("{prefix}Descriptions formatted: {desc_changed}");
    println!("{prefix}Fit assessments cleaned: {assess_changed}");
}

/// Silent variant for use in TUI startup — no stdout output.
pub fn run_silent(conn: &Connection) {
    format_all(conn, false);
}

/// Core formatting logic. Returns (total, desc_changed, assess_changed).
fn format_all(conn: &Connection, dry_run: bool) -> (usize, u64, u64) {
    let mut stmt = conn
        .prepare("SELECT id, raw_description, fit_assessment FROM jobs")
        .expect("failed to prepare");

    let rows: Vec<(i64, Option<String>, Option<String>)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .expect("failed to query")
        .filter_map(|r| r.ok())
        .collect();

    let total = rows.len();
    let mut desc_changed = 0u64;
    let mut assess_changed = 0u64;

    for (id, raw_desc, fit_assess) in &rows {
        if let Some(desc) = raw_desc {
            let formatted = format_description(desc);
            if formatted != *desc {
                if !dry_run {
                    conn.execute(
                        "UPDATE jobs SET raw_description = ?1 WHERE id = ?2",
                        rusqlite::params![formatted, id],
                    )
                    .expect("failed to update description");
                }
                desc_changed += 1;
            }
        }

        if let Some(assess) = fit_assess {
            let cleaned = clean_whitespace(assess);
            if cleaned != *assess {
                if !dry_run {
                    conn.execute(
                        "UPDATE jobs SET fit_assessment = ?1 WHERE id = ?2",
                        rusqlite::params![cleaned, id],
                    )
                    .expect("failed to update fit_assessment");
                }
                assess_changed += 1;
            }
        }
    }

    (total, desc_changed, assess_changed)
}

/// Convert an HTML job description to clean, readable plain text.
///
/// Preserves document structure: headings become UPPERCASE lines, lists
/// become bullet points, paragraphs get clean separation. Handles both
/// actual HTML (`<p>`) and entity-encoded HTML (`&lt;p&gt;`).
fn format_description(raw: &str) -> String {
    // Many ATS providers store entity-encoded HTML (&lt;p&gt; instead of <p>).
    // Detect and decode this first so the HTML processing pipeline works.
    let decoded = if raw.contains("&lt;") || raw.contains("&gt;") {
        raw.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
    } else {
        raw.to_string()
    };

    // If there's no HTML at all, just clean whitespace.
    if !decoded.contains('<') {
        return clean_plaintext(raw);
    }

    let raw = &decoded;

    let mut text = raw.to_string();

    // ── Phase 1: Structural conversion (before tag stripping) ──

    // Normalise <br> variants to newlines.
    for br in &["<br>", "<br/>", "<br />", "<BR>", "<BR/>", "<BR />"] {
        text = text.replace(br, "\n");
    }

    // Convert headings to UPPERCASE markers.
    text = convert_headings(&text);

    // Convert bold/strong text.
    text = convert_inline_tags(&text, &["strong", "b"], "");

    // Convert italic/em text (just strip the tags, keep content).
    text = convert_inline_tags(&text, &["em", "i"], "");

    // Convert links: <a href="url">text</a> → text
    text = convert_links(&text);

    // Convert list items to bullet points.
    text = convert_lists(&text);

    // Insert paragraph breaks around block elements.
    text = convert_blocks(&text);

    // ── Phase 2: Strip remaining HTML tags ──
    text = strip_tags(&text);

    // ── Phase 3: Decode HTML entities ──
    text = decode_entities(&text);

    // ── Phase 4: Clean up whitespace and tracking tags ──
    text = clean_output(&text);

    text
}

/// Convert <h1>-<h6> tags to UPPERCASE section headers.
fn convert_headings(text: &str) -> String {
    let mut result = text.to_string();

    for level in 1..=6 {
        let open = format!("<h{level}");
        let close = format!("</h{level}>");

        // Process each heading instance.
        loop {
            let lower = result.to_lowercase();
            let Some(start) = lower.find(&open) else { break };
            let Some(tag_end) = result[start..].find('>') else { break };
            let content_start = start + tag_end + 1;
            let Some(close_pos) = lower[content_start..].find(&close) else { break };
            let close_pos = content_start + close_pos;

            let content = strip_tags(&result[content_start..close_pos]);
            let content = decode_entities(&content).trim().to_string();
            let heading = if content.is_empty() {
                String::new()
            } else {
                format!("\n\n{}\n", content.to_uppercase())
            };

            result = format!(
                "{}{}{}",
                &result[..start],
                heading,
                &result[close_pos + close.len()..]
            );
        }
    }

    result
}

/// Strip inline tags (strong, b, em, i) while keeping their content.
fn convert_inline_tags(text: &str, tags: &[&str], _wrapper: &str) -> String {
    let mut result = text.to_string();

    for tag in tags {
        // Remove closing tags.
        let close = format!("</{tag}>");
        result = remove_tag_case_insensitive(&result, &close, "");

        // Remove opening tags (may have attributes).
        result = remove_opening_tag_case_insensitive(&result, tag, "");
    }

    result
}

/// Convert <a href="url">text</a> to just text.
/// Uses byte-based processing to avoid char/byte index mismatches.
fn convert_links(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let lower = text.to_lowercase();
    let bytes = text.as_bytes();
    let lower_bytes = lower.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Look for <a (case-insensitive) followed by whitespace or >.
        if i + 2 < len && lower_bytes[i] == b'<' && lower_bytes[i + 1] == b'a'
            && (i + 2 >= len || lower_bytes[i + 2].is_ascii_whitespace() || lower_bytes[i + 2] == b'>')
        {
            // Find the end of the opening <a ...> tag, handling quoted attributes.
            let mut tag_end = i;
            let mut in_quote = false;
            let mut quote_ch = b'"';
            let mut j = i;
            while j < len {
                if in_quote {
                    if bytes[j] == quote_ch { in_quote = false; }
                } else if bytes[j] == b'"' || bytes[j] == b'\'' {
                    in_quote = true;
                    quote_ch = bytes[j];
                } else if bytes[j] == b'>' {
                    tag_end = j;
                    break;
                }
                j += 1;
            }

            // Find </a> after the tag content.
            let content_start = tag_end + 1;
            if content_start < len {
                if let Some(close_offset) = lower[content_start..].find("</a>") {
                    let close_pos = content_start + close_offset;
                    // Extract the link text (between > and </a>).
                    result.push_str(&text[content_start..close_pos]);
                    i = close_pos + 4; // skip </a>
                    continue;
                }
            }
            // No closing tag found — skip just the opening tag.
            i = tag_end + 1;
        } else {
            // Safe: push one character at a time respecting UTF-8 boundaries.
            if let Some(ch) = text[i..].chars().next() {
                result.push(ch);
                i += ch.len_utf8();
            } else {
                i += 1;
            }
        }
    }

    result
}

/// Convert <ul>/<ol>/<li> structures to clean bullet points.
fn convert_lists(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_ordered_list = false;
    let mut item_counter = 0u32;

    while let Some(c) = chars.next() {
        if c == '<' {
            let mut tag = String::from('<');
            for tc in chars.by_ref() {
                tag.push(tc);
                if tc == '>' { break; }
            }
            let tag_lower = tag.to_lowercase();

            if tag_lower.starts_with("<li") && tag_lower.contains('>') {
                if in_ordered_list {
                    item_counter += 1;
                    result.push_str(&format!("\n{item_counter}. "));
                } else {
                    result.push_str("\n  • ");
                }
            } else if tag_lower == "</li>" {
                // skip
            } else if tag_lower.starts_with("<ol") {
                in_ordered_list = true;
                item_counter = 0;
                result.push('\n');
            } else if tag_lower == "</ol>" {
                in_ordered_list = false;
                result.push('\n');
            } else if tag_lower.starts_with("<ul") {
                result.push('\n');
            } else if tag_lower == "</ul>" {
                result.push('\n');
            } else {
                result.push_str(&tag);
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Insert newlines around block-level elements (p, div, etc.).
fn convert_blocks(text: &str) -> String {
    let mut result = text.to_string();

    let block_tags = ["p", "div", "section", "article", "header", "footer"];

    for tag in &block_tags {
        // Replace closing tags with double newline.
        result = remove_tag_case_insensitive(&result, &format!("</{tag}>"), "\n\n");

        // Replace opening tags (with attributes) with double newline.
        result = remove_opening_tag_case_insensitive(&result, tag, "\n\n");
    }

    result
}

/// Strip all remaining HTML tags, handling quoted attributes correctly.
fn strip_tags(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    let mut quote_char: Option<char> = None;

    for ch in text.chars() {
        if in_tag {
            match quote_char {
                Some(q) if ch == q => quote_char = None,
                Some(_) => {}
                None if ch == '"' || ch == '\'' => quote_char = Some(ch),
                None if ch == '>' => {
                    in_tag = false;
                }
                None => {}
            }
        } else if ch == '<' {
            in_tag = true;
            quote_char = None;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Decode named and numeric HTML entities.
fn decode_entities(text: &str) -> String {
    let mut t = text.to_string();

    // Named entities (most common first).
    t = t.replace("&amp;", "&");
    t = t.replace("&nbsp;", " ");
    t = t.replace("&lt;", "<");
    t = t.replace("&gt;", ">");
    t = t.replace("&quot;", "\"");
    t = t.replace("&#39;", "'");
    t = t.replace("&apos;", "'");
    t = t.replace("&#x27;", "'");
    t = t.replace("&mdash;", "—");
    t = t.replace("&ndash;", "–");
    t = t.replace("&bull;", "•");
    t = t.replace("&hellip;", "…");
    t = t.replace("&#8226;", "•");
    t = t.replace("&#8211;", "–");
    t = t.replace("&#8212;", "—");
    t = t.replace("&rsquo;", "\u{2019}");
    t = t.replace("&lsquo;", "\u{2018}");
    t = t.replace("&rdquo;", "\u{201D}");
    t = t.replace("&ldquo;", "\u{201C}");
    t = t.replace("&trade;", "\u{2122}");
    t = t.replace("&reg;", "\u{00AE}");
    t = t.replace("&copy;", "\u{00A9}");

    // Numeric entities (&#NNN; and &#xHHH;).
    let mut decoded = String::with_capacity(t.len());
    let mut chars = t.chars().peekable();

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

    decoded
}

/// Clean the final output: collapse whitespace, remove tracking tags, trim.
fn clean_output(text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut prev_blank = false;

    for line in text.lines() {
        let trimmed = line.trim();

        // Skip LinkedIn/ATS tracking tags.
        if trimmed.starts_with("#LI-") || trimmed.starts_with("#li-") {
            continue;
        }

        if trimmed.is_empty() {
            if !prev_blank && !lines.is_empty() {
                lines.push(String::new());
                prev_blank = true;
            }
        } else {
            lines.push(trimmed.to_string());
            prev_blank = false;
        }
    }

    // Remove trailing blank lines.
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}

/// Clean a plain text description (no HTML) — fix run-together lines and whitespace.
fn clean_plaintext(text: &str) -> String {
    clean_output(text)
}

/// Clean whitespace in fit assessments (already plain text).
fn clean_whitespace(text: &str) -> String {
    clean_output(text)
}

// ── Helper functions for case-insensitive tag manipulation ──

/// Remove a specific closing tag (case-insensitive) and replace with the given string.
fn remove_tag_case_insensitive(text: &str, tag: &str, replacement: &str) -> String {
    let tag_lower = tag.to_lowercase();
    let mut result = String::with_capacity(text.len());
    let mut remaining = text;

    loop {
        let lower_remaining = remaining.to_lowercase();
        if let Some(pos) = lower_remaining.find(&tag_lower) {
            result.push_str(&remaining[..pos]);
            result.push_str(replacement);
            remaining = &remaining[pos + tag.len()..];
        } else {
            result.push_str(remaining);
            break;
        }
    }

    result
}

/// Remove an opening tag (which may have attributes) case-insensitively.
fn remove_opening_tag_case_insensitive(text: &str, tag_name: &str, replacement: &str) -> String {
    let open_pattern = format!("<{tag_name}");
    let mut result = String::with_capacity(text.len());
    let mut remaining = text;

    loop {
        let lower_remaining = remaining.to_lowercase();
        if let Some(pos) = lower_remaining.find(&open_pattern) {
            // Verify this is actually a tag start (next char is whitespace, >, or /).
            let after = pos + open_pattern.len();
            if after < remaining.len() {
                let next_char = remaining.as_bytes()[after];
                if next_char != b' ' && next_char != b'>' && next_char != b'/'
                    && next_char != b'\t' && next_char != b'\n'
                {
                    // Not a real tag match (e.g., <div matching <divider).
                    result.push_str(&remaining[..pos + open_pattern.len()]);
                    remaining = &remaining[pos + open_pattern.len()..];
                    continue;
                }
            }

            result.push_str(&remaining[..pos]);
            result.push_str(replacement);

            // Skip to the end of the tag (past the >), handling quoted attributes.
            let tag_content = &remaining[pos..];
            let mut in_quote = false;
            let mut quote_ch = '"';
            let mut end = 0;
            for (j, ch) in tag_content.char_indices() {
                if in_quote {
                    if ch == quote_ch { in_quote = false; }
                } else if ch == '"' || ch == '\'' {
                    in_quote = true;
                    quote_ch = ch;
                } else if ch == '>' {
                    end = j + 1;
                    break;
                }
            }
            if end == 0 { end = tag_content.len(); }
            remaining = &remaining[pos + end..];
        } else {
            result.push_str(remaining);
            break;
        }
    }

    result
}
