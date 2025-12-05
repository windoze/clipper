use crate::{AppState, CleanupConfig};
use chrono::{Duration, Utc};

/// Default interval for short URL cleanup (1 hour)
const SHORT_URL_CLEANUP_INTERVAL_SECS: u64 = 3600;

/// Run the clip cleanup task periodically based on configuration.
/// This task deletes old clips that have no meaningful tags (only $host: tags or no tags).
pub async fn run_clip_cleanup_task(state: AppState, config: CleanupConfig) {
    if !config.is_active() {
        tracing::debug!("Clip cleanup task not active, skipping");
        return;
    }

    let interval = config.interval();
    tracing::info!(
        "Starting clip cleanup task: retention={} days, interval={} hours",
        config.retention_days,
        config.interval_hours
    );

    loop {
        // Wait for the configured interval
        tokio::time::sleep(interval).await;

        // Calculate the cutoff date
        let cutoff = Utc::now() - Duration::days(config.retention_days as i64);

        tracing::info!(
            "Running clip cleanup: deleting clips older than {} (retention: {} days)",
            cutoff.format("%Y-%m-%d %H:%M:%S UTC"),
            config.retention_days
        );

        // Run clip cleanup
        match state.indexer.cleanup_entries(None, Some(cutoff)).await {
            Ok(deleted_ids) => {
                if deleted_ids.is_empty() {
                    tracing::info!("Clip cleanup completed: no clips to delete");
                } else {
                    tracing::info!("Clip cleanup completed: deleted {} clips", deleted_ids.len());

                    // Notify connected clients about cleaned up clips
                    state.notify_clips_cleaned_up(deleted_ids);
                }
            }
            Err(e) => {
                tracing::error!("Clip cleanup failed: {}", e);
            }
        }
    }
}

/// Run the short URL cleanup task periodically.
/// This task deletes expired short URLs regardless of the clip cleanup configuration.
/// Runs every hour by default.
pub async fn run_short_url_cleanup_task(state: AppState) {
    let interval = std::time::Duration::from_secs(SHORT_URL_CLEANUP_INTERVAL_SECS);

    tracing::info!("Starting short URL cleanup task: interval=1 hour");

    loop {
        // Wait for the interval
        tokio::time::sleep(interval).await;

        tracing::debug!("Running short URL cleanup");

        // Run expired short URL cleanup
        match state.indexer.cleanup_expired_short_urls().await {
            Ok(count) => {
                if count == 0 {
                    tracing::debug!("Short URL cleanup completed: no expired URLs to delete");
                } else {
                    tracing::info!(
                        "Short URL cleanup completed: deleted {} expired URLs",
                        count
                    );
                }
            }
            Err(e) => {
                tracing::error!("Short URL cleanup failed: {}", e);
            }
        }
    }
}
