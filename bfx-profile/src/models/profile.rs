use bfx_proto::profile::ProfileDetails;
use o2o::o2o;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(o2o)]
#[owned_into(ProfileDetails)]
pub struct RawProfile {
    pub user_id: i64,
    pub display_name: Option<String>,
    pub username: String,
    #[into(avatar)]
    pub avatar_id: Option<i64>,
    pub bio: String,
    #[into(cover)]
    pub cover_id: Option<i64>,
    #[ghost]
    pub created_at: DateTime<Utc>,
    pub note: Option<String>,
}
