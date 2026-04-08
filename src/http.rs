use std::time::Duration;

/// Build a properly configured HTTP client for ATS API calls.
///
/// - 15-second timeout per request (ATS APIs should respond in <5s)
/// - User-Agent to avoid being blocked by WAFs
/// - Connection pooling via reqwest defaults
pub fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("Cernio/0.1 (job-search-tool)")
        .build()
        .expect("failed to build HTTP client")
}

/// Retry a future up to `max_retries` times with a short delay between attempts.
/// Returns the first successful result, or the last error.
pub async fn with_retry<F, Fut, T, E>(max_retries: u32, mut f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut last_err = None;

    for attempt in 0..=max_retries {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                last_err = Some(e);
                if attempt < max_retries {
                    tokio::time::sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                }
            }
        }
    }

    Err(last_err.unwrap())
}
