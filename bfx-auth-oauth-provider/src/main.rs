mod methods;
pub mod models;

use bfx_core::logging::setup_logging;
use bfx_core::service::client::require_router;
use bfx_core::service::database::{Db, require_db};
use bfx_core::service::environment::{require_env, require_env_file};
use bfx_core::service::id_encryption::{IdEncryptor, require_id_encryptor};
use bfx_core::service::start_service;
use bfx_proto::auth::auth_o_auth_provider_server::{AuthOAuthProvider, AuthOAuthProviderServer};
use bfx_proto::auth::{
    AcceptAuthorizationReply, AcceptAuthorizationRequest, GetAccessTokenReply,
    GetAccessTokenRequest, GetAuthorizationInfoReply, GetAuthorizationInfoRequest, GetJwkSetReply,
    GetJwkSetRequest, GetOpenidConfigurationReply, GetOpenidConfigurationRequest,
    TokenEndpointReply, TokenEndpointRequest, UserinfoEndpointReply, UserinfoEndpointRequest,
};
use openidconnect::core::CoreRsaPrivateSigningKey;
use openidconnect::{IssuerUrl, JsonWebKeyId};
use std::sync::Arc;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let service = AuthOAuthProviderService {
        db: require_db().await?,
        router: require_router()?,
        id_encryptor: require_id_encryptor()?,
        frontend_root: require_env("FRONTEND_ROOT")?,
        issuer: IssuerUrl::new(require_env("OPENID_ISSUER")?)?,
        rs_256_signing_key: Arc::new(
            CoreRsaPrivateSigningKey::from_pem(
                &String::from_utf8_lossy(&require_env_file("OPENID_SIGNING_KEY_PATH")?),
                Some(JsonWebKeyId::new(require_env("OPENID_SIGNING_KEY_ID")?)),
            )
            .map_err(|err| anyhow::anyhow!("failed to parse openid signing key: {err}"))?,
        ),
    };

    start_service(AuthOAuthProviderServer::new(service)).await?;

    Ok(())
}

pub struct AuthOAuthProviderService {
    db: Db,
    router: Channel,

    id_encryptor: IdEncryptor,
    frontend_root: String,
    issuer: IssuerUrl,
    rs_256_signing_key: Arc<CoreRsaPrivateSigningKey>,
}

#[tonic::async_trait]
impl AuthOAuthProvider for AuthOAuthProviderService {
    async fn get_openid_configuration(
        &self,
        request: Request<GetOpenidConfigurationRequest>,
    ) -> Result<Response<GetOpenidConfigurationReply>, Status> {
        self.get_openid_configuration(request)
    }

    async fn get_jwk_set(
        &self,
        request: Request<GetJwkSetRequest>,
    ) -> Result<Response<GetJwkSetReply>, Status> {
        self.get_jwk_set(request)
    }

    async fn get_authorization_info(
        &self,
        request: Request<GetAuthorizationInfoRequest>,
    ) -> Result<Response<GetAuthorizationInfoReply>, Status> {
        self.get_authorization_info(request).await
    }

    async fn accept_authorization(
        &self,
        request: Request<AcceptAuthorizationRequest>,
    ) -> Result<Response<AcceptAuthorizationReply>, Status> {
        self.accept_authorization(request).await
    }

    async fn token_endpoint(
        &self,
        request: Request<TokenEndpointRequest>,
    ) -> Result<Response<TokenEndpointReply>, Status> {
        self.token_endpoint(request).await
    }

    async fn get_access_token(
        &self,
        request: Request<GetAccessTokenRequest>,
    ) -> Result<Response<GetAccessTokenReply>, Status> {
        self.get_access_token_rpc(request).await
    }

    async fn userinfo_endpoint(
        &self,
        request: Request<UserinfoEndpointRequest>,
    ) -> Result<Response<UserinfoEndpointReply>, Status> {
        self.userinfo_endpoint(request).await
    }
}
