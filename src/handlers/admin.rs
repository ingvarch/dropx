use worker::{Request, Response, RouteContext, Result};
use crate::models::{
    CreateApiKeyRequest, ApiKeyResponse, ApiKeyLimits, RevokeApiKeyResponse, ResetRateLimitResponse,
};
use crate::services::DatabaseService;
use crate::utils::auth::{generate_api_key, require_admin_token};
use crate::utils::response::bad_request;
use crate::utils::constants::RATE_LIMITER_NAME;

pub async fn handle_create_key(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let env = ctx.env;

    if let Some(response) = require_admin_token(&req, &env) {
        return response;
    }

    let request_body: CreateApiKeyRequest = match req.json().await {
        Ok(body) => body,
        Err(_) => {
            return bad_request("Invalid request body", "Failed to parse JSON body");
        }
    };

    let key = generate_api_key();
    let now = chrono::Utc::now().timestamp();

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

    Ok(Response::from_json(&response)?.with_status(201))
}

pub async fn handle_revoke_key(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = require_admin_token(&req, &ctx.env) {
        return response;
    }

    let key = match ctx.param("key") {
        Some(k) => k,
        None => {
            return bad_request("Missing API key", "API key parameter is required");
        }
    };

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

    if let Some(response) = require_admin_token(&req, &env) {
        return response;
    }

    let stats = DatabaseService::get_stats(&env).await?;

    Response::from_json(&stats)
}

pub async fn handle_reset_rate_limit(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = require_admin_token(&req, &ctx.env) {
        return response;
    }

    let user_id = match ctx.param("user_id") {
        Some(id) => id,
        None => {
            return bad_request("Missing user_id", "User ID parameter is required");
        }
    };

    let namespace = ctx.env.durable_object(RATE_LIMITER_NAME)?;
    let stub = namespace.id_from_name(user_id)?.get_stub()?;

    let reset_url = "http://rate-limiter/reset";
    let reset_req = worker::Request::new(reset_url, worker::Method::Get)?;
    let response = stub.fetch_with_request(reset_req).await?;

    if response.status_code() != 200 {
        return Response::error("Failed to reset rate limit", response.status_code());
    }

    Response::from_json(&ResetRateLimitResponse {
        message: "Rate limit reset successfully".to_string(),
        user_id: user_id.to_string(),
    })
}
