use worker::{Result, D1Database};
use crate::models::{FileMetadata, ApiKey, ApiKeyTier};
use crate::utils::constants::{DB_NAME, DB_BOOL_TRUE, DB_BOOL_FALSE};
use crate::utils::current_timestamp;
use serde::Deserialize;

pub struct DatabaseService;

#[derive(Deserialize)]
struct FileRow {
    id: String,
    original_filename: String,
    size: i64,
    mime_type: String,
    created_at: i64,
    expires_at: i64,
    download_count: i64,
    uploaded_by: String,
}

impl From<FileRow> for FileMetadata {
    fn from(row: FileRow) -> Self {
        Self {
            id: row.id,
            original_filename: row.original_filename,
            size: row.size,
            mime_type: row.mime_type,
            created_at: row.created_at,
            expires_at: row.expires_at,
            download_count: row.download_count as i32,
            uploaded_by: row.uploaded_by,
        }
    }
}

#[derive(Deserialize)]
struct ApiKeyRow {
    key: String,
    user_email: Option<String>,
    created_at: i64,
    last_used_at: Option<i64>,
    is_active: i64,
    tier: String,
}

#[derive(Deserialize)]
struct CountResult {
    count: i64,
}

#[derive(Deserialize)]
struct SumResult {
    total: Option<i64>,
}

#[derive(Deserialize)]
struct FileIdResult {
    id: String,
}

impl DatabaseService {
    fn get_db(env: &worker::Env) -> Result<D1Database> {
        env.d1(DB_NAME)
    }

    pub async fn insert_file(env: &worker::Env, file: &FileMetadata) -> Result<()> {
        let d1 = Self::get_db(env)?;
        let statement = d1.prepare(
            "INSERT INTO files (id, original_filename, size, mime_type, created_at, expires_at, download_count, uploaded_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
        );

        statement.bind(&[
            file.id.clone().into(),
            file.original_filename.clone().into(),
            (file.size as f64).into(),
            file.mime_type.clone().into(),
            (file.created_at as f64).into(),
            (file.expires_at as f64).into(),
            (file.download_count as f64).into(),
            file.uploaded_by.clone().into(),
        ])?.run().await?;

        Ok(())
    }

    pub async fn get_file(env: &worker::Env, file_id: &str) -> Result<Option<FileMetadata>> {
        let d1 = Self::get_db(env)?;
        let statement = d1.prepare("SELECT * FROM files WHERE id = ?1");
        let query = statement.bind(&[file_id.into()])?;

        match query.first::<FileRow>(None).await? {
            Some(row) => Ok(Some(row.into())),
            None => Ok(None),
        }
    }

    pub async fn increment_download_count(env: &worker::Env, file_id: &str) -> Result<()> {
        let d1 = Self::get_db(env)?;
        let statement = d1.prepare("UPDATE files SET download_count = download_count + 1 WHERE id = ?1");
        statement.bind(&[file_id.into()])?.run().await?;
        Ok(())
    }

    pub async fn delete_expired_files(env: &worker::Env) -> Result<Vec<String>> {
        let d1 = Self::get_db(env)?;
        let now = current_timestamp();

        let select_stmt = d1.prepare("SELECT id FROM files WHERE expires_at < ?1");
        let query = select_stmt.bind(&[(now as f64).into()])?;
        let results = query.all().await?;
        let file_ids: Vec<String> = results
            .results::<FileIdResult>()?
            .iter()
            .map(|r| r.id.clone())
            .collect();

        if !file_ids.is_empty() {
            let delete_stmt = d1.prepare("DELETE FROM files WHERE expires_at < ?1");
            delete_stmt.bind(&[(now as f64).into()])?.run().await?;
        }

        Ok(file_ids)
    }

    pub async fn insert_api_key(
        env: &worker::Env,
        key: &str,
        email: Option<&str>,
        tier: ApiKeyTier,
        created_at: i64,
    ) -> Result<()> {
        let d1 = Self::get_db(env)?;
        let statement = d1.prepare(
            "INSERT INTO api_keys (key, user_email, created_at, is_active, tier)
             VALUES (?1, ?2, ?3, ?4, ?5)"
        );

        statement.bind(&[
            key.into(),
            email.map(|e| e.into()).unwrap_or_else(|| worker::wasm_bindgen::JsValue::NULL),
            (created_at as f64).into(),
            (DB_BOOL_TRUE as f64).into(),
            tier.as_str().into(),
        ])?.run().await?;

        Ok(())
    }

