use bfx_proto::auth::AuthSource;
use o2o::o2o;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Clone, o2o)]
#[owned_into(AuthSource)]
pub struct RawAuthSource {
    pub id: i64,
    pub user_id: i64,
    pub issuer: String,
    pub issuer_user_id: String,
    #[into(Some(~.into()))]
    pub created_at: DateTime<Utc>,
}
