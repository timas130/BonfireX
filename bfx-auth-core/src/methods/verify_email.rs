use crate::AuthCoreService;
use crate::models::user::RawUser;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{VerifyEmailReply, VerifyEmailRequest};
use chrono::{TimeDelta, Utc};
use tonic::{Code, Request, Response, Status};

impl AuthCoreService {
    /// Verify a user's email using a verification token
    ///
    /// # Errors
    ///
    /// - If the verification token is invalid or not found
    /// - If the user is already active
    /// - If the verification code has expired
    /// - Miscellaneous internal errors
    pub async fn verify_email(
        &self,
        request: Request<VerifyEmailRequest>,
    ) -> Result<Response<VerifyEmailReply>, Status> {
        let request = request.into_inner();

        let mut user = RawUser::by_email_verification_code(self, &request.token)
            .await?
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::EmailCodeNotFound))?;

        if user.active {
            return Err(Status::coded(
                Code::InvalidArgument,
                ErrorCode::UserAlreadyActive,
            ));
        }

        // check if code expired
        if let Some(email_verification_sent_at) = user.email_verification_sent_at
            && Utc::now().signed_duration_since(email_verification_sent_at) > TimeDelta::hours(24)
        {
            return Err(Status::coded(
                Code::InvalidArgument,
                ErrorCode::EmailCodeExpired,
            ));
        }

        sqlx::query!(
            "update auth_core.users
             set active = true,
                 email_verification_sent_at = null,
                 email_verification_code = null
             where id = $1",
            user.id,
        )
        .execute(&self.db)
        .await
        .map_err(Status::db)?;

        user.active = true;
        user.email_verification_sent_at = None;
        user.email_verification_code = None;

        Ok(Response::new(VerifyEmailReply {
            user: Some(user.into()),
        }))
    }
}
