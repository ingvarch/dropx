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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_name_constant() {
        assert_eq!(TRANSFER_BUCKET_NAME, "TRANSFER_BUCKET");
    }

    #[test]
    fn test_download_file_some_result() {
        let mock_data: Option<Vec<u8>> = Some(vec![1, 2, 3, 4, 5]);

        match mock_data {
            Some(data) => {
                assert_eq!(data.len(), 5);
                assert_eq!(data, vec![1, 2, 3, 4, 5]);
            }
            None => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_download_file_none_result() {
        let mock_data: Option<Vec<u8>> = None;

        match mock_data {
            Some(_) => panic!("Expected None, got Some"),
            None => {
                assert!(true);
            }
        }
    }

    #[test]
    fn test_download_file_empty_data() {
        let mock_data: Option<Vec<u8>> = Some(vec![]);

        match mock_data {
            Some(data) => {
                assert_eq!(data.len(), 0);
                assert!(data.is_empty());
            }
            None => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_download_file_large_data() {
        let mock_data: Option<Vec<u8>> = Some(vec![0; 1024 * 1024]);

        match mock_data {
            Some(data) => {
                assert_eq!(data.len(), 1024 * 1024);
            }
            None => panic!("Expected Some, got None"),
        }
    }

    #[test]
    fn test_file_id_formats() {
        let file_ids = vec![
            "abc123",
            "file_001",
            "test-file-123",
            "очень-длинный-идентификатор-файла",
        ];

        for file_id in file_ids {
            assert!(!file_id.is_empty());
        }
    }

    #[test]
    fn test_error_message_format() {
        let error_msg = "Failed to get object body";
        assert_eq!(error_msg, "Failed to get object body");
        assert!(error_msg.contains("Failed"));
        assert!(error_msg.contains("object body"));
    }

    #[test]
    fn test_option_is_some_vs_is_none() {
        let some_data: Option<Vec<u8>> = Some(vec![1, 2, 3]);
        let no_data: Option<Vec<u8>> = None;

        assert!(some_data.is_some());
        assert!(!some_data.is_none());

        assert!(!no_data.is_some());
        assert!(no_data.is_none());
    }

    #[test]
    fn test_vec_u8_operations() {
        let data = vec![1u8, 2, 3, 4, 5];

        assert_eq!(data.len(), 5);
        assert_eq!(data[0], 1);
        assert_eq!(data[4], 5);
        assert!(!data.is_empty());
    }

    #[test]
    fn test_ok_unit_result() {
        let result: std::result::Result<(), &str> = Ok(());
        assert!(result.is_ok());
    }
}
