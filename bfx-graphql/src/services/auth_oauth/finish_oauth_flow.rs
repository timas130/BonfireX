use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::services::auth_core::login_email::GLoginResultTokens;
use async_graphql::{Context, InputObject, Object, SimpleObject};
use bfx_proto::auth::auth_o_auth_client::AuthOAuthClient;
use bfx_proto::auth::{FinishOAuthFlowReply, FinishOAuthFlowRequest};
use o2o::o2o;

#[derive(Default)]
pub struct FinishOAuthFlowMutation;

/// Information received from the OAuth provider required to complete authentication
#[derive(InputObject)]
pub struct FinishOAuthFlowInput {
    /// Issuer URL
    ///
    /// For example, `https://accounts.google.com`.
    /// Must be the same as in `start_oauth_flow`.
    pub issuer: String,
    /// `state` parameter
    ///
    /// Must be the same as the one returned from `start_oauth_flow`
    pub state: String,
    /// Authorization code returned by the OAuth provider
    pub code: String,
}

/// The result of authenticating with an external OAuth provider
#[derive(SimpleObject, o2o)]
#[from_owned(FinishOAuthFlowReply)]
struct FinishOAuthFlowResponse {
    /// Resulting tokens
    #[from(~.map(From::from))]
    tokens: Option<GLoginResultTokens>,
    /// Whether a user is already registered to the email of the external user
    existing_email: bool,
}

#[Object]
impl FinishOAuthFlowMutation {
    /// Complete authenticating with external OAuth
    async fn finish_oauth_flow(
        &self,
        ctx: &Context<'_>,
        input: FinishOAuthFlowInput,
    ) -> Result<FinishOAuthFlowResponse, RespError> {
        let mut auth_oauth: AuthOAuthClient<_> = ctx.service();

        Ok(auth_oauth
            .finish_oauth_flow(FinishOAuthFlowRequest {
                issuer: input.issuer,
                code: input.code,
                state: input.state,
                user_context: Some(ctx.user_context().clone()),
            })
            .await?
            .into_inner()
            .into())
    }
}
