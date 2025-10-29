use worker::{Request, Response, RouteContext, Result};
use crate::models::FileInfo;
use crate::services::DatabaseService;
use crate::utils::response::{get_file_id_param, file_not_found};

pub async fn handle_info(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let file_id = get_file_id_param(&ctx)?;

    let file_metadata = match DatabaseService::get_file(&ctx.env, file_id).await? {
        Some(metadata) => metadata,
        None => return file_not_found(),
    };

    let file_info: FileInfo = file_metadata.into();
    Response::from_json(&file_info)
}
