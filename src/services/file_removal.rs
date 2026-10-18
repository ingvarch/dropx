use std::future::Future;
use worker::{Bucket, Env, Result};
use crate::services::{DatabaseService, StorageService};

/// Deletes the R2 object, then the D1 row. A failed object delete keeps the row so a later run retries it.
pub async fn remove_file(env: &Env, bucket: &Bucket, file_id: &str) -> Result<()> {
    remove_object_then_row(
        StorageService::delete_file(bucket, file_id),
        DatabaseService::delete_file(env, file_id),
    )
    .await
}

async fn remove_object_then_row<E>(
    delete_object: impl Future<Output = std::result::Result<(), E>>,
    delete_row: impl Future<Output = std::result::Result<(), E>>,
) -> std::result::Result<(), E> {
    delete_object.await?;
    delete_row.await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
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

    #[test]
    fn row_is_deleted_after_the_object_is_deleted() {
        let object_deleted = Cell::new(false);
        let row_deleted_after_object = Cell::new(false);

        let result = block_on(remove_object_then_row(
            async {
                object_deleted.set(true);
                Ok::<(), &str>(())
            },
            async {
                row_deleted_after_object.set(object_deleted.get());
                Ok(())
            },
        ));

        assert_eq!(result, Ok(()));
        assert!(row_deleted_after_object.get());
    }

    #[test]
    fn row_is_kept_when_the_object_delete_fails() {
        let row_deleted = Cell::new(false);

        let result = block_on(remove_object_then_row(
            async { Err("r2 unavailable") },
            async {
                row_deleted.set(true);
                Ok(())
            },
        ));

        assert_eq!(result, Err("r2 unavailable"));
        assert!(!row_deleted.get());
    }
}
