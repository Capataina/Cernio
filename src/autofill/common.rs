//! Shared CDP helpers for ATS-driving autofill.
//!
//! Two layers:
//!   - Real-keystroke / real-click helpers built on `Element::type_str`,
//!     `Element::click`, and `DOM.setFileInputFiles`. These generate
//!     genuine browser events and fire React's synthetic event system
//!     natively — fixing the bug noted in the old code where direct
//!     `.value = ...` assignment didn't propagate to React state.
//!   - Legacy JS-based fallback helpers (`fill_field`, `fill_by_strategies`,
//!     `fill_custom_questions`) preserved for non-Greenhouse providers and
//!     for emergency fallback if the schema-driven path fails.

use std::collections::HashMap;
use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::dom::SetFileInputFilesParams;
use chromiumoxide::page::Page;
use futures::StreamExt;

/// Chrome executable path on macOS.
const CHROME_PATH: &str =
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

/// Launch a headed Chrome browser and navigate to the given URL.
/// Returns the browser handle and the page.
///
/// The browser is visible (not headless) so the user can see the form
/// being filled and take over for review and submission.
pub async fn launch_and_navigate(url: &str) -> Result<(Browser, Page), String> {
    crate::tel!("chrome_config_build", "chrome_path": CHROME_PATH);

    // chromiumoxide's DEFAULT_ARGS includes `--enable-automation`, which makes
    // Chrome show the "Chrome is being controlled by automated test software"
    // infobar and trips reCAPTCHA's bot heuristics. We disable the default
    // arg set and manually re-add the defaults we actually want, EXCEPT
    // `--enable-automation`. The remaining flags are taken from chromiumoxide
    // 0.9.1's DEFAULT_ARGS list (see chromiumoxide/src/browser/config.rs).
    let config = BrowserConfig::builder()
        .chrome_executable(CHROME_PATH)
        .with_head()
        .disable_default_args()
        // Re-added defaults (everything in chromiumoxide DEFAULT_ARGS except
        // `enable-automation`):
        .arg("--disable-background-networking")
        .arg("--enable-features=NetworkService,NetworkServiceInProcess")
        .arg("--disable-background-timer-throttling")
        .arg("--disable-backgrounding-occluded-windows")
        .arg("--disable-breakpad")
        .arg("--disable-client-side-phishing-detection")
        .arg("--disable-component-extensions-with-background-pages")
        .arg("--disable-default-apps")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-features=TranslateUI")
        .arg("--disable-hang-monitor")
        .arg("--disable-ipc-flooding-protection")
        .arg("--disable-popup-blocking")
        .arg("--disable-prompt-on-repost")
        .arg("--disable-renderer-backgrounding")
        .arg("--disable-sync")
        .arg("--force-color-profile=srgb")
        .arg("--metrics-recording-only")
        .arg("--no-first-run")
        .arg("--password-store=basic")
        .arg("--use-mock-keychain")
        .arg("--lang=en_US")
        // Stealth additions:
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--no-default-browser-check")
        .arg("--start-maximized")
        .build()
        .map_err(|e| {
            crate::tel!("chrome_config_error", "error": e.clone());
            format!("Failed to build browser config: {e}")
        })?;

    crate::tel!("chrome_launch_start");
    let (browser, mut handler) = Browser::launch(config).await.map_err(|e| {
        let msg = e.to_string();
        crate::tel!("chrome_launch_error", "error": msg.clone());
        format!("Failed to launch Chrome: {msg}")
    })?;
    crate::tel!("chrome_launched");

    // Spawn the CDP handler — manages websocket communication with Chrome.
    tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            let _ = event;
        }
    });

    // Create a blank page first so we can inject the stealth init-script
    // BEFORE the target URL loads. If we created the page directly at the
    // target URL, the navigator.webdriver = undefined override would only
    // take effect on subsequent navigations, not on the initial load.
    crate::tel!("chrome_new_page_start", "url": url);
    let page = browser.new_page("about:blank").await.map_err(|e| {
        let msg = e.to_string();
        crate::tel!("chrome_new_page_error", "url": url, "error": msg.clone());
        format!("Failed to open page: {msg}")
    })?;

    // Inject stealth overrides before any document loads. Mirrors the
    // puppeteer-extra-stealth surface for the most commonly-fingerprinted
    // properties. Failures are non-fatal — the form may still work without
    // them; they reduce reCAPTCHA's bot score, they don't gate function.
    let stealth = r#"
        Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
        Object.defineProperty(navigator, 'plugins',   { get: () => [1, 2, 3, 4, 5] });
        Object.defineProperty(navigator, 'languages', { get: () => ['en-GB', 'en'] });
        window.chrome = { runtime: {} };
        const originalQuery = window.navigator.permissions && window.navigator.permissions.query;
        if (originalQuery) {
            window.navigator.permissions.query = (p) =>
                p.name === 'notifications'
                    ? Promise.resolve({ state: Notification.permission })
                    : originalQuery(p);
        }
    "#;
    if let Err(e) = page.evaluate_on_new_document(stealth.to_string()).await {
        crate::tel!("chrome_stealth_inject_error", "error": e.to_string());
    } else {
        crate::tel!("chrome_stealth_injected");
    }

    // Now navigate to the real target URL.
    if let Err(e) = page.goto(url).await {
        let msg = e.to_string();
        crate::tel!("chrome_navigate_error", "url": url, "error": msg.clone());
        return Err(format!("Failed to navigate to {url}: {msg}"));
    }
    crate::tel!("chrome_new_page_ready");

    // Initial wait for the page to start rendering.
    tokio::time::sleep(Duration::from_secs(2)).await;

    Ok((browser, page))
}

