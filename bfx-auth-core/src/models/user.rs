use crate::AuthCoreService;
use bfx_core::status::StatusExt;
use bfx_proto::auth::User;
use chrono::{DateTime, Utc};
use o2o::o2o;
use tonic::Status;

#[derive(o2o)]
#[owned_into(User)]
#[derive(Debug)]
pub struct RawUser {
    pub id: i64,
    pub email: Option<String>,
    pub permission_level: i32,
    pub banned: bool,
    pub active: bool,
    #[into(~.map(From::from))]
    pub email_verification_sent_at: Option<DateTime<Utc>>,
    #[ghost]
    pub email_verification_code: Option<String>,
    #[ghost]
    pub password: Option<String>,
    #[into(Some(~.into()))]
    pub created_at: DateTime<Utc>,
}

impl RawUser {
    /// Find a user by their ID
    ///
    /// # Errors
    ///
    /// - If the database query fails
    pub async fn by_id(service: &AuthCoreService, id: i64) -> Result<Option<Self>, Status> {
        let user = sqlx::query_as!(RawUser, "select * from auth_core.users where id = $1", id)
            .fetch_optional(&service.db)
            .await
            .map_err(Status::db)?;

        Ok(user)
    }

    /// Find a user by their email address
    ///
    /// # Errors
    ///
    /// - If the database query fails
    pub async fn by_email(service: &AuthCoreService, email: &str) -> Result<Option<Self>, Status> {
        let user = sqlx::query_as!(
            RawUser,
            "select * from auth_core.users where email = $1",
            email,
        )
        .fetch_optional(&service.db)
        .await
        .map_err(Status::db)?;

        Ok(user)
    }

    /// Find a user by their email verification code
    ///
    /// # Errors
    ///
    /// - If the database query fails
    pub async fn by_email_verification_code(
        service: &AuthCoreService,
        code: &str,
    ) -> Result<Option<Self>, Status> {
        let user = sqlx::query_as!(
            RawUser,
            "select * from auth_core.users where email_verification_code = $1",
            code,
        )
        .fetch_optional(&service.db)
        .await
        .map_err(Status::db)?;

        Ok(user)
    }
}
