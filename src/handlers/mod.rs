pub mod upload;
pub mod download;
pub mod delete;
pub mod get;
pub mod inline;
pub mod info;
pub mod admin;
pub mod landing;

pub use upload::handle_upload;
pub use download::handle_download;
pub use delete::handle_delete;
pub use get::handle_get;
pub use inline::handle_inline;
pub use info::handle_info;
pub use admin::{handle_create_key, handle_revoke_key, handle_get_stats, handle_reset_rate_limit};
pub use landing::handle_landing;
