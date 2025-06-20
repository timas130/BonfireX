use crate::context::{GlobalContext, ServiceFactory};
use crate::services::auth_oauth_provider::bearer_authorization::{
    OAuthBearerToken, OAuthBearerTokenForm,
};
use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bfx_proto::auth::UserinfoEndpointRequest;
use bfx_proto::auth::auth_o_auth_provider_client::AuthOAuthProviderClient;
use tracing::error;

/// GET /openid/userinfo
///
/// # Errors
///
/// - If the underlying RPC call fails.
///   See [`AuthOAuthProviderClient::userinfo_endpoint`]
pub async fn userinfo_endpoint_get(
    Extension(context): Extension<GlobalContext>,
    OAuthBearerToken(access_token): OAuthBearerToken,
) -> Result<impl IntoResponse, StatusCode> {
    let mut oauth_provider: AuthOAuthProviderClient<_> = context.service();

    let resp = oauth_provider
        .userinfo_endpoint(UserinfoEndpointRequest { access_token })
        .await
        .map_err(|err| {
            error!(?err, "failed to get userinfo");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner();

    #[allow(clippy::cast_possible_truncation)]
    Ok((
        StatusCode::from_u16(resp.status as u16).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        [
            ("content-type", "application/json"),
            ("cache-control", "no-store"),
        ],
        resp.json,
    ))
}

/// POST /openid/userinfo
///
/// # Errors
///
/// - If the underlying RPC call fails.
///   See [`AuthOAuthProviderClient::userinfo_endpoint`]
pub async fn userinfo_endpoint_post(
    Extension(context): Extension<GlobalContext>,
    access_token: OAuthBearerTokenForm,
) -> Result<impl IntoResponse, StatusCode> {
    userinfo_endpoint_get(Extension(context), access_token.into()).await
}
