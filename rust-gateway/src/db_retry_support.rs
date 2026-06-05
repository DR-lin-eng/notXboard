use std::future::Future;

use crate::*;

pub(crate) async fn retry_db_write<F, Fut, T>(
    label: &str,
    mut operation: F,
) -> Result<T, sqlx::Error>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, sqlx::Error>>,
{
    let max_attempts = std::env::var("DB_WRITE_RETRY_ATTEMPTS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .map(|value| value.clamp(1, 8))
        .unwrap_or(3);
    let base_backoff_ms = std::env::var("DB_WRITE_RETRY_BACKOFF_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.clamp(1, 250))
        .unwrap_or(25);

    let mut attempt = 1u32;
    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) if attempt < max_attempts && is_retryable_mysql_concurrency_error(&err) => {
                warn!(
                    label = label,
                    attempt,
                    max_attempts,
                    "retrying mysql concurrency error"
                );
                tokio::time::sleep(Duration::from_millis(base_backoff_ms * attempt as u64)).await;
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}

fn is_retryable_mysql_concurrency_error(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => matches!(
            db_err.code().as_deref(),
            Some("1213") | Some("1205")
        ),
        _ => false,
    }
}
