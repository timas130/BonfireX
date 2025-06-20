use crate::context::{GlobalContext, ServiceFactory};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Basic;
use bfx_proto::auth::auth_o_auth_provider_client::AuthOAuthProviderClient;
use bfx_proto::auth::{BasicAuthorization, TokenEndpointRequest};
use std::collections::HashMap;
use tracing::error;

/// POST /openid/token
///
/// # Errors
///
/// - If the underlying RPC call fails.
///   See [`AuthOAuthProviderClient::token_endpoint`]
#[allow(clippy::implicit_hasher)]
pub async fn token_endpoint(
    Extension(context): Extension<GlobalContext>,
    authorization: Option<TypedHeader<Authorization<Basic>>>,
    Form(params): Form<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut oauth_provider: AuthOAuthProviderClient<_> = context.service();

    let resp = oauth_provider
        .token_endpoint(TokenEndpointRequest {
            query: params,
            authorization: authorization.map(|auth| BasicAuthorization {
                username: auth.username().to_string(),
                password: auth.password().to_string(),
            }),
        })
        .await
        .map_err(|err| {
            error!(?err, "token endpoint returned error");
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
