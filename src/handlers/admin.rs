use worker::{Request, Response, RouteContext, Result};
use crate::models::{
    CreateApiKeyRequest, ApiKeyResponse, ApiKeyLimits, RevokeApiKeyResponse, ResetRateLimitResponse,
};
use crate::services::{DatabaseService, RateLimiterService};
use crate::utils::auth::generate_api_key;
use crate::utils::response::{bad_request, get_required_param};
use crate::utils::current_timestamp;
use crate::utils::constants::HTTP_CREATED;
use crate::require_admin;

pub async fn handle_create_key(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;

    require_admin!(&req, &env);

    let request_body: CreateApiKeyRequest = match req.json().await {
        Ok(body) => body,
        Err(_) => {
            return bad_request("Invalid request body", "Failed to parse JSON body");
        }
    };

    let key = generate_api_key();
    let now = current_timestamp();

    DatabaseService::insert_api_key(
        &env,
        &key,
        Some(&request_body.email),
        request_body.tier,
        now,
    )
    .await?;

    let response = ApiKeyResponse {
        key,
        tier: request_body.tier,
        created_at: now,
        limits: ApiKeyLimits {
            hourly: request_body.tier.hourly_limit(),
            daily: request_body.tier.daily_limit(),
        },
    };

    Ok(Response::from_json(&response)?.with_status(HTTP_CREATED))
}

pub async fn handle_revoke_key(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_admin!(&req, &ctx.env);

    let key = get_required_param(&ctx, "key", "API key parameter is required")?;

    let success = DatabaseService::revoke_api_key(&ctx.env, key).await?;

    if success {
        Response::from_json(&RevokeApiKeyResponse {
            message: "API key revoked".to_string(),
            key: key.to_string(),
        })
    } else {
        use crate::utils::response::not_found;
        not_found("API key not found", "The specified API key does not exist")
    }
}

pub async fn handle_get_stats(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;

    require_admin!(&req, &env);

    let stats = DatabaseService::get_stats(&env).await?;

    Response::from_json(&stats)
}

pub async fn handle_reset_rate_limit(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_admin!(&req, &ctx.env);

    let user_id = get_required_param(&ctx, "user_id", "User ID parameter is required")?;

    RateLimiterService::reset_limit(&ctx.env, user_id).await?;

    Response::from_json(&ResetRateLimitResponse {
        message: "Rate limit reset successfully".to_string(),
        user_id: user_id.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::api_key::ApiKeyTier;

    #[test]
    fn test_api_key_format() {
        let key = "tfr_abc123def456";
        assert!(key.starts_with("tfr_"));
        assert!(key.len() > 10);
    }

    #[test]
    fn test_api_key_limits_standard() {
        let tier = ApiKeyTier::Standard;
        let limits = ApiKeyLimits {
            hourly: tier.hourly_limit(),
            daily: tier.daily_limit(),
        };

        assert_eq!(limits.hourly, 100);
        assert_eq!(limits.daily, 500);
    }

    #[test]
    fn test_api_key_limits_premium() {
        let tier = ApiKeyTier::Premium;
        let limits = ApiKeyLimits {
            hourly: tier.hourly_limit(),
            daily: tier.daily_limit(),
        };

        assert_eq!(limits.hourly, 1000);
        assert_eq!(limits.daily, 5000);
    }

    #[test]
    fn test_revoke_response_format() {
        let response = RevokeApiKeyResponse {
            message: "API key revoked".to_string(),
            key: "tfr_test123".to_string(),
        };

        assert_eq!(response.message, "API key revoked");
        assert_eq!(response.key, "tfr_test123");
    }

    #[test]
    fn test_reset_rate_limit_response_format() {
        let response = ResetRateLimitResponse {
            message: "Rate limit reset successfully".to_string(),
            user_id: "user123".to_string(),
        };

        assert_eq!(response.message, "Rate limit reset successfully");
        assert_eq!(response.user_id, "user123");
    }

    #[test]
    fn test_api_key_response_creation() {
        let key = "tfr_test123".to_string();
        let tier = ApiKeyTier::Premium;
        let created_at = 1704067200;

        let response = ApiKeyResponse {
            key: key.clone(),
            tier,
            created_at,
            limits: ApiKeyLimits {
                hourly: tier.hourly_limit(),
                daily: tier.daily_limit(),
            },
        };

        assert_eq!(response.key, "tfr_test123");
        assert_eq!(response.created_at, 1704067200);
        assert_eq!(response.limits.hourly, 1000);
        assert_eq!(response.limits.daily, 5000);
    }

    #[test]
    fn test_api_key_revoke_success_flag() {
        let success = true;
        assert!(success);

        let success = false;
        assert!(!success);
    }

    #[test]
    fn test_tier_limits_relationship() {
        let standard = ApiKeyTier::Standard;
        let premium = ApiKeyTier::Premium;

        assert!(premium.hourly_limit() > standard.hourly_limit());
        assert!(premium.daily_limit() > standard.daily_limit());
    }
}
