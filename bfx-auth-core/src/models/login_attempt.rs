use chrono::{DateTime, Utc};

#[allow(unused)]
pub struct RawLoginAttempt {
    pub id: i64,
    pub user_id: i64,
    pub user_context_id: i64,
    pub status: i32,
    pub created_at: DateTime<Utc>,
}
