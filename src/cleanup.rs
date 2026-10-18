use std::future::Future;
use worker::{console_log, Env, Result};
use crate::services::{DatabaseService, StorageService};

pub async fn run_cleanup(env: &Env) -> Result<()> {
    console_log!("Starting cleanup job...");

    let file_ids = DatabaseService::get_expired_file_ids(env).await?;

    console_log!("Found {} expired files", file_ids.len());

    if file_ids.is_empty() {
        console_log!("No expired files to clean up");
        return Ok(());
    }

    let bucket = StorageService::get_bucket(env)?;
    let outcome = delete_objects(&file_ids, |file_id| {
        let bucket = &bucket;
        async move { StorageService::delete_file(bucket, &file_id).await }
    })
    .await;

    for failure in &outcome.failed {
        console_log!("Failed to delete from R2: {} - {}", failure.file_id, failure.error);
    }

    DatabaseService::delete_files(env, &outcome.deleted).await?;

    console_log!(
        "Cleanup completed: {} files deleted, {} failed and will be retried",
        outcome.deleted.len(),
        outcome.failed.len()
    );

    Ok(())
}

#[derive(Debug, Default, PartialEq)]
struct DeleteOutcome {
    deleted: Vec<String>,
    failed: Vec<FailedDelete>,
}

#[derive(Debug, PartialEq)]
struct FailedDelete {
    file_id: String,
    error: String,
}

async fn delete_objects<F, Fut, E>(file_ids: &[String], mut delete: F) -> DeleteOutcome
where
    F: FnMut(String) -> Fut,
    Fut: Future<Output = std::result::Result<(), E>>,
    E: std::fmt::Debug,
{
    let mut outcome = DeleteOutcome::default();

    for file_id in file_ids {
        match delete(file_id.clone()).await {
            Ok(()) => outcome.deleted.push(file_id.clone()),
            Err(e) => outcome.failed.push(FailedDelete {
                file_id: file_id.clone(),
                error: format!("{e:?}"),
            }),
        }
    }

    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        let mut future = pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => value,
            Poll::Pending => panic!("future did not complete at once"),
        }
    }

    fn ids(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn delete_objects_reports_every_id_deleted_when_all_succeed() {
        let outcome = block_on(delete_objects(&ids(&["a", "b"]), |_| async { Ok::<(), ()>(()) }));

        assert_eq!(outcome.deleted, ids(&["a", "b"]));
        assert!(outcome.failed.is_empty());
    }

    #[test]
    fn delete_objects_keeps_a_failed_id_out_of_the_deleted_list() {
        let outcome = block_on(delete_objects(&ids(&["a", "b", "c"]), |id| async move {
            if id == "b" { Err("r2 unavailable") } else { Ok(()) }
        }));

        assert_eq!(outcome.deleted, ids(&["a", "c"]));
        assert_eq!(
            outcome.failed,
            vec![FailedDelete { file_id: "b".to_string(), error: "\"r2 unavailable\"".to_string() }]
        );
    }

    #[test]
    fn delete_objects_with_no_ids_deletes_nothing() {
        let outcome = block_on(delete_objects(&[], |_| async { Ok::<(), ()>(()) }));

        assert_eq!(outcome, DeleteOutcome::default());
    }
}
