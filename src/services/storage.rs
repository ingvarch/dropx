use worker::{Bucket, Data, Env, Object, ObjectBody, Range, ResponseBody, Result};
use crate::utils::constants::TRANSFER_BUCKET_NAME;
use crate::utils::generate_storage_path;

pub struct StorageService;

fn body_of(object: &Object) -> Result<ObjectBody<'_>> {
    object
        .body()
        .ok_or_else(|| worker::Error::RustError("Failed to get object body".to_string()))
}

impl StorageService {
    pub fn get_bucket(env: &Env) -> Result<Bucket> {
        env.bucket(TRANSFER_BUCKET_NAME)
    }

    /// Returns the size of the stored object as R2 reports it.
    pub async fn upload_file(bucket: &Bucket, file_id: &str, data: impl Into<Data>) -> Result<u64> {
        let storage_path = generate_storage_path(file_id);
        let object = bucket.put(&storage_path, data).execute().await?;
        object
            .map(|object| object.size())
            .ok_or_else(|| worker::Error::RustError("R2 did not store the object".to_string()))
    }

    pub async fn read_head(bucket: &Bucket, file_id: &str, length: u64) -> Result<Vec<u8>> {
        let storage_path = generate_storage_path(file_id);
        let object = bucket
            .get(&storage_path)
            .range(Range::Prefix { length })
            .execute()
            .await?
            .ok_or_else(|| worker::Error::RustError("Stored object not found".to_string()))?;
        body_of(&object)?.bytes().await
    }

    pub async fn open_file(bucket: &Bucket, file_id: &str) -> Result<Option<ResponseBody>> {
        let storage_path = generate_storage_path(file_id);
        let Some(object) = bucket.get(&storage_path).execute().await? else {
            return Ok(None);
        };
        body_of(&object)?.response_body().map(Some)
    }

    pub async fn delete_file(bucket: &Bucket, file_id: &str) -> Result<()> {
        let storage_path = generate_storage_path(file_id);
        bucket.delete(&storage_path).await?;
        Ok(())
    }
}
