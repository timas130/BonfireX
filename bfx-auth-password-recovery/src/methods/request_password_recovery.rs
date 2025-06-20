use crate::PasswordRecoveryService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::auth_core_client::AuthCoreClient;
use bfx_proto::auth::{
    GetUserByEmailRequest, RequestPasswordRecoveryReply, RequestPasswordRecoveryRequest,
};
use bfx_proto::notification::SendNotificationRequest;
use bfx_proto::notification::notification_client::NotificationClient;
use bfx_proto::param_map;
use tonic::{Code, Request, Response, Status};

impl PasswordRecoveryService {
    pub async fn request_password_recovery(
        &self,
        request: Request<RequestPasswordRecoveryRequest>,
    ) -> Result<Response<RequestPasswordRecoveryReply>, Status> {
        let request = request.into_inner();

        let RequestPasswordRecoveryRequest { email, .. } = request;

        let mut auth_core = AuthCoreClient::new(self.router.clone());

        let user = auth_core
            .get_user_by_email(GetUserByEmailRequest { email })
            .await?
            .into_inner()
            .user
            .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::UserNotFound))?;

        let mut tx = self.db.begin().await.map_err(Status::db)?;

        // check if the user is already requesting a password reset
        // (1-hour timeout)
        let already_requested = sqlx::query_scalar!(
            "select 1 as \"found!\"
             from auth_password_recovery.password_reset_requests
             where user_id = $1 and used_at is null
                   and created_at > (now() - interval '1 hour')",
            user.id
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(Status::db)?
        .is_some();

        if already_requested {
            tx.rollback().await.map_err(Status::db)?;
            return Err(Status::coded(
                Code::ResourceExhausted,
                ErrorCode::TooManyRequests,
            ));
        }

        // create a new password reset request
        let password_reset_request = sqlx::query!(
            "insert into auth_password_recovery.password_reset_requests
             (user_id, token)
             values ($1, $2)
             returning *",
            user.id,
            nanoid::nanoid!(32)
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Status::db)?;

        // send the email
        let mut notification = NotificationClient::new(self.router.clone());

        notification
            .send_notification(SendNotificationRequest {
                user_id: user.id,
                user_override: None,
                definition: include_str!("../../notifications/password_reset.yml").to_string(),
                params: param_map! {
                    "reset_url" => format!(
                        "{}/auth/reset-password?token={}",
                        self.frontend_root, password_reset_request.token
                    )
                },
            })
            .await?;

        tx.commit().await.map_err(Status::db)?;

        Ok(Response::new(RequestPasswordRecoveryReply {}))
    }
}
