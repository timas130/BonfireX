use crate::AuthOAuthService;
use crate::client::OAuthClients;
use crate::models::raw_auth_source::RawAuthSource;
use bfx_core::log_if_error::LogIfErrorExt;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{UnbindAuthSourceReply, UnbindAuthSourceRequest};
use bfx_proto::notification::SendNotificationRequest;
use bfx_proto::notification::notification_client::NotificationClient;
use bfx_proto::param_map;
use tonic::{Code, Request, Response, Status};

impl AuthOAuthService {
    /// Unbind an external auth source
    ///
    /// # Errors
    ///
    /// - If the auth source does not exist
    /// - Miscellaneous internal errors
    pub async fn unbind_auth_source(
        &self,
        request: Request<UnbindAuthSourceRequest>,
    ) -> Result<Response<UnbindAuthSourceReply>, Status> {
        let request = request.into_inner();

        let user_context = request
            .user_context
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

        let auth_source = sqlx::query_as!(
            RawAuthSource,
            "delete from auth_oauth.auth_sources
             where id = $1 and user_id = $2
             returning *",
            request.auth_source_id,
            request.user_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?
        .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::AuthSourceNotFound))?;

        let mut notification = NotificationClient::new(self.router.clone());
        notification
            .send_notification(SendNotificationRequest {
                user_id: auth_source.user_id,
                user_override: None,
                definition: include_str!("../../notifications/oauth_unbound.yml").to_string(),
                params: param_map! {
                    "audit_ip" => user_context.ip.to_string(),
                    "audit_time" => user_context.user_agent,
                    "provider" => OAuthClients::get_provider_name(&auth_source.issuer),
                },
            })
            .await
            .log_if_error("sending oauth unbound notification");

        Ok(Response::new(UnbindAuthSourceReply {}))
    }
}
