use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::services::auth_oauth::auth_sources::GAuthSource;
use crate::services::auth_oauth::finish_oauth_flow::FinishOAuthFlowInput;
use async_graphql::{Context, Object};
use bfx_proto::auth::auth_o_auth_client::AuthOAuthClient;
use bfx_proto::auth::{BindOAuthRequest, FinishOAuthFlowRequest};

#[derive(Default)]
pub struct BindOAuthMutation;

#[Object]
impl BindOAuthMutation {
    /// Bind an external OAuth account to the current user
    /// (by finishing an OAuth flow as an authenticated user)
    async fn bind_oauth(
        &self,
        ctx: &Context<'_>,
        input: FinishOAuthFlowInput,
    ) -> Result<GAuthSource, RespError> {
        let mut auth_oauth: AuthOAuthClient<_> = ctx.service();

        let user = ctx.require_user()?;

        auth_oauth
            .bind_oauth(BindOAuthRequest {
                user_id: user.id,
                finish_request: Some(FinishOAuthFlowRequest {
                    user_context: Some(ctx.user_context().clone()),
                    issuer: input.issuer,
                    state: input.state,
                    code: input.code,
                }),
            })
            .await?
            .into_inner()
            .auth_source
            .ok_or_else(RespError::missing_field)?
            .try_into()
    }
}
