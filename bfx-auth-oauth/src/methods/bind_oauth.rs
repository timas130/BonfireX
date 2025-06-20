use crate::AuthOAuthService;
use crate::client::OAuthClients;
use crate::models::raw_auth_source::RawAuthSource;
use bfx_core::log_if_error::LogIfErrorExt;
use bfx_core::service::database::DbResultExt;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{BindOAuthReply, BindOAuthRequest};
use bfx_proto::notification::SendNotificationRequest;
use bfx_proto::notification::notification_client::NotificationClient;
use bfx_proto::param_map;
use sqlx::types::chrono::Utc;
use tonic::{Code, Request, Response, Status};

impl AuthOAuthService {
    /// Bind an external account to an existing user
    ///
    /// # Errors
    ///
    /// - If [`AuthOAuthService::get_id_token_claims`] fails
    /// - If an account from the issuer has already been bound
    /// - Miscellaneous internal errors
    pub async fn bind_oauth(
        &self,
        request: Request<BindOAuthRequest>,
    ) -> Result<Response<BindOAuthReply>, Status> {
        let request = request.into_inner();

        let finish_request = request
            .finish_request
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

        let user_context = finish_request
            .user_context
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

        let claims = self
            .get_id_token_claims(
                &finish_request.issuer,
                finish_request.state,
                finish_request.code,
            )
            .await?;

        let result = sqlx::query_as!(
            RawAuthSource,
            "insert into auth_oauth.auth_sources (user_id, issuer, issuer_user_id)
             values ($1, $2, $3)
             returning *",
            request.user_id,
            finish_request.issuer,
            claims.subject().as_str(),
        )
        .fetch_one(&self.db)
        .await;
        if result.is_unique_violation() {
            return Err(Status::coded(
                Code::AlreadyExists,
                ErrorCode::OAuthAlreadyBound,
            ));
        }

        let mut notification = NotificationClient::new(self.router.clone());
        notification
            .send_notification(SendNotificationRequest {
                user_id: request.user_id,
                user_override: None,
                definition: include_str!("../../notifications/oauth_bound.yml").to_string(),
                params: param_map! {
                    "audit_ip" => user_context.ip,
                    "audit_time" => Utc::now().to_rfc3339(),
                    "provider" => OAuthClients::get_provider_name(&finish_request.issuer),
                },
            })
            .await
            .log_if_error("sending oauth bound notification");

        Ok(Response::new(BindOAuthReply {
            auth_source: Some(result.map_err(Status::db)?.into()),
        }))
    }
}