/// Poll until the selector exists in the DOM or the timeout elapses.
pub async fn wait_for_selector(page: &Page, selector: &str, timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if page.find_element(selector).await.is_ok() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    false
}

/// Type real keystrokes into the focused element identified by selector.
///
/// Uses CDP `Input.dispatchKeyEvent` (via chromiumoxide's `Element::type_str`)
/// which fires through React's synthetic event system natively — no
/// `nativeInputValueSetter` workaround needed.
///
/// Returns true if the element was found and the input was dispatched.
pub async fn type_into(page: &Page, selector: &str, text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    let Ok(element) = page.find_element(selector).await else {
        return false;
    };
    if element.focus().await.is_err() {
        return false;
    }
    element.type_str(text).await.is_ok()
}

/// Upload a file to `<input type="file">` matching selector.
///
/// Uses CDP `DOM.setFileInputFiles` against the element's `backend_node_id`.
/// This is the only path that works for hidden file inputs (Greenhouse's
/// resume input has `class="visually-hidden"` and clicking the visible
/// "Attach Resume" button opens the native file picker).
pub async fn set_file(page: &Page, selector: &str, path: &str) -> bool {
    let Ok(element) = page.find_element(selector).await else {
        return false;
    };
    let Ok(params) = SetFileInputFilesParams::builder()
        .file(path.to_string())
        .backend_node_id(element.backend_node_id)
        .build()
    else {
        return false;
    };
    page.execute(params).await.is_ok()
}

/// Open a Greenhouse combobox and click the option whose visible label
/// matches `option_label` (case-insensitive, trimmed).
///
/// Greenhouse renders selects as React-Select comboboxes:
///   `<input id="X" role="combobox" aria-haspopup="true">` + a separate
///   `<button aria-label="Toggle flyout">`. Options appear as `<li>` or
///   `[role="option"]` children of the listbox after the trigger fires.
pub async fn click_combobox_option(
    page: &Page,
    combobox_selector: &str,
    option_label: &str,
) -> bool {
    // Focus the combobox input — typing into it filters options on
    // many implementations, and focusing alone is enough to make the
    // adjacent trigger button work reliably.
    let Ok(combobox) = page.find_element(combobox_selector).await else {
        return false;
    };
    let _ = combobox.focus().await;
    let _ = combobox.click().await;

    // Give React a tick to render the listbox.
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Find an option in the listbox whose visible text matches the label.
    // We use evaluate here because the listbox is rendered into a portal
    // and the exact selector varies — React-Select uses
    // `[id^="react-select-"][id$="-option-N"]`. We match by text content.
    let js = format!(
        r#"
        (() => {{
            const target = {target_json};
            const norm = s => s.toLowerCase().trim();
            const wanted = norm(target);

            const options = document.querySelectorAll(
                '[role="option"], li[id*="react-select"]'
            );
            for (const opt of options) {{
                if (norm(opt.textContent || '') === wanted) {{
                    opt.click();
                    return true;
                }}
            }}
            // Fallback: contains match.
            for (const opt of options) {{
                if (norm(opt.textContent || '').includes(wanted)) {{
                    opt.click();
                    return true;
                }}
            }}
            return false;
        }})()
        "#,
        target_json = serde_json::to_string(option_label).unwrap_or_default(),
    );

    page.evaluate(js)
        .await
        .ok()
        .and_then(|r| r.into_value::<bool>().ok())
        .unwrap_or(false)
}

