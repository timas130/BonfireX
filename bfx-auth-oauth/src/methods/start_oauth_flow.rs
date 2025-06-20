use crate::AuthOAuthService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{StartOAuthFlowReply, StartOAuthFlowRequest};
use openidconnect::core::CoreAuthenticationFlow;
use openidconnect::{CsrfToken, Nonce, PkceCodeChallenge, Scope};
use tonic::{Code, Request, Response, Status};

impl AuthOAuthService {
    /// Get the URL to start an external authorization
    ///
    /// # Errors
    ///
    /// - If the issuer is not supported
    /// - Miscellaneous internal errors
    pub async fn start_oauth_flow(
        &self,
        request: Request<StartOAuthFlowRequest>,
    ) -> Result<Response<StartOAuthFlowReply>, Status> {
        let request = request.into_inner();

        let client = self.clients.get_provider(&request.issuer)?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let req = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .set_pkce_challenge(pkce_challenge.clone());
        let (url, csrf_token, nonce) = req.url();

        sqlx::query!(
            "insert into auth_oauth.flows
             (issuer, state, nonce, pkce_verifier)
             values ($1, $2, $3, $4)",
            request.issuer,
            csrf_token.secret(),
            nonce.secret(),
            pkce_verifier.secret(),
        )
        .execute(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(Response::new(StartOAuthFlowReply {
            scope: url
                .query_pairs()
                .find(|(k, _)| k == "scope")
                .ok_or_else(|| Status::coded(Code::Internal, ErrorCode::Internal))?
                .1
                .to_string(),
            // I have no idea why state has into_secret but nonce doesn't
            state: csrf_token.into_secret(),
            nonce: nonce.secret().clone(),
            url: url.to_string(),
            code_challenge: Some(pkce_challenge.as_str().to_owned()),
            code_challenge_method: Some(pkce_challenge.method().to_string()),
        }))
    }
}
