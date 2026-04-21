//! Diagnostic test written by the 2026-04-21 code-health-audit.
//!
//! Proves that `cernio::ats::lever::strip_html` (the quote-aware stripper) correctly
//! handles HTML whose attribute values contain unbalanced `>` characters — for
//! example `data-ccp-props='{"k":">"}'`. This is the input shape that motivated
//! the quote-aware state machine in `lever.rs` and `greenhouse.rs`.
//!
//! The audit also observed that `src/ats/smartrecruiters.rs` and `src/ats/workable.rs`
//! define their own independent `strip_html` functions that DO NOT track quote
//! state — the inner `>` would terminate the tag early and leak attribute content
//! into the output. Those implementations are private so they cannot be called
//! from an integration test; this test locks in the correct behaviour for the
//! one public stripper so future refactors can treat it as the reference.

use cernio::ats::lever::strip_html;

#[test]
fn lever_strip_html_handles_unbalanced_gt_in_quoted_attribute() {
    // Raw JSON-in-attribute that contains a literal '>' between the double
    // quotes. A naive `<...>` state machine that does not track quote state
    // would terminate the tag at the first '>' and leak '"}>visible' into
    // the stripped output.
    let input = r#"<span data-ccp-props='{"k":">"}'>visible</span>"#;
    assert_eq!(strip_html(input), "visible");
}

#[test]
fn lever_strip_html_handles_double_quoted_gt() {
    // Same hazard, but the attribute uses double quotes containing an inner '>'.
    let input = r#"<span data-x="k:>"/>visible after tag"#;
    assert_eq!(strip_html(input), "visible after tag");
}

#[test]
fn lever_strip_html_simple_markup_roundtrip() {
    // Basic sanity — the audit relies on this being stable.
    assert_eq!(strip_html("<p>hello <strong>world</strong>!</p>"), "hello world!");
}

#[test]
fn lever_strip_html_empty_input() {
    assert_eq!(strip_html(""), "");
}

#[test]
fn lever_strip_html_pass_through_lone_gt() {
    // Bare `>` outside a tag is passed through unchanged — correct per HTML5.
    // This test locks the semantics so future refactors don't silently start
    // eating stray `>` characters.
    assert_eq!(strip_html("plain text with > in it"), "plain text with > in it");
}

#[test]
fn lever_strip_html_unbalanced_lt_eats_to_end() {
    // An unmatched `<` opens the tag and everything after it is consumed until
    // a `>` is found (or input ends). Matches the documented behaviour in the
    // Phase-4 test suite's strip_tags comment ("eats-on-unmatched-`<` documented").
    assert_eq!(strip_html("visible then < is eaten"), "visible then ");
}
