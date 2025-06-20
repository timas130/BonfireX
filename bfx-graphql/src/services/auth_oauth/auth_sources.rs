use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::models::user::GUser;
use async_graphql::{Context, SimpleObject};
use bfx_graphql_derive::complex_object_ext;
use bfx_proto::auth::auth_o_auth_client::AuthOAuthClient;
use bfx_proto::auth::{AuthSource, GetAuthSourcesRequest};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use o2o::o2o;

/// An external authentication source (OAuth record)
#[derive(SimpleObject, o2o)]
#[graphql(complex, name = "AuthSource")]
#[try_from_owned(AuthSource, RespError)]
pub struct GAuthSource {
    #[graphql(skip)]
    id: i64,
    #[graphql(skip)]
    user_id: i64,
    /// Issuer URL of the external auth provider
    issuer: String,
    /// ID of the external account from the auth provider
    issuer_user_id: String,
    /// When the external account was first associated with the user
    #[try_from(~.ok_or_else(RespError::missing_field)?.try_into()?)]
    created_at: DateTime<Utc>,
}

#[complex_object_ext]
impl GAuthSource {
    /// ID of this auth source
    id!(id => id, AuthSource);

    /// User associated with this auth source
    user!(user_id => user);
}

impl GUser {
    pub async fn _auth_sources(&self, ctx: &Context<'_>) -> Result<Vec<GAuthSource>, RespError> {
        ctx.require_self_or_admin(self._id)?;

        let mut auth_oauth: AuthOAuthClient<_> = ctx.service();

        Ok(auth_oauth
            .get_auth_sources(GetAuthSourcesRequest { user_id: self._id })
            .await?
            .into_inner()
            .auth_sources
            .into_iter()
            .map(TryFrom::try_from)
            .try_collect()?)
    }
}
