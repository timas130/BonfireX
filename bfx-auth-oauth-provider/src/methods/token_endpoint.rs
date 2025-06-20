use crate::AuthOAuthProviderService;
use crate::methods::get_authorization_info::is_subset;
use crate::models::flow::Flow;
use bfx_core::log_if_error::LogIfErrorExt;
use bfx_core::service::id_encryption::IdType;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::auth_core_client::AuthCoreClient;
use bfx_proto::auth::{TokenEndpointReply, TokenEndpointRequest, User};
use bfx_proto::image::image_client::ImageClient;
use bfx_proto::profile::ProfileDetails;
use bfx_proto::profile::profile_client::ProfileClient;
use nanoid::nanoid;
use openidconnect::core::{
    CoreErrorResponseType, CoreGenderClaim, CoreGrantType, CoreIdToken, CoreIdTokenClaims,
    CoreIdTokenFields, CoreJwsSigningAlgorithm, CoreTokenResponse, CoreTokenType,
};
use openidconnect::{
    AccessToken, Audience, AuthorizationCode, ClientId, EmptyAdditionalClaims,
    EmptyExtraTokenFields, EndUserEmail, EndUserName, EndUserPictureUrl, EndUserProfileUrl,
    EndUserUsername, Nonce, PkceCodeChallenge, PkceCodeVerifier, RefreshToken, Scope,
    StandardClaims, SubjectIdentifier,
};
use serde_json::{Value, json};
use sqlx::types::chrono::Utc;
use sqlx::{Postgres, Transaction};
use std::time::Duration;
use tonic::{Code, Request, Response, Status};
use tracing::warn;

macro_rules! let_some {
    ($self:ident, $var:ident, $code:expr) => {
        let Some($var) = $var else {
            return Ok(Self::make_error_resp(
                $code,
                concat!("missing parameter `", stringify!($var), "`"),
            ));
        };
    };
    ($self:ident, $var:ident) => {
        let_some!($self, $var, CoreErrorResponseType::InvalidRequest);
    };
}

const OAUTH_ACCESS_TOKEN_LIFETIME: Duration = Duration::from_secs(3600);

