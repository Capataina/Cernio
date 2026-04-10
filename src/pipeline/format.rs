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

// ══════════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════════
//
// The format pipeline is the most complex pure logic in the repo and
// touches every job description in the database. These tests are
// organised by sub-function (bottom-up) then by end-to-end scenarios
// (top-down), with property tests at the end for invariants that are
// hard to enumerate by hand.
//
// Guiding principles:
// - Every sub-function is tested in isolation. Regressions stay local.
// - Every HTML entity in the decoder table has a matching test.
// - Edge cases cover what we have historically seen in real ATS data:
//   entity-encoded HTML, quoted attributes with '>', LinkedIn tracking
//   tags, nested lists, case-variant tags.
// - No I/O. Pure-function tests only. Integration tests that run
//   `format::run` against a seeded DB live in tests/pipeline_format.rs.

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────
    // decode_entities
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn decode_entities_amp() {
        assert_eq!(decode_entities("Ben &amp; Jerry"), "Ben & Jerry");
    }

    #[test]
    fn decode_entities_nbsp() {
        assert_eq!(decode_entities("a&nbsp;b"), "a b");
    }

    #[test]
    fn decode_entities_lt_gt() {
        assert_eq!(decode_entities("&lt;div&gt;"), "<div>");
    }

    #[test]
    fn decode_entities_quot() {
        assert_eq!(decode_entities("&quot;hi&quot;"), "\"hi\"");
    }

    #[test]
    fn decode_entities_apos_variants() {
        // All three flavours of apostrophe must round-trip to '.
        assert_eq!(decode_entities("don&#39;t"), "don't");
        assert_eq!(decode_entities("don&apos;t"), "don't");
        assert_eq!(decode_entities("don&#x27;t"), "don't");
    }

    #[test]
    fn decode_entities_dashes() {
        assert_eq!(decode_entities("3&ndash;5"), "3–5");
        assert_eq!(decode_entities("yes&mdash;we"), "yes—we");
        assert_eq!(decode_entities("yes&#8212;we"), "yes—we");
        assert_eq!(decode_entities("3&#8211;5"), "3–5");
    }

    #[test]
    fn decode_entities_smart_quotes() {
        assert_eq!(decode_entities("it&rsquo;s"), "it\u{2019}s");
        assert_eq!(decode_entities("&lsquo;hi&rsquo;"), "\u{2018}hi\u{2019}");
        assert_eq!(decode_entities("&ldquo;hi&rdquo;"), "\u{201C}hi\u{201D}");
    }

    #[test]
    fn decode_entities_bullets_ellipsis() {
        assert_eq!(decode_entities("&bull; a"), "• a");
        assert_eq!(decode_entities("&#8226; a"), "• a");
        assert_eq!(decode_entities("wait&hellip;"), "wait…");
    }

    #[test]
    fn decode_entities_symbols() {
        assert_eq!(decode_entities("Acme&reg;"), "Acme\u{00AE}");
        assert_eq!(decode_entities("Brand&trade;"), "Brand\u{2122}");
        assert_eq!(decode_entities("&copy; 2026"), "\u{00A9} 2026");
    }

    #[test]
    fn decode_entities_numeric_decimal() {
        assert_eq!(decode_entities("&#65;&#66;"), "AB");
        assert_eq!(decode_entities("&#10003;"), "\u{2713}");
    }

    #[test]
    fn decode_entities_numeric_hex() {
        assert_eq!(decode_entities("&#x41;"), "A");
        assert_eq!(decode_entities("&#X41;"), "A");
        assert_eq!(decode_entities("&#x2014;"), "—");
    }

    #[test]
    fn decode_entities_invalid_numeric_dropped() {
        // Invalid code points are silently dropped — bad data shouldn't crash.
        assert_eq!(decode_entities("&#notanumber;"), "");
    }

    #[test]
    fn decode_entities_mixed() {
        let input = "Q&amp;A &mdash; say &quot;hi&#33;&quot;";
        assert_eq!(decode_entities(input), "Q&A — say \"hi!\"");
    }

    #[test]
    fn decode_entities_unknown_named_preserved() {
        // We don't recognise &foobar; — it should survive untouched.
        assert_eq!(decode_entities("&foobar;"), "&foobar;");
    }

    #[test]
    fn decode_entities_idempotent_on_plain_text() {
        let input = "plain text with no entities";
        assert_eq!(decode_entities(input), input);
    }

    // ─────────────────────────────────────────────────────────────
    // strip_tags
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn strip_tags_simple() {
        assert_eq!(strip_tags("<p>hi</p>"), "hi");
    }

    #[test]
    fn strip_tags_with_attributes() {
        assert_eq!(strip_tags("<a href=\"x\">link</a>"), "link");
    }

    #[test]
    fn strip_tags_quoted_attribute_with_gt() {
        // This is the nasty case: '>' inside a quoted attribute must not
        // prematurely end the tag. Historically caused content loss.
        let input = "<span data-json='{\"key\":true}'>visible</span>";
        assert_eq!(strip_tags(input), "visible");
    }

    #[test]
    fn strip_tags_quoted_attribute_single_quotes() {
        let input = r#"<div id='>weird'>text</div>"#;
        assert_eq!(strip_tags(input), "text");
    }

    #[test]
    fn strip_tags_nested() {
        assert_eq!(strip_tags("<p><b>bold</b></p>"), "bold");
    }

    #[test]
    fn strip_tags_eats_from_unmatched_lt() {
        // A lone '<' with no matching '>' puts the parser into tag mode and
        // consumes the rest of the input. This is the current behaviour and
        // we rely on it elsewhere — pre-decoding of &lt;/&gt; in
        // `format_description` ensures well-formed HTML reaches this function.
        // Bad ATS payloads with unescaped angle brackets in prose will lose
        // content, which is a known trade-off documented by this test.
        assert_eq!(strip_tags("a < b"), "a ");
    }

    #[test]
    fn strip_tags_empty_input() {
        assert_eq!(strip_tags(""), "");
    }

    #[test]
    fn strip_tags_only_tags() {
        assert_eq!(strip_tags("<p></p><div></div>"), "");
    }

    // ─────────────────────────────────────────────────────────────
    // convert_headings
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn convert_headings_h1() {
        let out = convert_headings("<h1>Responsibilities</h1>");
        assert!(out.contains("RESPONSIBILITIES"));
    }

    #[test]
    fn convert_headings_all_levels() {
        for level in 1..=6 {
            let input = format!("<h{level}>Level {level}</h{level}>");
            let out = convert_headings(&input);
            assert!(
                out.contains(&format!("LEVEL {level}")),
                "h{level} not uppercased: {out:?}"
            );
        }
    }

    #[test]
    fn convert_headings_case_insensitive() {
        let out = convert_headings("<H2>About</H2>");
        assert!(out.contains("ABOUT"));
    }

    #[test]
    fn convert_headings_with_attributes() {
        let out = convert_headings(r#"<h3 class="x">Benefits</h3>"#);
        assert!(out.contains("BENEFITS"));
    }

    #[test]
    fn convert_headings_with_nested_tags() {
        // Inner <strong> should be stripped before uppercasing.
        let out = convert_headings("<h2><strong>Hi</strong> there</h2>");
        assert!(out.contains("HI THERE"));
    }

    #[test]
    fn convert_headings_preserves_non_heading_text() {
        let out = convert_headings("before<h1>X</h1>after");
        assert!(out.contains("before"));
        assert!(out.contains("after"));
        assert!(out.contains('X'));
    }

    #[test]
    fn convert_headings_empty_content_becomes_blank() {
        // Empty headings shouldn't leave a phantom marker.
        let out = convert_headings("<h1></h1>");
        assert!(!out.contains("\n\n\n")); // no triple blank
    }

    // ─────────────────────────────────────────────────────────────
    // convert_inline_tags
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn convert_inline_strong() {
        let out = convert_inline_tags("<strong>bold</strong>", &["strong", "b"], "");
        assert_eq!(out, "bold");
    }

    #[test]
    fn convert_inline_em() {
        let out = convert_inline_tags("<em>italic</em>", &["em", "i"], "");
        assert_eq!(out, "italic");
    }

    #[test]
    fn convert_inline_with_attributes() {
        let out = convert_inline_tags(
            r#"<strong class="x">bold</strong>"#,
            &["strong"],
            "",
        );
        assert_eq!(out, "bold");
    }

    #[test]
    fn convert_inline_case_insensitive() {
        let out = convert_inline_tags("<STRONG>yelling</STRONG>", &["strong"], "");
        assert_eq!(out, "yelling");
    }

    // ─────────────────────────────────────────────────────────────
    // convert_links
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn convert_links_basic() {
        let out = convert_links(r#"<a href="https://example.com">click me</a>"#);
        assert_eq!(out, "click me");
    }

    #[test]
    fn convert_links_preserves_surrounding_text() {
        let out = convert_links(r#"see <a href="x">this</a> for details"#);
        assert_eq!(out, "see this for details");
    }

    #[test]
    fn convert_links_multiple() {
        let out = convert_links(r#"<a href="a">one</a> and <a href="b">two</a>"#);
        assert_eq!(out, "one and two");
    }

    #[test]
    fn convert_links_handles_quoted_gt_in_href() {
        // '>' inside the href value must not end the tag early.
        let out = convert_links(r#"<a href="foo?q=1>2">label</a>"#);
        assert_eq!(out, "label");
    }

    #[test]
    fn convert_links_case_insensitive() {
        let out = convert_links(r#"<A HREF="x">X</A>"#);
        assert_eq!(out, "X");
    }

    #[test]
    fn convert_links_unmatched_open_tag() {
        // Opening tag without closing — shouldn't crash. We drop the opening
        // tag but keep the rest of the string.
        let out = convert_links("<a href=\"x\">unclosed");
        assert!(out.contains("unclosed"));
    }

    #[test]
    fn convert_links_preserves_unicode_text() {
        let out = convert_links(r#"<a href="x">日本語 🌸</a>"#);
        assert_eq!(out, "日本語 🌸");
    }

    #[test]
    fn convert_links_does_not_match_abbr() {
        // <abbr> starts with 'a' but shouldn't be treated as <a>.
        let out = convert_links("<abbr>hi</abbr>");
        assert!(out.contains("hi"));
        assert!(out.contains("<abbr>") || out.contains("abbr"));
    }

    // ─────────────────────────────────────────────────────────────
    // convert_lists
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn convert_lists_unordered() {
        let input = "<ul><li>one</li><li>two</li></ul>";
        let out = convert_lists(input);
        assert!(out.contains("• one"));
        assert!(out.contains("• two"));
    }

    #[test]
    fn convert_lists_ordered_numbers() {
        let input = "<ol><li>first</li><li>second</li><li>third</li></ol>";
        let out = convert_lists(input);
        assert!(out.contains("1. first"));
        assert!(out.contains("2. second"));
        assert!(out.contains("3. third"));
    }

    #[test]
    fn convert_lists_ordered_counter_resets_between_lists() {
        let input = "<ol><li>a</li></ol> gap <ol><li>a</li><li>b</li></ol>";
        let out = convert_lists(input);
        // Counter should restart for the second list.
        let two_occurrences: Vec<_> = out.match_indices("1. a").collect();
        assert_eq!(two_occurrences.len(), 2, "got: {out:?}");
    }

    #[test]
    fn convert_lists_li_with_attributes() {
        let out = convert_lists(r#"<ul><li class="x">item</li></ul>"#);
        assert!(out.contains("• item"));
    }

    #[test]
    fn convert_lists_bare_li_defaults_to_bullet() {
        // <li> outside any list is treated as a bullet.
        let out = convert_lists("<li>orphan</li>");
        assert!(out.contains("• orphan"));
    }

    // ─────────────────────────────────────────────────────────────
    // convert_blocks
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn convert_blocks_p_gets_separators() {
        let out = convert_blocks("<p>one</p><p>two</p>");
        // Both opening and closing tags convert to \n\n, so adjacent paragraphs
        // end up well-separated.
        assert!(out.contains("one"));
        assert!(out.contains("two"));
        assert!(out.contains("\n\n"));
    }

    #[test]
    fn convert_blocks_div() {
        let out = convert_blocks("<div>a</div><div>b</div>");
        assert!(out.contains("a"));
        assert!(out.contains("b"));
        assert!(out.contains("\n\n"));
    }

    #[test]
    fn convert_blocks_all_block_tags() {
        // Every block tag we recognise must be handled.
        for tag in &["p", "div", "section", "article", "header", "footer"] {
            let input = format!("<{tag}>x</{tag}>");
            let out = convert_blocks(&input);
            assert!(out.contains("x"), "tag {tag} lost content: {out:?}");
        }
    }

    #[test]
    fn convert_blocks_case_insensitive() {
        let out = convert_blocks("<P>x</P>");
        assert!(out.contains("x"));
    }

    // ─────────────────────────────────────────────────────────────
    // clean_output
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn clean_output_collapses_multiple_blank_lines() {
        let input = "a\n\n\n\nb";
        assert_eq!(clean_output(input), "a\n\nb");
    }

    #[test]
    fn clean_output_trims_lines() {
        let input = "   a   \n   b   ";
        assert_eq!(clean_output(input), "a\nb");
    }

    #[test]
    fn clean_output_removes_leading_blanks() {
        let input = "\n\n\nhello";
        assert_eq!(clean_output(input), "hello");
    }

    #[test]
    fn clean_output_removes_trailing_blanks() {
        let input = "hello\n\n\n";
        assert_eq!(clean_output(input), "hello");
    }

    #[test]
    fn clean_output_removes_linkedin_tracking() {
        let input = "Job description.\n#LI-Remote\n#LI-POST\nMore content.";
        let out = clean_output(input);
        assert!(!out.contains("#LI-"));
        assert!(out.contains("Job description"));
        assert!(out.contains("More content"));
    }

    #[test]
    fn clean_output_removes_lowercase_linkedin_tracking() {
        let out = clean_output("text\n#li-remote\nmore");
        assert!(!out.contains("#li-"));
    }

    #[test]
    fn clean_output_empty_input() {
        assert_eq!(clean_output(""), "");
    }

    #[test]
    fn clean_output_only_blank_lines() {
        assert_eq!(clean_output("\n\n\n"), "");
    }

    // ─────────────────────────────────────────────────────────────
    // remove_tag_case_insensitive / remove_opening_tag_case_insensitive
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn remove_tag_literal_match() {
        let out = remove_tag_case_insensitive("<P>hi</P>", "</p>", "\n");
        assert_eq!(out, "<P>hi\n");
    }

    #[test]
    fn remove_opening_tag_no_attributes() {
        let out = remove_opening_tag_case_insensitive("<p>hi</p>", "p", "\n");
        assert!(out.contains("\nhi</p>"));
    }

    #[test]
    fn remove_opening_tag_with_attributes() {
        let out = remove_opening_tag_case_insensitive(
            r#"<p class="intro">hi</p>"#,
            "p",
            "\n",
        );
        assert!(out.contains("\nhi</p>"));
    }

    #[test]
    fn remove_opening_tag_does_not_match_prefix() {
        // <div matching <divider must NOT fire — bad match would strip "<divider".
        let out = remove_opening_tag_case_insensitive("<divider>x", "div", "");
        assert!(out.contains("<divider"));
    }

    #[test]
    fn remove_opening_tag_quoted_gt() {
        let out = remove_opening_tag_case_insensitive(
            r#"<p data-x="a>b">content</p>"#,
            "p",
            "",
        );
        assert!(out.contains("content"));
        assert!(!out.contains("data-x"));
    }

    // ─────────────────────────────────────────────────────────────
    // format_description — end-to-end scenarios
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn format_description_plain_text_unchanged() {
        let input = "plain text no html";
        let out = format_description(input);
        assert_eq!(out, "plain text no html");
    }

    #[test]
    fn format_description_simple_paragraph() {
        let input = "<p>Hello, world.</p>";
        let out = format_description(input);
        assert!(out.contains("Hello, world."));
    }

    #[test]
    fn format_description_heading_then_body() {
        let input = "<h2>About</h2><p>We build things.</p>";
        let out = format_description(input);
        assert!(out.contains("ABOUT"));
        assert!(out.contains("We build things."));
        // Heading should be on its own line above the body.
        let about_idx = out.find("ABOUT").unwrap();
        let body_idx = out.find("We build things").unwrap();
        assert!(about_idx < body_idx);
    }

    #[test]
    fn format_description_with_list() {
        let input = "<p>Requirements:</p><ul><li>Rust</li><li>SQL</li></ul>";
        let out = format_description(input);
        assert!(out.contains("Requirements:"));
        assert!(out.contains("• Rust"));
        assert!(out.contains("• SQL"));
    }

    #[test]
    fn format_description_ordered_list() {
        let input = "<ol><li>first</li><li>second</li></ol>";
        let out = format_description(input);
        assert!(out.contains("1. first"));
        assert!(out.contains("2. second"));
    }

    #[test]
    fn format_description_inline_formatting_stripped() {
        let input = "<p>We use <strong>Rust</strong> and <em>Tokio</em>.</p>";
        let out = format_description(input);
        assert_eq!(out, "We use Rust and Tokio.");
    }

    #[test]
    fn format_description_link_becomes_text() {
        let input = r#"<p>See <a href="https://x.com">our blog</a> for more.</p>"#;
        let out = format_description(input);
        assert_eq!(out, "See our blog for more.");
    }

    #[test]
    fn format_description_br_becomes_newline() {
        let input = "line 1<br>line 2<br/>line 3";
        let out = format_description(input);
        assert!(out.contains("line 1"));
        assert!(out.contains("line 2"));
        assert!(out.contains("line 3"));
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines.len() >= 3, "expected 3+ lines, got {lines:?}");
    }

    #[test]
    fn format_description_entity_encoded_html() {
        // Many ATS providers store HTML in entity-encoded form.
        let input = "&lt;p&gt;Hello &amp; welcome&lt;/p&gt;";
        let out = format_description(input);
        assert!(out.contains("Hello & welcome"));
    }

    #[test]
    fn format_description_named_entities_in_content() {
        let input = "<p>3&nbsp;&mdash;&nbsp;5 years</p>";
        let out = format_description(input);
        assert!(out.contains("3 — 5 years") || out.contains("3  —  5 years"));
    }

    #[test]
    fn format_description_linkedin_tracking_stripped() {
        let input = "<p>Join us.</p>\n#LI-Remote\n<p>We're hiring.</p>";
        let out = format_description(input);
        assert!(!out.contains("#LI-"));
        assert!(out.contains("Join us"));
        assert!(out.contains("We're hiring"));
    }

    #[test]
    fn format_description_quoted_attribute_with_gt_survives() {
        let input = r#"<span data-x='{"key":"val>ue"}'>visible text</span>"#;
        let out = format_description(input);
        assert!(out.contains("visible text"));
        assert!(!out.contains("data-x"));
    }

    #[test]
    fn format_description_is_idempotent() {
        // Running format twice should yield the same result. This is the
        // property `cernio format` depends on when it runs on TUI startup.
        let input = "<h2>Role</h2><p>Build <strong>things</strong>. &mdash; team</p>\
                     <ul><li>Rust</li><li>Linux</li></ul>";
        let once = format_description(input);
        let twice = format_description(&once);
        assert_eq!(once, twice, "format_description should be idempotent");
    }

    #[test]
    fn format_description_empty() {
        assert_eq!(format_description(""), "");
    }

    #[test]
    fn format_description_only_tags() {
        let input = "<p></p><div></div><br><br/>";
        let out = format_description(input);
        assert!(out.trim().is_empty(), "expected empty, got {out:?}");
    }

    #[test]
    fn format_description_nested_lists() {
        let input = "<ul><li>a<ul><li>b</li></ul></li></ul>";
        let out = format_description(input);
        assert!(out.contains("• a"));
        assert!(out.contains("• b"));
    }

    #[test]
    fn format_description_preserves_content_inside_heading() {
        // A real-world case: heading with inline tags.
        let input = "<h2>About <strong>Cernio</strong></h2>";
        let out = format_description(input);
        assert!(out.contains("ABOUT CERNIO"));
    }

    #[test]
    fn format_description_real_world_greenhouse_style() {
        // Rough approximation of what Greenhouse boards return.
        let input = r#"
            <div class="posting">
                <h2>About the role</h2>
                <p>We are looking for a <strong>Junior Rust Engineer</strong>.</p>
                <h3>Responsibilities</h3>
                <ul>
                    <li>Write <em>safe</em> Rust.</li>
                    <li>Review PRs.</li>
                </ul>
                <p>See our <a href="https://example.com">careers page</a>.</p>
                <p>#LI-Remote</p>
            </div>
        "#;
        let out = format_description(input);
        assert!(out.contains("ABOUT THE ROLE"));
        assert!(out.contains("Junior Rust Engineer"));
        assert!(out.contains("RESPONSIBILITIES"));
        assert!(out.contains("• Write safe Rust."));
        assert!(out.contains("• Review PRs."));
        assert!(out.contains("See our careers page."));
        assert!(!out.contains("#LI-Remote"));
        assert!(!out.contains("<"));
        assert!(!out.contains('>'));
    }

    // ─────────────────────────────────────────────────────────────
    // Invariants (fast property-style tests using hand-picked inputs)
    // ─────────────────────────────────────────────────────────────

    #[test]
    fn format_description_never_produces_raw_tags() {
        // For a wide range of inputs, output must contain no HTML tags.
        let inputs = [
            "<p>hi</p>",
            "<div><span>nested</span></div>",
            "<a href=\"x\">link</a>",
            "<ul><li>a</li><li>b</li></ul>",
            "<h1>Header</h1><p>Body</p>",
            "<br><br/><br />",
            "<custom-tag data='x'>content</custom-tag>",
            "plain text",
            "",
        ];
        for input in inputs {
            let out = format_description(input);
            assert!(
                !out.contains('<') && !out.contains('>'),
                "output contained angle brackets: input={input:?} output={out:?}"
            );
        }
    }

    #[test]
    fn format_description_never_produces_triple_blank_line() {
        // clean_output must collapse all gaps to a single blank line.
        let inputs = [
            "<p>a</p><p></p><p></p><p>b</p>",
            "a\n\n\n\n\nb",
            "<div></div><div></div><div></div>",
        ];
        for input in inputs {
            let out = format_description(input);
            assert!(
                !out.contains("\n\n\n"),
                "triple blank in output: input={input:?} output={out:?}"
            );
        }
    }

    #[test]
    fn format_description_handles_malformed_html() {
        // Bad input should not panic.
        let inputs = [
            "<p",
            "<<<>>>",
            "<p>unclosed",
            "unopened</p>",
            "<a href=\"",
            "&amp;&lt;&gt;",
            "<h1><h2>double</h1>",
        ];
        for input in inputs {
            let _ = format_description(input); // just must not panic
        }
    }
}
