use worker::{Bucket, Env, Result};
use crate::utils::constants::TRANSFER_BUCKET_NAME;

pub struct StorageService;

impl StorageService {
    pub fn get_bucket(env: &Env) -> Result<Bucket> {
        env.bucket(TRANSFER_BUCKET_NAME)
    }

    pub async fn upload_file(bucket: &Bucket, file_id: &str, data: Vec<u8>) -> Result<()> {
        bucket.put(file_id, data).execute().await?;
        Ok(())
    }

    pub async fn download_file(bucket: &Bucket, file_id: &str) -> Result<Option<Vec<u8>>> {
        let object = bucket.get(file_id).execute().await?;

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
        bucket.delete(file_id).await?;
        Ok(())
    }
}
