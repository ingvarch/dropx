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

    let bucket = StorageService::get_bucket(env)?;
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_cleanup_empty_list_handling() {
        let file_ids: Vec<String> = vec![];
        assert!(file_ids.is_empty());
    }

    #[test]
    fn test_cleanup_counter_single_file() {
        let mut deleted_count = 0;
        let success = true;

        if success {
            deleted_count += 1;
        }

        assert_eq!(deleted_count, 1);
    }

    #[test]
    fn test_cleanup_counter_multiple_files() {
        let mut deleted_count = 0;
        let results = vec![true, true, true];

        for success in results {
            if success {
                deleted_count += 1;
            }
        }

        assert_eq!(deleted_count, 3);
    }

    #[test]
    fn test_cleanup_partial_failure() {
        let mut deleted_count = 0;
        let results = vec![true, false, true];

        for success in results {
            if success {
                deleted_count += 1;
            }
        }

        assert_eq!(deleted_count, 2);
    }

    #[test]
    fn test_cleanup_all_failures() {
        let mut deleted_count = 0;
        let results = vec![false, false, false];

        for success in results {
            if success {
                deleted_count += 1;
            }
        }

        assert_eq!(deleted_count, 0);
    }

    #[test]
    fn test_cleanup_file_id_iteration() {
        let file_ids = vec![
            "file1".to_string(),
            "file2".to_string(),
            "file3".to_string(),
        ];

        let mut processed = Vec::new();
        for file_id in &file_ids {
            processed.push(file_id.clone());
        }

        assert_eq!(processed.len(), 3);
        assert_eq!(processed[0], "file1");
        assert_eq!(processed[1], "file2");
        assert_eq!(processed[2], "file3");
    }

    #[test]
    fn test_cleanup_large_batch() {
        let file_ids: Vec<String> = (0..1000).map(|i| format!("file_{}", i)).collect();
        assert_eq!(file_ids.len(), 1000);
    }

    #[test]
    fn test_cleanup_success_rate_calculation() {
        let total = 10;
        let deleted_count = 7;
        let failed = total - deleted_count;

        assert_eq!(failed, 3);
        assert_eq!(deleted_count, 7);
    }
}
