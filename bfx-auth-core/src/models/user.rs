use bfx_proto::auth::User;
use chrono::{DateTime, Utc};
use o2o::o2o;

#[derive(o2o)]
#[owned_into(User)]
pub struct RawUser {
    pub id: i64,
    pub email: Option<String>,
    pub permission_level: i32,
    pub banned: bool,
    pub active: bool,
    #[into(~.map(From::from))]
    pub email_verification_sent_at: Option<DateTime<Utc>>,
    #[ghost]
    pub password: Option<String>,
    #[into(Some(~.into()))]
    pub created_at: DateTime<Utc>,
}
