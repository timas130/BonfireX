use crate::AuthOAuthProviderService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{UserinfoEndpointReply, UserinfoEndpointRequest};
use openidconnect::EmptyAdditionalClaims;
use openidconnect::core::CoreUserInfoClaims;
use tonic::{Code, Request, Response, Status};

impl AuthOAuthProviderService {
    /// `/openid/userinfo` endpoint
    ///
    /// # Errors
    ///
    /// - If [`AuthOAuthProviderService::get_access_token`] fails
    /// - Miscellaneous internal errors
    pub async fn userinfo_endpoint(
        &self,
        request: Request<UserinfoEndpointRequest>,
    ) -> Result<Response<UserinfoEndpointReply>, Status> {
        let request = request.into_inner();

        let token = self.get_access_token(request.access_token).await?;

        let standard_claims = self.get_standard_claims(token.user_id, &token.scope).await;

        let userinfo_response = CoreUserInfoClaims::new(standard_claims, EmptyAdditionalClaims {})
            .set_issuer(Some(self.issuer.clone()));

        Ok(Response::new(UserinfoEndpointReply {
            status: 200,
            json: serde_json::to_string(&userinfo_response).map_err(|err| {
                Status::coded(Code::Internal, ErrorCode::Internal).with_source(err)
            })?,
        }))
    }
}