impl AuthOAuthProviderService {
    /// `/openid/token` endpoint
    ///
    /// # Errors
    ///
    /// Only internal errors are returned as `Err`.
    /// All other request errors are returned as `Ok(Response)` with the appropriate error code.
    pub async fn token_endpoint(
        &self,
        request: Request<TokenEndpointRequest>,
    ) -> Result<Response<TokenEndpointReply>, Status> {
        let request = request.into_inner();

        let mut params = request.query;

        let (client_id, client_secret) = match request.authorization {
            Some(basic) => (Some(basic.username), Some(basic.password)),
            None => (None, None),
        };

        let client_id = params.remove("client_id").or(client_id);
        let client_secret = params.remove("client_secret").or(client_secret);

        let_some!(self, client_id, CoreErrorResponseType::InvalidClient);
        let_some!(self, client_secret, CoreErrorResponseType::InvalidClient);

        let grant_type = params.remove("grant_type");
        let_some!(self, grant_type);
        let grant_type: Result<CoreGrantType, _> =
            serde_json::from_value(Value::String(grant_type));
        let Ok(grant_type) = grant_type else {
            return Ok(Self::make_error_resp(
                CoreErrorResponseType::UnsupportedGrantType,
                "failed to parse grant_type",
            ));
        };

        let client = sqlx::query!(
            "select * from auth_oauth_provider.clients where client_id = $1",
            client_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?;

        let Some(client) = client else {
            return Ok(Self::make_error_resp(
                CoreErrorResponseType::InvalidClient,
                "client not found",
            ));
        };

        if client.client_secret != client_secret {
            return Ok(Self::make_error_resp(
                CoreErrorResponseType::InvalidClient,
                "wrong client secret",
            ));
        }

        match grant_type {
            CoreGrantType::AuthorizationCode => {
                let code = params.remove("code");
                let code_verifier = params.remove("code_verifier");
                let redirect_uri = params.remove("redirect_uri");
                let_some!(self, code, CoreErrorResponseType::InvalidRequest);

                let mut tx = self.db.begin().await.map_err(Status::db)?;

                let flow = sqlx::query_as!(
                    Flow,
                    "select * from auth_oauth_provider.flows
                     where code = $1 and client_id = $2 and authorized_at is null
                     for update",
                    code,
                    client.id,
                )
                .fetch_optional(&mut *tx)
                .await
                .map_err(Status::db)?;

                let Some(flow) = flow else {
                    tx.rollback().await.map_err(Status::db)?;
                    return Ok(Self::make_error_resp(
                        CoreErrorResponseType::InvalidGrant,
                        "invalid code",
                    ));
                };

                // check that code_verifier is specified if required or the other way around
                if flow.code_challenge.is_some() && code_verifier.is_none() {
                    tx.rollback().await.map_err(Status::db)?;
                    return Ok(Self::make_error_resp(
                        CoreErrorResponseType::InvalidRequest,
                        "code_verifier is required",
                    ));
                }
                if flow.code_challenge.is_none() && code_verifier.is_some() {
                    tx.rollback().await.map_err(Status::db)?;
                    return Ok(Self::make_error_resp(
                        CoreErrorResponseType::InvalidRequest,
                        "code_verifier is not required",
                    ));
                }

                // check that code_verifier matches
                if let Some(code_verifier) = code_verifier {
                    if code_verifier.len() < 43 || code_verifier.len() > 128 {
                        tx.rollback().await.map_err(Status::db)?;
                        return Ok(Self::make_error_resp(
                            CoreErrorResponseType::InvalidRequest,
                            "code_verifier length must be between 43 and 128 characters",
                        ));
                    }

                    let code_verifier = PkceCodeVerifier::new(code_verifier);
                    let code_challenge =
                        PkceCodeChallenge::from_code_verifier_sha256(&code_verifier);

                    if Some(code_challenge.as_str()) != flow.code_challenge.as_deref() {
                        tx.rollback().await.map_err(Status::db)?;
                        return Ok(Self::make_error_resp(
                            CoreErrorResponseType::InvalidGrant,
                            "code_verifier does not match",
                        ));
                    }
                }

                // check redirect_uri
                if redirect_uri.is_none() && client.redirect_uris.len() > 1 {
                    tx.rollback().await.map_err(Status::db)?;
                    return Ok(Self::make_error_resp(
                        CoreErrorResponseType::InvalidRequest,
                        "redirect_uri is required",
                    ));
                }
                if let Some(redirect_uri) = redirect_uri
                    && !client.redirect_uris.iter().any(|uri| uri == &redirect_uri)
                {
                    tx.rollback().await.map_err(Status::db)?;
                    return Ok(Self::make_error_resp(
                        CoreErrorResponseType::InvalidRequest,
                        "redirect_uri does not match",
                    ));
                }

                let ret = self
                    .get_token_response_for_flow(flow, None, Some(code), &mut tx)
                    .await?;

                tx.commit().await.map_err(Status::db)?;

                Ok(Response::new(ret))
            }
            CoreGrantType::RefreshToken => {
                let refresh_token = params.remove("refresh_token");
                let_some!(self, refresh_token, CoreErrorResponseType::InvalidRequest);

                let flow_id = refresh_token.split('/').nth(2);
                let Some(flow_id) = flow_id else {
                    return Ok(Self::make_error_resp(
                        CoreErrorResponseType::InvalidGrant,
                        "invalid refresh token",
                    ));
                };
                let flow_id = self.id_encryptor.decrypt_id(IdType::OAuthFlow, flow_id);
                let Ok(flow_id) = flow_id else {
                    return Ok(Self::make_error_resp(
                        CoreErrorResponseType::InvalidGrant,
                        "invalid refresh token",
                    ));
                };

                let mut tx = self.db.begin().await.map_err(Status::db)?;

                let flow = sqlx::query_as!(
                    Flow,
                    "select * from auth_oauth_provider.flows
                     where id = $1 and refresh_token = $2 and client_id = $3",
                    flow_id,
                    refresh_token,
                    client.id,
                )
                .fetch_optional(&mut *tx)
                .await
                .map_err(Status::db)?;

                let Some(mut flow) = flow else {
                    tx.rollback().await.map_err(Status::db)?;
                    return Ok(Self::make_error_resp(
                        CoreErrorResponseType::InvalidGrant,
                        "invalid refresh token",
                    ));
                };

                let scope = params.remove("scope");
                if let Some(scope) = scope {
                    let scope = scope.split(' ').collect::<Vec<_>>();

                    if !is_subset!(&scope, &flow.scopes) {
                        tx.rollback().await.map_err(Status::db)?;
                        return Ok(Self::make_error_resp(
                            CoreErrorResponseType::InvalidScope,
                            "invalid scope",
                        ));
                    }

                    flow.scopes = scope.into_iter().map(String::from).collect();
                }

                let ret = self
                    .get_token_response_for_flow(flow, Some(refresh_token), None, &mut tx)
                    .await?;

                tx.commit().await.map_err(Status::db)?;

                Ok(Response::new(ret))
            }
            typ => Ok(Self::make_error_resp(
                CoreErrorResponseType::UnsupportedGrantType,
                &format!("unsupported grant type `{}`", typ.as_ref()),
            )),
        }
    }

    async fn get_token_response_for_flow(
        &self,
        flow: Flow,
        refresh_token: Option<String>,
        code: Option<String>,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<TokenEndpointReply, Status> {
        let flow_id = self.id_encryptor.encrypt_id(IdType::OAuthFlow, flow.id);

        let access_token = format!("BF/A/{}/{}", flow_id, nanoid!(32));
        let refresh_token = if let Some(refresh_token) = refresh_token {
            Some(refresh_token)
        } else if flow.scopes.iter().any(|scope| scope == "offline_access") {
            Some(format!("BF/R/{}/{}", flow_id, nanoid!(32)))
        } else {
            None
        };

        let at_expiration = Utc::now() + OAUTH_ACCESS_TOKEN_LIFETIME;

        sqlx::query!(
            "update auth_oauth_provider.flows
             set access_token = $1,
                 refresh_token = $2,
                 access_token_expires_at = $3,
                 authorized_at = now()
             where id = $4",
            access_token,
            refresh_token,
            at_expiration,
            flow.id,
        )
        .execute(&mut **tx)
        .await
        .map_err(Status::db)?;

        let standard_claims = self.get_standard_claims(flow.user_id, &flow.scopes).await;

        let client_id = sqlx::query_scalar!(
            "select client_id from auth_oauth_provider.clients
             where id = $1",
            flow.client_id,
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(Status::db)?;

        let access_token = AccessToken::new(access_token);
        let id_token = CoreIdToken::new(
            CoreIdTokenClaims::new(
                // iss
                self.issuer.clone(),
                // aud
                vec![Audience::new(client_id.clone())],
                // exp
                at_expiration,
                // iat
                Utc::now(),
                standard_claims,
                EmptyAdditionalClaims {},
            )
            // azp
            .set_authorized_party(Some(ClientId::new(client_id)))
            // nonce
            .set_nonce(flow.nonce.map(Nonce::new)),
            // signing key
            &*self.rs_256_signing_key,
            // alg
            CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
            // at_hash
            Some(&access_token),
            // c_hash
            code.map(AuthorizationCode::new).as_ref(),
        )
        .map_err(|err| Status::coded(Code::Internal, ErrorCode::Internal).with_source(err))?;

        let mut resp = CoreTokenResponse::new(
            access_token,
            CoreTokenType::Bearer,
            CoreIdTokenFields::new(Some(id_token), EmptyExtraTokenFields {}),
        );
        resp.set_refresh_token(refresh_token.map(RefreshToken::new));
        resp.set_scopes(Some(flow.scopes.into_iter().map(Scope::new).collect()));
        resp.set_expires_in(Some(&OAUTH_ACCESS_TOKEN_LIFETIME));

        Ok(TokenEndpointReply {
            status: 200,
            json: serde_json::to_string(&resp).map_err(|err| {
                Status::coded(Code::Internal, ErrorCode::Internal).with_source(err)
            })?,
        })
    }

    /// Gets the ID token claims for a user
    pub async fn get_standard_claims(
        &self,
        user_id: i64,
        scopes: &[String],
    ) -> StandardClaims<CoreGenderClaim> {
        let mut auth_core = AuthCoreClient::new(self.router.clone());
        let mut profile = ProfileClient::new(self.router.clone());

        let user = auth_core
            .get_user_by_id(user_id)
            .await
            .or_with_log_default("getting user for claims");
        let profile = profile
            .get_profile_by_id(user_id)
            .await
            .or_with_log_default("getting profile for claims");

        let user_id = self.id_encryptor.encrypt_id(IdType::User, user_id);
        let mut standard_claims = StandardClaims::new(SubjectIdentifier::new(user_id));

        if let Some(user) = user {
            standard_claims = self.set_user_claims(standard_claims, user, scopes);
        }
        if let Some(profile) = profile {
            standard_claims = self
                .set_profile_claims(standard_claims, profile, scopes)
                .await;
        }

        standard_claims
    }

    pub fn set_user_claims(
        &self,
        mut standard_claims: StandardClaims<CoreGenderClaim>,
        user: User,
        scopes: &[String],
    ) -> StandardClaims<CoreGenderClaim> {
        if scopes.iter().any(|scope| scope == "email") {
            standard_claims = standard_claims
                .set_email(user.email.map(EndUserEmail::new))
                .set_email_verified(Some(user.active));
        }

        standard_claims
    }

    pub async fn set_profile_claims(
        &self,
        mut standard_claims: StandardClaims<CoreGenderClaim>,
        profile: ProfileDetails,
        scopes: &[String],
    ) -> StandardClaims<CoreGenderClaim> {
        if scopes.iter().any(|scope| scope == "profile") {
            let mut image = ImageClient::new(self.router.clone());

            // try to get avatar from profile (or fail silently)
            let avatar = if let Some(avatar) = profile.avatar {
                image
                    .get_image_ext(Some(avatar), format!("profile:{}:avatar", profile.user_id))
                    .await
                    .unwrap_or_else(|err| {
                        warn!(
                            ?err,
                            user_id = profile.user_id,
                            image_id = avatar,
                            "failed to get avatar for profile"
                        );
                        None
                    })
            } else {
                None
            };

            standard_claims = standard_claims
                .set_name(Some(EndUserName::new(profile.display_name().into()).into()))
                .set_preferred_username(Some(EndUserUsername::new(profile.username)))
                .set_profile(Some(
                    EndUserProfileUrl::new(format!(
                        "{}/user/{}",
                        self.frontend_root,
                        self.id_encryptor.encrypt_id(IdType::User, profile.user_id)
                    ))
                    .into(),
                ))
                .set_picture(avatar.and_then(|img| {
                    img.full
                        .or(img.thumbnail)
                        .map(|img| EndUserPictureUrl::new(img.url).into())
                }));
        }

        standard_claims
    }

    #[allow(clippy::needless_pass_by_value)] // for convenience (this is called many times)
    fn make_error_resp(
        error: CoreErrorResponseType,
        description: &str,
    ) -> Response<TokenEndpointReply> {
        Response::new(TokenEndpointReply {
            status: if error == CoreErrorResponseType::InvalidClient {
                401
            } else {
                400
            },
            json: serde_json::to_string(&json!({
                "error": error,
                "error_description": description,
            }))
            .unwrap(),
        })
    }
}
