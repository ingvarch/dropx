use worker::{Request, Response, RouteContext, Result};
use crate::models::FileInfo;
use crate::services::DatabaseService;
use crate::utils::response::{get_required_param, not_found};

pub async fn handle_info(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let file_id = get_required_param(&ctx, "id", "File ID is required")?;

    let file_metadata = match DatabaseService::get_file(&ctx.env, file_id).await? {
        Some(metadata) => metadata,
        None => {
            return not_found(
                "File not found",
                "The requested file does not exist or has expired",
            );
        }
    };

    let file_info: FileInfo = file_metadata.into();
    Response::from_json(&file_info)
}
