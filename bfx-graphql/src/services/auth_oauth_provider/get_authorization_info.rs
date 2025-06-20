use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use async_graphql::{Context, Object, SimpleObject};
use bfx_graphql_derive::complex_object_ext;
use bfx_proto::auth::auth_o_auth_provider_client::AuthOAuthProviderClient;
use bfx_proto::auth::{GetAuthorizationInfoReply, GetAuthorizationInfoRequest, RpInfo};
use o2o::o2o;
use std::collections::HashMap;

#[derive(Default)]
pub struct GetAuthorizationInfoQuery;

/// Information used to show the authorization prompt screen (or to instantly redirect)
#[derive(SimpleObject, o2o)]
#[graphql(complex)]
#[try_from_owned(GetAuthorizationInfoReply, RespError)]
pub struct AuthorizationInfo {
    #[graphql(skip)]
    pub flow_id: Option<i64>,
    /// Information about the OAuth client
    #[try_from(~.ok_or_else(RespError::missing_field)?.into())]
    pub rp_info: GRpInfo,
    /// Requested scopes
    pub scopes: Vec<String>,
    /// URL to redirect to (if the user is already authorized or prompt=none)
    pub redirect_to: Option<String>,
}

#[complex_object_ext]
impl AuthorizationInfo {
    /// ID of the OAuth flow
    ///
    /// Only present if the user is authenticated
    optional_id!(flow_id => flow_id, OAuthFlow);
}

/// OAuth relying party (client) info
#[derive(SimpleObject, o2o)]
#[graphql(complex, name = "RpInfo")]
#[from_owned(RpInfo)]
pub struct GRpInfo {
    #[graphql(skip)]
    pub id: i64,
    /// Name of the service
    pub display_name: String,
    /// Privacy policy URL of the service
    pub privacy_url: Option<String>,
    /// Terms of service URL of the service
    pub tos_url: Option<String>,
    /// Whether the service is marked as official
    pub official: bool,
}

#[complex_object_ext]
impl GRpInfo {
    /// Unique ID of the OAuth client (does not equal the `client_id`)
    id!(id => id, OAuthClient);
}

#[Object]
impl GetAuthorizationInfoQuery {
    /// Get information for authorizing an OAuth client
    ///
    /// `query` is a map of the query parameters passed to the `/authorize` endpoint
    #[graphql(cache_control(private))]
    async fn get_authorization_info(
        &self,
        ctx: &Context<'_>,
        query: HashMap<String, String>,
    ) -> Result<AuthorizationInfo, RespError> {
        let mut auth_oauth_provider: AuthOAuthProviderClient<_> = ctx.service();
        let user = ctx.user();

        let resp = auth_oauth_provider
            .get_authorization_info(GetAuthorizationInfoRequest {
                user_id: user.map(|user| user.id),
                query,
            })
            .await?
            .into_inner();

        resp.try_into()
    }
}
