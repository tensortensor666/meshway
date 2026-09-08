use crate::database::Database;

pub async fn record(
    database: &Database,
    api_key_id: &str,
    provider: &str,
    model: Option<&str>,
    path: &str,
    status: u16,
    started: std::time::Instant,
) {
    if let Err(error) = database
        .add_log(
            Some(api_key_id),
            provider,
            model,
            path,
            status,
            started.elapsed().as_millis() as i64,
        )
        .await
    {
        tracing::warn!(%error, "failed to write request log");
    }
}
