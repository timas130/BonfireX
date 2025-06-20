use crate::AuthCoreService;
use crate::models::user::RawUser;
use bfx_core::log_if_error::LogIfErrorExt;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{ChangePasswordReply, ChangePasswordRequest};
use bfx_proto::notification::notification_client::NotificationClient;
use bfx_proto::param_map;
use chrono::Utc;
use tonic::{Code, Request, Response, Status};

impl AuthCoreService {
    /// Change a user's password
    ///
    /// # Errors
    ///
    /// - If the user is not found
    /// - If `old_password` is provided but doesn't match the current password
    /// - If the new password is too weak
    /// - Miscellaneous internal errors
    pub async fn change_password(
        &self,
        request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordReply>, Status> {
        let request = request.into_inner();

        let user_context = request
            .user_context
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

        // check new password strength
        self.check_password(&request.new_password, &[])?;

        let mut tx = self.db.begin().await.map_err(Status::db)?;

        let user = sqlx::query!(
            "select id, password from auth_core.users where id = $1 for update",
            request.user_id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(Status::db)?
        .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::UserNotFound))?;

        // check the old password if provided and it exists
        if let Some(old_password) = &request.old_password
            && let Some(current_hash) = &user.password
        {
            let password_valid = self
                .verify_password(old_password, current_hash)
                .map_err(Status::anyhow)?;

            if !password_valid {
                return Err(Status::coded(
                    Code::InvalidArgument,
                    ErrorCode::IncorrectPassword,
                ));
            }
        }

        // update the password
        let new_password_hash = self
            .hash_password(&request.new_password)
            .map_err(Status::anyhow)?;

        let user = sqlx::query_as!(
            RawUser,
            "update auth_core.users set password = $1 where id = $2 returning *",
            new_password_hash,
            request.user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Status::db)?;

        if request.terminate_all_sessions {
            sqlx::query!(
                "update auth_core.sessions set expires_at = now() where user_id = $1",
                request.user_id
            )
            .execute(&mut *tx)
            .await
            .map_err(Status::db)?;
        }

        tx.commit().await.map_err(Status::db)?;

        // send notifying email
        let mut notification = NotificationClient::new(self.router.clone());
        notification
            .send_notification(bfx_proto::notification::SendNotificationRequest {
                user_id: user.id,
                user_override: Some(user.into()),
                definition: include_str!("../../notifications/password_change.yml").to_string(),
                params: param_map! {
                    "audit_time" => Utc::now().to_rfc3339(),
                    "audit_ip" => user_context.ip,
                },
            })
            .await
            .log_if_error("sending password change notification");

        Ok(Response::new(ChangePasswordReply {}))
    }
}
