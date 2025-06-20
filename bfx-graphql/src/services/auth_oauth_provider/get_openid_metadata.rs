use crate::context::{GlobalContext, ServiceFactory};
use axum::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bfx_proto::auth::GetOpenidConfigurationRequest;
use bfx_proto::auth::auth_o_auth_provider_client::AuthOAuthProviderClient;
use tracing::error;

/// GET /.well-known/openid-configuration
///
/// # Errors
///
/// - If the underlying RPC call fails.
///   See [`AuthOAuthProviderClient::get_openid_configuration`]
pub async fn get_openid_metadata(
    Extension(context): Extension<GlobalContext>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut oauth_provider: AuthOAuthProviderClient<_> = context.service();

    let openid_configuration = oauth_provider
        .get_openid_configuration(GetOpenidConfigurationRequest {})
        .await
        .map_err(|err| {
            error!(?err, "failed to get openid metadata");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .into_inner()
        .json;

    Ok((
        [
            ("content-type", "application/json"),
            ("cache-control", "max-age=3600"),
        ],
        openid_configuration,
    ))
}
