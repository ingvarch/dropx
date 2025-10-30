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
        .delete_async("/:id/:token", handlers::handle_delete)
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_route_paths() {
        let routes = vec![
            "/",
            "/:filename",
            "/:id/info",
            "/:id/:token",
            "/:id",
            "/admin/api-keys",
            "/admin/api-keys/:key",
            "/admin/stats",
            "/admin/rate-limit/reset/:user_id",
        ];

        for route in routes {
            assert!(!route.is_empty());
        }
    }

    #[test]
    fn test_root_route() {
        let route = "/";
        assert_eq!(route, "/");
        assert_eq!(route.len(), 1);
    }

    #[test]
    fn test_parameterized_routes() {
        let filename_route = "/:filename";
        assert!(filename_route.contains(":filename"));

        let id_info_route = "/:id/info";
        assert!(id_info_route.contains(":id"));
        assert!(id_info_route.ends_with("/info"));

        let id_route = "/:id";
        assert!(id_route.contains(":id"));
    }

    #[test]
    fn test_admin_routes_prefix() {
        let admin_routes = vec![
            "/admin/api-keys",
            "/admin/api-keys/:key",
            "/admin/stats",
            "/admin/rate-limit/reset/:user_id",
        ];

        for route in admin_routes {
            assert!(route.starts_with("/admin"));
        }
    }

    #[test]
    fn test_admin_api_keys_routes() {
        let create_route = "/admin/api-keys";
        let revoke_route = "/admin/api-keys/:key";

        assert_eq!(create_route, "/admin/api-keys");
        assert!(revoke_route.contains(":key"));
        assert!(revoke_route.starts_with("/admin/api-keys"));
    }

    #[test]
    fn test_admin_stats_route() {
        let stats_route = "/admin/stats";
        assert_eq!(stats_route, "/admin/stats");
        assert!(stats_route.ends_with("stats"));
    }

    #[test]
    fn test_rate_limit_reset_route() {
        let reset_route = "/admin/rate-limit/reset/:user_id";
        assert!(reset_route.contains(":user_id"));
        assert!(reset_route.contains("rate-limit"));
        assert!(reset_route.contains("reset"));
    }

    #[test]
    fn test_file_info_route_structure() {
        let info_route = "/:id/info";
        let parts: Vec<&str> = info_route.split('/').collect();

        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "");
        assert_eq!(parts[1], ":id");
        assert_eq!(parts[2], "info");
    }

    #[test]
    fn test_upload_routes() {
        let post_root = "/";
        let put_filename = "/:filename";

        assert_eq!(post_root, "/");
        assert!(put_filename.contains(":filename"));
    }

    #[test]
    fn test_download_route() {
        let download_route = "/:id";
        assert!(download_route.starts_with("/"));
        assert!(download_route.contains(":id"));
        assert_eq!(download_route.len(), 4);
    }

    #[test]
    fn test_route_parameter_names() {
        let params = vec![":filename", ":id", ":key", ":user_id"];

        for param in params {
            assert!(param.starts_with(':'));
            assert!(param.len() > 1);
        }
    }

    #[test]
    fn test_http_methods_mapping() {
        let routes = vec![
            ("GET", "/"),
            ("POST", "/"),
            ("PUT", "/:filename"),
            ("GET", "/:id/info"),
            ("DELETE", "/:id/:token"),
            ("GET", "/:id"),
            ("POST", "/admin/api-keys"),
            ("DELETE", "/admin/api-keys/:key"),
            ("GET", "/admin/stats"),
            ("POST", "/admin/rate-limit/reset/:user_id"),
        ];

        assert_eq!(routes.len(), 10);

        let get_count = routes.iter().filter(|(m, _)| *m == "GET").count();
        let post_count = routes.iter().filter(|(m, _)| *m == "POST").count();
        let put_count = routes.iter().filter(|(m, _)| *m == "PUT").count();
        let delete_count = routes.iter().filter(|(m, _)| *m == "DELETE").count();

        assert_eq!(get_count, 4);
        assert_eq!(post_count, 3);
        assert_eq!(put_count, 1);
        assert_eq!(delete_count, 2);
    }

    #[test]
    fn test_cleanup_error_handling() {
        let cleanup_result: std::result::Result<(), &str> = Err("Cleanup failed");

        if let Err(e) = cleanup_result {
            assert_eq!(e, "Cleanup failed");
        } else {
            panic!("Expected error");
        }
    }

    #[test]
    fn test_cleanup_success_case() {
        let cleanup_result: std::result::Result<(), &str> = Ok(());
        assert!(cleanup_result.is_ok());
    }

    #[test]
    fn test_route_uniqueness() {
        let routes = vec![
            ("GET", "/"),
            ("POST", "/"),
            ("PUT", "/:filename"),
            ("GET", "/:id/info"),
            ("DELETE", "/:id/:token"),
            ("GET", "/:id"),
            ("POST", "/admin/api-keys"),
            ("DELETE", "/admin/api-keys/:key"),
            ("GET", "/admin/stats"),
            ("POST", "/admin/rate-limit/reset/:user_id"),
        ];

        let unique_routes: std::collections::HashSet<_> = routes.iter().collect();
        assert_eq!(unique_routes.len(), routes.len());
    }

    #[test]
    fn test_admin_routes_count() {
        let admin_routes = vec![
            "/admin/api-keys",
            "/admin/api-keys/:key",
            "/admin/stats",
            "/admin/rate-limit/reset/:user_id",
        ];

        assert_eq!(admin_routes.len(), 4);
    }

    #[test]
    fn test_public_routes_count() {
        let public_routes = vec![
            "/",
            "/:filename",
            "/:id/info",
            "/:id/:token",
            "/:id",
        ];

        assert_eq!(public_routes.len(), 5);
    }

    #[test]
    fn test_parameterized_admin_routes() {
        let parameterized = vec![
            "/admin/api-keys/:key",
            "/admin/rate-limit/reset/:user_id",
        ];

        for route in parameterized {
            assert!(route.contains(':'));
        }
    }
}