use crate::AuthOAuthProviderService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{GetAccessTokenReply, GetAccessTokenRequest};
use tonic::{Code, Request, Response, Status};

impl AuthOAuthProviderService {
    /// Gets information about an OAuth access token
    ///
    /// # Errors
    ///
    /// - If the access token is not found
    /// - If the access token is expired
    /// - Miscellaneous internal errors
    pub async fn get_access_token(
        &self,
        access_token: String,
    ) -> Result<GetAccessTokenReply, Status> {
        let flow = sqlx::query!(
            "select
                 f.scopes,
                 f.grant_id as \"grant_id!\",
                 f.client_id,
                 g.user_id
             from auth_oauth_provider.flows f
             inner join auth_oauth_provider.grants g on f.grant_id = g.id
             where f.access_token = $1 and f.access_token_expires_at > now()
                   and grant_id is not null",
            access_token,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?
        .ok_or_else(|| Status::coded(Code::Unauthenticated, ErrorCode::InvalidToken))?;

        Ok(GetAccessTokenReply {
            user_id: flow.user_id,
            grant_id: flow.grant_id,
            client_id: flow.client_id,
            scope: flow.scopes,
        })
    }

    /// Gets information about an OAuth access token (RPC wrapper)
    ///
    /// # Errors
    ///
    /// See [`AuthOAuthProviderService::get_access_token`]
    pub async fn get_access_token_rpc(
        &self,
        request: Request<GetAccessTokenRequest>,
    ) -> Result<Response<GetAccessTokenReply>, Status> {
        let request = request.into_inner();

        Ok(Response::new(
            self.get_access_token(request.access_token).await?,
        ))
    }
}
