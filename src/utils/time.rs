use chrono::Utc;

pub fn current_timestamp() -> i64 {
    Utc::now().timestamp()
}
