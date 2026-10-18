use worker::{Bucket, Env, Result};
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

    pub async fn download_file(bucket: &Bucket, file_id: &str) -> Result<Option<Vec<u8>>> {
        let storage_path = generate_storage_path(file_id);
        let object = bucket.get(&storage_path).execute().await?;

        match object {
            Some(obj) => {
                let body = obj.body().ok_or_else(|| {
                    worker::Error::RustError("Failed to get object body".to_string())
                })?;
                let bytes = body.bytes().await?;
                Ok(Some(bytes))
            }
            None => Ok(None),
        }
    }

    pub async fn delete_file(bucket: &Bucket, file_id: &str) -> Result<()> {
        let storage_path = generate_storage_path(file_id);
        bucket.delete(&storage_path).await?;
        Ok(())
    }
}
