use crate::PasswordRecoveryService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{CheckPasswordResetReply, CheckPasswordResetTokenRequest};
use bfx_proto::profile::profile_client::ProfileClient;
use chrono::{TimeDelta, Utc};
use tonic::{Code, Request, Response, Status};

impl PasswordRecoveryService {
    pub async fn check_password_reset_token(
        &self,
        request: Request<CheckPasswordResetTokenRequest>,
    ) -> Result<Response<CheckPasswordResetReply>, Status> {
        let request = request.into_inner();

        let mut tx = self.db.begin().await.map_err(Status::db)?;

        let request = sqlx::query!(
            "select *
             from auth_password_recovery.password_reset_requests
             where token = $1
             for update",
            request.token
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(Status::db)?
        .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::RecoveryTokenNotFound))?;

        // check if the token is already used
        if request.used_at.is_some() {
            return Err(Status::coded(
                Code::InvalidArgument,
                ErrorCode::RecoveryTokenUsed,
            ));
        }

        // check expiration
        if Utc::now().signed_duration_since(request.created_at) > TimeDelta::hours(24) {
            return Err(Status::coded(
                Code::InvalidArgument,
                ErrorCode::RecoveryTokenExpired,
            ));
        }

        // get the profile to return user's username
        let mut profile = ProfileClient::new(self.router.clone());
        let username = profile
            .get_profile_by_id(request.user_id)
            .await?
            .map(|profile| profile.username);

        tx.commit().await.map_err(Status::db)?;

        Ok(Response::new(CheckPasswordResetReply { username }))
    }
}
