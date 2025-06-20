use crate::AuthCoreService;
use crate::models::user::RawUser;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{SendVerificationEmailReply, SendVerificationEmailRequest};
use bfx_proto::notification::SendNotificationRequest;
use bfx_proto::notification::notification_client::NotificationClient;
use bfx_proto::param_map;
use chrono::{TimeDelta, Utc};
use nanoid::nanoid;
use tonic::{Code, Request, Response, Status};

impl AuthCoreService {
    /// Send a verification email to a user
    ///
    /// # Errors
    ///
    /// - If the user is not found
    /// - If the user is already active
    /// - If too many requests have been made recently
    /// - If the email sending fails
    /// - Miscellaneous internal errors
    ///
    /// # Panics
    ///
    /// - If the user email is None (should not happen for valid users)
    pub async fn send_verification_email(
        &self,
        request: Request<SendVerificationEmailRequest>,
    ) -> Result<Response<SendVerificationEmailReply>, Status> {
        let request = request.into_inner();

        let user = RawUser::by_email(self, &request.email)
            .await?
            .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::UserNotFound))?;

        // don't reverify active users
        if user.active {
            return Err(Status::coded(
                Code::InvalidArgument,
                ErrorCode::UserAlreadyActive,
            ));
        }

        // check if another email was sent recently
        if let Some(email_verification_sent_at) = user.email_verification_sent_at
            && Utc::now().signed_duration_since(email_verification_sent_at)
                < TimeDelta::seconds(600)
        {
            return Err(Status::coded(
                Code::ResourceExhausted,
                ErrorCode::TooManyRequests,
            ));
        }

        // store the verification code and update email_verification_sent_at
        let email_verification_code = user
            .email_verification_code
            .clone()
            .unwrap_or_else(|| nanoid!());

        let mut tx = self.db.begin().await.map_err(Status::db)?;

        sqlx::query!(
            "update auth_core.users
             set email_verification_code = $1, email_verification_sent_at = now()
             where id = $2",
            email_verification_code,
            user.id,
        )
        .execute(&mut *tx)
        .await
        .map_err(Status::db)?;

        // send the email
        let mut notification = NotificationClient::new(self.router.clone());

        notification
            .send_notification(SendNotificationRequest {
                user_id: user.id,
                user_override: Some(user.into()),
                definition: include_str!("../../notifications/email_verification.yml").to_string(),
                params: param_map! {
                    "verify_url" => format!(
                        "{}/auth/verify-email?token={}",
                        self.frontend_root, email_verification_code
                    )
                },
            })
            .await?;

        tx.commit().await.map_err(Status::db)?;

        Ok(Response::new(SendVerificationEmailReply {}))
    }
}
