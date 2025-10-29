use worker::*;

mod handlers;
mod models;
mod services;
mod utils;
mod durable_objects;
mod cleanup;

pub use durable_objects::RateLimiter;


#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/", handlers::handle_landing)
        .post_async("/", handlers::handle_upload)
        .put_async("/:filename", handlers::handle_upload)
        .get_async("/:id/info", handlers::handle_info)
        .get_async("/:id", handlers::handle_download)
        .post_async("/admin/api-keys", handlers::handle_create_key)
        .delete_async("/admin/api-keys/:key", handlers::handle_revoke_key)
        .get_async("/admin/stats", handlers::handle_get_stats)
        .post_async("/admin/rate-limit/reset/:user_id", handlers::handle_reset_rate_limit)
        .run(req, env)
        .await
}

#[event(scheduled)]
async fn scheduled(_event: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    if let Err(e) = cleanup::run_cleanup(&env).await {
        console_log!("Cleanup job failed: {:?}", e);
    }
}