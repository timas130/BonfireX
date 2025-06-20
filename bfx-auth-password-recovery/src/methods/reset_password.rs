use crate::PasswordRecoveryService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::auth_core_client::AuthCoreClient;
use bfx_proto::auth::{ChangePasswordRequest, ResetPasswordReply, ResetPasswordRequest};
use tonic::{Code, Request, Response, Status};

impl PasswordRecoveryService {
    pub async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordReply>, Status> {
        let request = request.into_inner();

        let user_context = request
            .user_context
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

        let mut tx = self.db.begin().await.map_err(Status::db)?;

        let token = sqlx::query!(
            "update auth_password_recovery.password_reset_requests
             set used_at = now()
             where token = $1 and used_at is null and created_at > (now() - interval '24 hours')
             returning user_id",
            request.token
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Status::db)?;

        let mut auth_core = AuthCoreClient::new(self.router.clone());

        auth_core
            .change_password(ChangePasswordRequest {
                user_id: token.user_id,
                old_password: None,
                new_password: request.new_password,
                user_context: Some(user_context),
                terminate_all_sessions: true,
            })
            .await?;

        tx.commit().await.map_err(Status::db)?;

        Ok(Response::new(ResetPasswordReply {}))
    }
}
