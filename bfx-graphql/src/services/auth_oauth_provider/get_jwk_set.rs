use crate::context::{GlobalContext, ServiceFactory};
use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bfx_proto::auth::GetJwkSetRequest;
use bfx_proto::auth::auth_o_auth_provider_client::AuthOAuthProviderClient;
use tracing::error;

/// GET /openid/jwks
///
/// # Errors
///
/// - If the underlying RPC call fails.
///   See [`AuthOAuthProviderClient::get_jwk_set`]
pub async fn get_jwk_set(
    Extension(context): Extension<GlobalContext>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut oauth_provider: AuthOAuthProviderClient<_> = context.service();

    let jwk_set = oauth_provider
        .get_jwk_set(GetJwkSetRequest {})
        .await
        .map_err(|err| {
            error!(?err, "failed to get jwk set");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner()
        .json;

    Ok((
        [
            ("content-type", "application/json"),
            ("cache-control", "max-age=3600"),
        ],
        jwk_set,
    ))
}
