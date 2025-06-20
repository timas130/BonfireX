use crate::AuthOAuthService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::UserContext;
use bfx_proto::auth::auth_core_client::AuthCoreClient;
use bfx_proto::auth::{
    CreateUserRequest, FinishOAuthFlowReply, FinishOAuthFlowRequest, LoginExternalRequest, Tokens,
};
use openidconnect::core::CoreIdTokenClaims;
use openidconnect::{
    AccessTokenHash, AuthorizationCode, Nonce, OAuth2TokenResponse, PkceCodeVerifier, TokenResponse,
};
use std::error::Error as StdError;
use tonic::{Code, Request, Response, Status};

trait ResultExt<T> {
    fn provider_err(self) -> Result<T, Status>;
}

impl<T, E: StdError + Send + Sync + 'static> ResultExt<T> for Result<T, E> {
    fn provider_err(self) -> Result<T, Status> {
        self.map_err(|e| Status::coded(Code::Internal, ErrorCode::ProviderError).with_source(e))
    }
}

impl AuthOAuthService {
    /// Consume the authorization code from an OAuth provider and finish the flow
    ///
    /// # Errors
    ///
    /// - If [`AuthOAuthService::get_id_token_claims`] fails
    /// - If the ID token doesn't contain an email, or it's not verified
    /// - If calling [`AuthCoreClient::login_external`] fails
    /// - If `AuthOAuthService::register_user` fails (if [`AuthCoreClient::create_user`] fails)
    /// - Miscellaneous internal errors
    pub async fn finish_oauth_flow(
        &self,
        request: Request<FinishOAuthFlowRequest>,
    ) -> Result<Response<FinishOAuthFlowReply>, Status> {
        let request = request.into_inner();

        let user_context = request
            .user_context
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

        // 1. exchange code and get id token claims
        let claims = self
            .get_id_token_claims(&request.issuer, request.state, request.code)
            .await?;

        let Some(email) = claims.email() else {
            return Err(Status::coded(Code::Internal, ErrorCode::ProviderEmailError)
                .with_details("no email"));
        };
        if !claims.email_verified().unwrap_or(false) {
            return Err(Status::coded(Code::Internal, ErrorCode::ProviderEmailError)
                .with_details("email is not verified"));
        }

        // 2. get existing auth_source
        let auth_source = sqlx::query!(
            "select * from auth_oauth.auth_sources where issuer = $1 and issuer_user_id = $2",
            &request.issuer,
            claims.subject().as_str(),
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?;

        // 3. log in if auth source exists or register a new user
        let tokens = if let Some(auth_source) = auth_source {
            // get user
            let mut auth_core = AuthCoreClient::new(self.router.clone());
            let user = auth_core
                .get_user_by_id(auth_source.user_id)
                .await?
                .ok_or_else(|| {
                    Status::coded(Code::Internal, ErrorCode::Internal)
                        .with_details("user from auth_source not found")
                })?;

            // get tokens
            auth_core
                .login_external(LoginExternalRequest {
                    user_id: user.id,
                    user_context: Some(user_context),
                })
                .await?
                .into_inner()
                .tokens
                .ok_or_else(|| {
                    Status::coded(Code::Internal, ErrorCode::Internal)
                        .with_details("external login failed")
                })?
        } else {
            // if no auth source, try to register a new user
            let tokens = self
                .register_user(
                    email.as_str().to_string(),
                    user_context,
                    &request.issuer,
                    claims.subject().as_str(),
                )
                .await?;

            if let Some(tokens) = tokens {
                tokens
            } else {
                return Ok(Response::new(FinishOAuthFlowReply {
                    existing_email: true,
                    tokens: None,
                }));
            }
        };

        Ok(Response::new(FinishOAuthFlowReply {
            existing_email: false,
            tokens: Some(tokens),
        }))
    }

    /// Get ID token claims from an OAuth provider using an authorization code
    ///
    /// # Errors
    ///
    /// - If the flow corresponding to the issuer and the state is not found
    /// - If calling the provider fails
    /// - If ID token verification fails: signature, `a_hash`, `nonce`
    /// - Miscellaneous internal errors
    pub async fn get_id_token_claims(
        &self,
        issuer: &str,
        state: String,
        code: String,
    ) -> Result<CoreIdTokenClaims, Status> {
        let client = self.clients.get_provider(issuer)?;

        let flow = sqlx::query!(
            "delete from auth_oauth.flows where issuer = $1 and state = $2 returning *",
            issuer,
            state,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?
        .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::FlowNotFound))?;

        let token_response = client
            .exchange_code(AuthorizationCode::new(code))
            .provider_err()?
            .set_pkce_verifier(PkceCodeVerifier::new(flow.pkce_verifier))
            .request_async(&self.clients.http_client)
            .await
            .provider_err()?;

        let nonce = Nonce::new(flow.nonce);

        // verify id token
        let id_token = token_response.id_token().ok_or_else(|| {
            Status::coded(Code::Internal, ErrorCode::ProviderError).with_details("no id token")
        })?;
        let id_token_verifier = client.id_token_verifier();
        let claims: &CoreIdTokenClaims =
            id_token.claims(&id_token_verifier, &nonce).provider_err()?;

        // verify access token hash if present
        if let Some(access_token_hash) = claims.access_token_hash() {
            let actual_access_token_hash = AccessTokenHash::from_token(
                token_response.access_token(),
                id_token.signing_alg().provider_err()?,
                id_token.signing_key(&id_token_verifier).provider_err()?,
            )
            .provider_err()?;
            if actual_access_token_hash != *access_token_hash {
                return Err(Status::coded(Code::Internal, ErrorCode::ProviderError)
                    .with_details("access token hash mismatch"));
            }
        }

        Ok(claims.clone())
    }

    /// Register a new user from an OAuth provider
    ///
    /// # Errors
    ///
    /// - If calling [`AuthCoreClient::create_user`] fails
    /// - Miscellaneous internal errors
    async fn register_user(
        &self,
        email: String,
        user_context: UserContext,
        issuer: &str,
        issuer_user_id: &str,
    ) -> Result<Option<Tokens>, Status> {
        let mut auth_core = AuthCoreClient::new(self.router.clone());

        let resp = auth_core
            .create_user(CreateUserRequest {
                email: Some(email),
                active: true,
                password: None,
                user_context: Some(user_context),
            })
            .await;
        if resp.as_ref().err().and_then(Status::to_error_code) == Some(ErrorCode::EmailExists) {
            return Ok(None);
        }
        let resp = resp?.into_inner();

        let user = resp
            .user
            .ok_or_else(|| Status::coded(Code::Internal, ErrorCode::Internal))?;

        sqlx::query!(
            "insert into auth_oauth.auth_sources (user_id, issuer, issuer_user_id)
             values ($1, $2, $3)",
            user.id,
            issuer,
            issuer_user_id,
        )
        .execute(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(Some(resp.tokens.ok_or_else(|| {
            Status::coded(Code::Internal, ErrorCode::Internal)
        })?))
    }
}
