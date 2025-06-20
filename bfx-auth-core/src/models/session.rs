use bfx_proto::auth::Session;
use chrono::{DateTime, Utc};
use o2o::o2o;

pub struct RawSession {
    pub id: i64,
    pub user_id: i64,
    pub login_attempt_id: Option<i64>,
    pub last_user_context_id: i64,
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
