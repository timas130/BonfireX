use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::id_encryption::IdEncryptor;
use async_graphql::{Context, ID, Object};
use bfx_core::service::id_encryption::IdType;
use bfx_proto::auth::UnbindAuthSourceRequest;
use bfx_proto::auth::auth_o_auth_client::AuthOAuthClient;

#[derive(Default)]
pub struct UnbindAuthSourceMutation;

#[Object]
impl UnbindAuthSourceMutation {
    /// Unbind an external OAuth account from the current user
    ///
    /// Returns the ID of the removed auth source
    async fn unbind_auth_source(&self, ctx: &Context<'_>, id: ID) -> Result<ID, RespError> {
        let mut auth_oauth: AuthOAuthClient<_> = ctx.service();

        let user = ctx.require_user()?;
        let auth_source_id = ctx.decrypt_id(IdType::AuthSource, &id)?;

        auth_oauth
            .unbind_auth_source(UnbindAuthSourceRequest {
                auth_source_id,
                user_id: user.id,
                user_context: Some(ctx.user_context().clone()),
            })
            .await?;

        Ok(id)
    }
}
