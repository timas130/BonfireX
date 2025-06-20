use bfx_proto::auth::Tokens;
use chrono::{DateTime, Utc};

pub struct RawSession {
    pub id: i64,
    pub user_id: i64,
    pub login_attempt_id: Option<i64>,
    pub last_user_context_id: i64,
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<RawSession> for Tokens {
    fn from(session: RawSession) -> Self {
        Self {
            access_token: session.access_token,
            session_id: session.id,
            login_attempt_id: session.login_attempt_id,
        }
    }
}
