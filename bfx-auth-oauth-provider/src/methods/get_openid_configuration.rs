use crate::AuthOAuthProviderService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{GetOpenidConfigurationReply, GetOpenidConfigurationRequest};
use openidconnect::core::{
    CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClientAuthMethod, CoreGrantType,
    CoreJsonWebKey, CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm,
    CoreJwsSigningAlgorithm, CoreResponseMode, CoreResponseType, CoreSubjectIdentifierType,
};
use openidconnect::{
    AdditionalProviderMetadata, AuthUrl, JsonWebKeySetUrl, PkceCodeChallengeMethod,
    ProviderMetadata, ResponseTypes, Scope, TokenUrl, UserInfoUrl,
};
use serde::{Deserialize, Serialize};
use tonic::{Code, Request, Response, Status};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BfxAdditionalProviderMetadata {
    // no idea why it's not included in `openidconnect`, it's part of the spec:
    // https://www.rfc-editor.org/rfc/rfc8414.html#section-2
    pub code_challenge_methods_supported: Vec<PkceCodeChallengeMethod>,
}
impl AdditionalProviderMetadata for BfxAdditionalProviderMetadata {}

pub type BfxProviderMetadata = ProviderMetadata<
    BfxAdditionalProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

impl AuthOAuthProviderService {
    /// `/.well-known/openid-configuration` endpoint
    ///
    /// # Errors
    ///
    /// - Miscellaneous internal errors
    ///
    /// # Panics
    ///
    /// If `FRONTEND_ROOT` is not a valid URL.
    pub fn get_openid_configuration(
        &self,
        _request: Request<GetOpenidConfigurationRequest>,
    ) -> Result<Response<GetOpenidConfigurationReply>, Status> {
        let provider_metadata = BfxProviderMetadata::new(
            self.issuer.clone(),
            AuthUrl::new(format!("{}/openid/authorize", self.frontend_root)).unwrap(),
            JsonWebKeySetUrl::new(format!("{}/openid/jwks", self.frontend_root)).unwrap(),
            vec![ResponseTypes::new(vec![CoreResponseType::Code])],
            vec![CoreSubjectIdentifierType::Public],
            vec![CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256],
            BfxAdditionalProviderMetadata {
                code_challenge_methods_supported: vec![PkceCodeChallengeMethod::new(
                    "S256".to_string(),
                )],
            },
        )
        .set_token_endpoint(Some(
            TokenUrl::new(format!("{}/openid/token", self.frontend_root)).unwrap(),
        ))
        .set_userinfo_endpoint(Some(
            UserInfoUrl::new(format!("{}/openid/userinfo", self.frontend_root)).unwrap(),
        ))
        .set_scopes_supported(Some(
            vec!["openid", "email", "profile", "offline_access"]
                .into_iter()
                .map(|str| Scope::new(str.to_string()))
                .collect(),
        ))
        .set_claims_supported(Some(
            vec![
                "sub",
                "aud",
                "email",
                "email_verified",
                "exp",
                "iat",
                "iss",
                "name",
                "preferred_username",
                "profile",
                "picture",
            ]
            .into_iter()
            .map(|str| CoreClaimName::new(str.to_string()))
            .collect(),
        ))
        .set_token_endpoint_auth_methods_supported(Some(vec![
            CoreClientAuthMethod::ClientSecretBasic,
            CoreClientAuthMethod::ClientSecretPost,
        ]));

        Ok(Response::new(GetOpenidConfigurationReply {
            json: serde_json::to_string(&provider_metadata).map_err(|err| {
                Status::coded(Code::Internal, ErrorCode::Internal).with_source(err)
            })?,
        }))
    }
}
