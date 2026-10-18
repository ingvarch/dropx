use worker::{console_log, Env, Result};
use crate::services::{remove_file, DatabaseService, StorageService};

pub async fn run_cleanup(env: &Env) -> Result<()> {
    console_log!("Starting cleanup job...");

    let file_ids = DatabaseService::get_expired_file_ids(env).await?;

    console_log!("Found {} expired files", file_ids.len());

    if file_ids.is_empty() {
        console_log!("No expired files to clean up");
        return Ok(());
    }

    let bucket = StorageService::get_bucket(env)?;
    let mut deleted = 0;
    let mut failed = 0;

    for file_id in &file_ids {
        match remove_file(env, &bucket, file_id).await {
            Ok(()) => deleted += 1,
            Err(e) => {
                failed += 1;
                console_log!("Failed to remove file: {} - {:?}", file_id, e);
            }
        }
    }

    console_log!(
        "Cleanup completed: {} files deleted, {} failed and will be retried",
        deleted,
        failed
    );

    Ok(())
}
