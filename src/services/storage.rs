use worker::{Bucket, Env, Result, ResponseBody};
use crate::utils::constants::TRANSFER_BUCKET_NAME;
use crate::utils::generate_storage_path;

pub struct StorageService;

impl StorageService {
    pub fn get_bucket(env: &Env) -> Result<Bucket> {
        env.bucket(TRANSFER_BUCKET_NAME)
    }

    pub async fn upload_file(bucket: &Bucket, file_id: &str, data: Vec<u8>) -> Result<()> {
        let storage_path = generate_storage_path(file_id);
        bucket.put(&storage_path, data).execute().await?;
        Ok(())
    }

    pub async fn open_file(bucket: &Bucket, file_id: &str) -> Result<Option<ResponseBody>> {
        let storage_path = generate_storage_path(file_id);
        let Some(object) = bucket.get(&storage_path).execute().await? else {
            return Ok(None);
        };
        let body = object
            .body()
            .ok_or_else(|| worker::Error::RustError("Failed to get object body".to_string()))?;
        body.response_body().map(Some)
    }

    pub async fn delete_file(bucket: &Bucket, file_id: &str) -> Result<()> {
        let storage_path = generate_storage_path(file_id);
        bucket.delete(&storage_path).await?;
        Ok(())
    }
}
