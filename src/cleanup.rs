use worker::{console_log, Env, Result};
use crate::services::{DatabaseService, StorageService};

pub async fn run_cleanup(env: &Env) -> Result<()> {
    console_log!("Starting cleanup job...");

    let file_ids = DatabaseService::delete_expired_files(env).await?;

    console_log!("Found {} expired files", file_ids.len());

    if file_ids.is_empty() {
        console_log!("No expired files to clean up");
        return Ok(());
    }

    let bucket = env.bucket("TRANSFER_BUCKET")?;
    let mut deleted_count = 0;

    for file_id in &file_ids {
        match StorageService::delete_file(&bucket, file_id).await {
            Ok(_) => {
                console_log!("Deleted from R2: {}", file_id);
                deleted_count += 1;
            }
            Err(e) => {
                console_log!("Failed to delete from R2: {} - {:?}", file_id, e);
            }
        }
    }

    console_log!(
        "Cleanup completed: {} files deleted from storage",
        deleted_count
    );

    Ok(())
}
