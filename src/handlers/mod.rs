pub mod upload;
pub mod download;
pub mod delete;
pub mod info;
pub mod admin;
pub mod landing;

pub use upload::handle_upload;
pub use download::{handle_download, handle_get, handle_inline};
pub use delete::handle_delete;
pub use info::handle_info;
pub use admin::{handle_create_key, handle_revoke_key, handle_get_stats, handle_reset_rate_limit};
pub use landing::handle_landing;
