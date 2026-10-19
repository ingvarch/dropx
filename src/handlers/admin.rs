use worker::{Request, Response, RouteContext, Result};
use crate::models::{
    CreateApiKeyRequest, ApiKeyResponse, Limits, ApiKeyLimits, RevokeApiKeyResponse, ResetRateLimitResponse,
};
use crate::services::{DatabaseService, RateLimiterService};
use crate::utils::auth::{generate_api_key, hash_secret};
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
        &hash_secret(&key),
        Some(&request_body.email),
        request_body.tier,
        now,
    )
    .await?;

    let limits = Limits::for_tier(Some(request_body.tier));
    let response = ApiKeyResponse {
        key,
        tier: request_body.tier,
        created_at: now,
        limits: ApiKeyLimits {
            hourly: limits.hourly,
            daily: limits.daily,
        },
    };

    Ok(Response::from_json(&response)?.with_status(HTTP_CREATED))
}

pub async fn handle_revoke_key(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    require_admin!(&req, &ctx.env);

    let key = get_required_param(&ctx, "key", "API key parameter is required")?;

    let success = DatabaseService::revoke_api_key(&ctx.env, &hash_secret(key)).await?;

    if success {
        Response::from_json(&RevokeApiKeyResponse {
            message: "API key revoked".to_string(),
            key: key.to_string(),
        })
    } else {
        use crate::utils::response::not_found;
        not_found("NotFound", "The specified API key does not exist")
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
