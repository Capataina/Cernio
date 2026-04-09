use std::collections::HashMap;
use std::time::Duration;

use chromiumoxide::browser::{Browser, BrowserConfig};
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
    let config = BrowserConfig::builder()
        .chrome_executable(CHROME_PATH)
        .with_head()
        .arg("--disable-blink-features=AutomationControlled")
        .arg("--start-maximized")
        .build()
        .map_err(|e| format!("Failed to build browser config: {e}"))?;

    let (browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|e| format!("Failed to launch Chrome: {e}"))?;

    // Spawn the CDP handler — manages websocket communication with Chrome.
    tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            let _ = event;
        }
    });

    let page = browser
        .new_page(url)
        .await
        .map_err(|e| format!("Failed to open page: {e}"))?;

    // Wait for the page to be fully loaded.
    tokio::time::sleep(Duration::from_secs(2)).await;

    Ok((browser, page))
}

/// Try to find an input/textarea by CSS selector and fill it with text.
/// Uses JavaScript to set the value directly, avoiding conflicts with
/// Chrome's autofill and ensuring the full value is inserted cleanly.
/// Returns true if the field was found and filled.
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

/// Try to find an input field by various strategies:
/// 1. By id
/// 2. By name attribute
/// 3. By label text (using aria or for= association)
/// Returns true if found and filled.
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

/// Fill custom question textareas using pre-generated answers.
///
/// Greenhouse renders custom questions as textarea or input elements
/// with associated labels. We match answer keys against label text
/// (case-insensitive, substring match) and fill matching fields.
///
/// Returns the number of fields successfully filled.
pub async fn fill_custom_questions(page: &Page, answers: &HashMap<String, String>) -> u32 {
    let mut filled = 0u32;

    // Use JavaScript to find all textareas and large inputs with labels,
    // then match them against our answer keys.
    for (question, answer) in answers {
        // Try to find a textarea or input whose label contains the question text.
        // We use JavaScript here because matching by label text requires DOM traversal.
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
                        // Try sibling/child textarea.
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

/// Upload a file to an input[type="file"] element.
#[allow(dead_code)]
pub async fn upload_file(page: &Page, selector: &str, path: &str) -> bool {
    let Ok(element) = page.find_element(selector).await else {
        return false;
    };

    // Use CDP to set file input — chromiumoxide exposes this through
    // the element's underlying node.
    let _ = element
        .click()
        .await;

    // File upload via CDP requires DOM.setFileInputFiles.
    // For now, we'll skip this — the user can drag-drop their CV.
    // TODO: Implement CDP file upload when needed.
    let _ = path;
    false
}
