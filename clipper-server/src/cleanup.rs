use crate::{AppState, CleanupConfig, ClipUpdate};
use chrono::{Duration, Utc};

/// Run the cleanup task periodically based on configuration.
/// This task deletes old clips that have no meaningful tags (only $host: tags or no tags).
pub async fn run_cleanup_task(state: AppState, config: CleanupConfig) {
    if !config.is_active() {
        tracing::debug!("Cleanup task not active, skipping");
        return;
    }

    let interval = config.interval();
    tracing::info!(
        "Starting cleanup task: retention={} days, interval={} hours",
        config.retention_days,
        config.interval_hours
    );

    loop {
        // Wait for the configured interval
        tokio::time::sleep(interval).await;

        // Calculate the cutoff date
        let cutoff = Utc::now() - Duration::days(config.retention_days as i64);

        tracing::info!(
            "Running cleanup: deleting clips older than {} (retention: {} days)",
            cutoff.format("%Y-%m-%d %H:%M:%S UTC"),
            config.retention_days
        );

        // Run cleanup
        match state.indexer.cleanup_entries(None, Some(cutoff)).await {
            Ok(deleted_ids) => {
                if deleted_ids.is_empty() {
                    tracing::info!("Cleanup completed: no clips to delete");
                } else {
                    tracing::info!("Cleanup completed: deleted {} clips", deleted_ids.len());

                    // Notify connected clients about deleted clips
                    for id in deleted_ids {
                        let _ = state.clip_updates.send(ClipUpdate::DeletedClip { id });
                    }
                }
            }
            Err(e) => {
                tracing::error!("Cleanup failed: {}", e);
            }
        }
    }
}