    pub async fn get_api_key(env: &worker::Env, key: &str) -> Result<Option<ApiKey>> {
        let d1 = Self::get_db(env)?;
        let statement = d1.prepare("SELECT * FROM api_keys WHERE key = ?1");
        let query = statement.bind(&[key.into()])?;

        match query.first::<ApiKeyRow>(None).await? {
            Some(row) => {
                Ok(Some(ApiKey {
                    key: row.key,
                    user_email: row.user_email,
                    created_at: row.created_at,
                    last_used_at: row.last_used_at,
                    is_active: row.is_active == DB_BOOL_TRUE,
                    tier: ApiKeyTier::from_str(&row.tier),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn update_api_key_last_used(env: &worker::Env, key: &str, timestamp: i64) -> Result<()> {
        let d1 = Self::get_db(env)?;
        let statement = d1.prepare("UPDATE api_keys SET last_used_at = ?1 WHERE key = ?2");
        statement.bind(&[(timestamp as f64).into(), key.into()])?.run().await?;
        Ok(())
    }

    pub async fn revoke_api_key(env: &worker::Env, key: &str) -> Result<bool> {
        let d1 = Self::get_db(env)?;
        let statement = d1.prepare("UPDATE api_keys SET is_active = ?1 WHERE key = ?2");
        statement.bind(&[(DB_BOOL_FALSE as f64).into(), key.into()])?.run().await?;
        Ok(true)
    }

    pub async fn get_stats(env: &worker::Env) -> Result<StatsResponse> {
        let d1 = Self::get_db(env)?;
        let now = current_timestamp();

        let total_files = d1.prepare("SELECT COUNT(*) as count FROM files")
            .first::<CountResult>(None)
            .await?
            .map(|r| r.count)
            .unwrap_or(0);

        let total_size = d1.prepare("SELECT SUM(size) as total FROM files")
            .first::<SumResult>(None)
            .await?
            .and_then(|r| r.total)
            .unwrap_or(0);

        let active_files = d1.prepare("SELECT COUNT(*) as count FROM files WHERE expires_at > ?1")
            .bind(&[(now as f64).into()])?
            .first::<CountResult>(None)
            .await?
            .map(|r| r.count)
            .unwrap_or(0);

        let total_downloads = d1.prepare("SELECT SUM(download_count) as total FROM files")
            .first::<SumResult>(None)
            .await?
            .and_then(|r| r.total)
            .unwrap_or(0);

        let api_keys_count = d1.prepare("SELECT COUNT(*) as count FROM api_keys WHERE is_active = ?1")
            .bind(&[(DB_BOOL_TRUE as f64).into()])?
            .first::<CountResult>(None)
            .await?
            .map(|r| r.count)
            .unwrap_or(0);

        Ok(StatsResponse {
            total_files,
            total_size_bytes: total_size,
            active_files,
            total_downloads,
            api_keys_count,
        })
    }
}

#[derive(serde::Serialize)]
pub struct StatsResponse {
    pub total_files: i64,
    pub total_size_bytes: i64,
    pub active_files: i64,
    pub total_downloads: i64,
    pub api_keys_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_row_to_metadata_conversion() {
        let row = FileRow {
            id: "test123".to_string(),
            original_filename: "document.pdf".to_string(),
            size: 1024,
            mime_type: "application/pdf".to_string(),
            created_at: 1000000,
            expires_at: 2000000,
            download_count: 5,
            uploaded_by: "user123".to_string(),
        };

        let metadata: FileMetadata = row.into();

        assert_eq!(metadata.id, "test123");
        assert_eq!(metadata.original_filename, "document.pdf");
        assert_eq!(metadata.size, 1024);
        assert_eq!(metadata.mime_type, "application/pdf");
        assert_eq!(metadata.created_at, 1000000);
        assert_eq!(metadata.expires_at, 2000000);
        assert_eq!(metadata.download_count, 5);
        assert_eq!(metadata.uploaded_by, "user123");
    }

    #[test]
    fn test_file_row_download_count_conversion() {
        let row = FileRow {
            id: "test".to_string(),
            original_filename: "file".to_string(),
            size: 100,
            mime_type: "text/plain".to_string(),
            created_at: 1000,
            expires_at: 2000,
            download_count: 999,
            uploaded_by: "user".to_string(),
        };

        let metadata: FileMetadata = row.into();
        assert_eq!(metadata.download_count, 999);
    }

    #[test]
    fn test_is_active_flag_true() {
        let is_active_db = DB_BOOL_TRUE;
        let is_active = is_active_db == DB_BOOL_TRUE;
        assert!(is_active);
    }

    #[test]
    fn test_is_active_flag_false() {
        let is_active_db = DB_BOOL_FALSE;
        let is_active = is_active_db == DB_BOOL_TRUE;
        assert!(!is_active);
    }

    #[test]
    fn test_api_key_tier_from_str_standard() {
        let tier_str = "standard";
        let tier = ApiKeyTier::from_str(tier_str);
        assert_eq!(tier, ApiKeyTier::Standard);
    }

    #[test]
    fn test_api_key_tier_from_str_premium() {
        let tier_str = "premium";
        let tier = ApiKeyTier::from_str(tier_str);
        assert_eq!(tier, ApiKeyTier::Premium);
    }

    #[test]
    fn test_api_key_tier_from_str_unknown() {
        let tier_str = "unknown";
        let tier = ApiKeyTier::from_str(tier_str);
        assert_eq!(tier, ApiKeyTier::Standard);
    }

    #[test]
    fn test_api_key_tier_to_str_standard() {
        let tier = ApiKeyTier::Standard;
        assert_eq!(tier.as_str(), "standard");
    }

    #[test]
    fn test_api_key_tier_to_str_premium() {
        let tier = ApiKeyTier::Premium;
        assert_eq!(tier.as_str(), "premium");
    }

    #[test]
    fn test_stats_response_serialization() {
        let stats = StatsResponse {
            total_files: 100,
            total_size_bytes: 1048576,
            active_files: 80,
            total_downloads: 500,
            api_keys_count: 10,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"total_files\":100"));
        assert!(json.contains("\"total_size_bytes\":1048576"));
        assert!(json.contains("\"active_files\":80"));
        assert!(json.contains("\"total_downloads\":500"));
        assert!(json.contains("\"api_keys_count\":10"));
    }

    #[test]
    fn test_db_bool_constants() {
        assert_eq!(DB_BOOL_TRUE, 1);
        assert_eq!(DB_BOOL_FALSE, 0);
        assert_ne!(DB_BOOL_TRUE, DB_BOOL_FALSE);
    }

    #[test]
    fn test_stats_response_default_values() {
        let stats = StatsResponse {
            total_files: 0,
            total_size_bytes: 0,
            active_files: 0,
            total_downloads: 0,
            api_keys_count: 0,
        };

        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_size_bytes, 0);
    }
}
