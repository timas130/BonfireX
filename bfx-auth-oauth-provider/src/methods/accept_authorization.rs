use crate::AuthOAuthProviderService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{AcceptAuthorizationReply, AcceptAuthorizationRequest};
use nanoid::nanoid;
use openidconnect::RedirectUrl;
use sqlx::types::chrono::Utc;
use std::time::Duration;
use tonic::{Code, Request, Response, Status};

impl AuthOAuthProviderService {
    /// Approve an OAuth authorization request
    ///
    /// # Errors
    ///
    /// - If the flow is not found or has already been approved
    /// - If the flow is expired
    /// - Miscellaneous internal errors
    pub async fn accept_authorization(
        &self,
        request: Request<AcceptAuthorizationRequest>,
    ) -> Result<Response<AcceptAuthorizationReply>, Status> {
        let request = request.into_inner();

        let flow = sqlx::query!(
            "select * from auth_oauth_provider.flows
             where id = $1 and user_id = $2 and code is null",
            request.flow_id,
            request.user_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?
        .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::FlowNotFound))?;

        if flow.created_at + Duration::from_secs(30 * 60) < Utc::now() {
            return Err(Status::coded(Code::NotFound, ErrorCode::FlowNotFound));
        }

        let grant = sqlx::query!(
            "insert into auth_oauth_provider.grants
             (client_id, user_id, scopes)
             values ($1, $2, $3)
             on conflict (user_id, client_id) do update
             set scopes = auth_oauth_provider.merge_arrays(grants.scopes, excluded.scopes)
             returning id",
            flow.client_id,
            flow.user_id,
            flow.scopes as Vec<String>,
        )
        .fetch_one(&self.db)
        .await
        .map_err(Status::db)?;

        let code = format!("BF/C/{}", nanoid!(32));

        sqlx::query!(
            "update auth_oauth_provider.flows
             set code = $1, grant_id = $2
             where id = $3",
            code,
            grant.id,
            flow.id,
        )
        .execute(&self.db)
        .await
        .map_err(Status::db)?;

        let redirect_uri = RedirectUrl::new(flow.redirect_uri.clone())
            .map_err(|_| Status::coded(Code::InvalidArgument, ErrorCode::InvalidRedirectUri))?;

        Ok(Response::new(AcceptAuthorizationReply {
            redirect_to: self.make_code_redirect(&redirect_uri, &code, flow.state.as_deref()),
        }))
    }
}