/// Click a checkbox in a multi-select fieldset whose label matches `option_label`.
///
/// Greenhouse multi-selects render as:
///   `<fieldset id="question_NNN[]" class="checkbox">
///       <input type="checkbox" id="question_NNN[]_OPTION_ID" value="OPTION_ID"/>
///       <label for="question_NNN[]_OPTION_ID">Option text</label>
///       ...
///    </fieldset>`
///
/// We find the label whose text matches, then click the associated input
/// (using `for=`). Idempotent — already-checked boxes are left alone.
pub async fn set_checkbox_in_fieldset(
    page: &Page,
    fieldset_selector: &str,
    option_label: &str,
    checked: bool,
) -> bool {
    let js = format!(
        r#"
        (() => {{
            const fs = document.querySelector({fs_json});
            if (!fs) return false;
            const target = {target_json}.toLowerCase().trim();
            const labels = fs.querySelectorAll('label');
            for (const lab of labels) {{
                if ((lab.textContent || '').toLowerCase().trim() === target) {{
                    const forId = lab.getAttribute('for');
                    if (!forId) continue;
                    const cb = document.getElementById(forId);
                    if (!cb) continue;
                    const want = {want};
                    if (cb.checked !== want) cb.click();
                    return true;
                }}
            }}
            return false;
        }})()
        "#,
        fs_json = serde_json::to_string(fieldset_selector).unwrap_or_default(),
        target_json = serde_json::to_string(option_label).unwrap_or_default(),
        want = if checked { "true" } else { "false" },
    );
    page.evaluate(js)
        .await
        .ok()
        .and_then(|r| r.into_value::<bool>().ok())
        .unwrap_or(false)
}

// ── Legacy JS-based helpers (kept for non-Greenhouse providers + fallback) ──

/// Try to find an input/textarea by CSS selector and fill it with text.
/// Uses JavaScript to set the value directly, avoiding conflicts with
/// Chrome's autofill and ensuring the full value is inserted cleanly.
///
/// NOTE: this is the React-incompatible path. For Greenhouse, prefer
/// `type_into()` which uses real CDP keystrokes.
pub async fn fill_field(page: &Page, selector: &str, value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    let js = format!(
        r#"
        (() => {{
            const el = document.querySelector({sel});
            if (!el) return false;
            el.focus();
            el.value = {val};
            el.dispatchEvent(new Event('input', {{ bubbles: true }}));
            el.dispatchEvent(new Event('change', {{ bubbles: true }}));
            return true;
        }})()
        "#,
        sel = serde_json::to_string(selector).unwrap_or_default(),
        val = serde_json::to_string(value).unwrap_or_default(),
    );

    page.evaluate(js)
        .await
        .ok()
        .and_then(|r| r.into_value::<bool>().ok())
        .unwrap_or(false)
}

/// Try multiple selectors in order; return true on first hit.
#[allow(dead_code)]
pub async fn fill_by_strategies(
    page: &Page,
    strategies: &[&str],
    value: &str,
) -> bool {
    for selector in strategies {
        if fill_field(page, selector, value).await {
            return true;
        }
    }
    false
}

/// Legacy label-text matching for custom-question textareas.
/// Used by non-Greenhouse ATSes; the Greenhouse path uses the JSON schema.
#[allow(dead_code)]
pub async fn fill_custom_questions(page: &Page, answers: &HashMap<String, String>) -> u32 {
    let mut filled = 0u32;

    for (question, answer) in answers {
        let js = format!(
            r#"
            (() => {{
                const q = {question_json};
                const labels = document.querySelectorAll('label');
                for (const label of labels) {{
                    if (label.textContent.toLowerCase().includes(q.toLowerCase())) {{
                        const forId = label.getAttribute('for');
                        if (forId) {{
                            const el = document.getElementById(forId);
                            if (el && (el.tagName === 'TEXTAREA' || el.tagName === 'INPUT')) {{
                                return forId;
                            }}
                        }}
                        const parent = label.closest('.field') || label.parentElement;
                        if (parent) {{
                            const ta = parent.querySelector('textarea, input[type="text"]');
                            if (ta && ta.id) return ta.id;
                        }}
                    }}
                }}
                return null;
            }})()
            "#,
            question_json = serde_json::to_string(question).unwrap_or_default(),
        );

        if let Ok(result) = page.evaluate(js).await {
            if let Ok(Some(field_id)) = result.into_value::<Option<String>>() {
                let selector = format!("#{field_id}");
                if fill_field(page, &selector, answer).await {
                    filled += 1;
                }
            }
        }
    }

    filled
}
