use crate::context::ServiceFactory;
use crate::error::RespError;
use async_graphql::{Context, Object, SimpleObject};
use bfx_proto::auth::auth_o_auth_client::AuthOAuthClient;
use bfx_proto::auth::{StartOAuthFlowReply, StartOAuthFlowRequest};
use o2o::o2o;

#[derive(Default)]
pub struct StartOAuthFlowMutation;

/// Information to authenticate using an external OAuth provider
#[derive(SimpleObject, o2o)]
#[from_owned(StartOAuthFlowReply)]
struct StartOAuthFlowResponse {
    /// Space-separated requested scopes
    scope: String,
    /// `state` parameter
    state: String,
    /// `nonce` parameter
    nonce: String,
    /// `code_challenge` parameter if PKCE is supported by issuer
    code_challenge: Option<String>,
    /// `code_challenge_method` parameter if PKCE is supported by issuer
    code_challenge_method: Option<String>,
    /// Authorization URL for browsers to redirect to
    url: String,
}

#[Object]
impl StartOAuthFlowMutation {
    /// Initiate an OAuth authentication flow
    async fn start_oauth_flow(
        &self,
        ctx: &Context<'_>,
        issuer: String,
    ) -> Result<StartOAuthFlowResponse, RespError> {
        let mut auth_oauth: AuthOAuthClient<_> = ctx.service();

        Ok(auth_oauth
            .start_oauth_flow(StartOAuthFlowRequest { issuer })
            .await?
            .into_inner()
            .into())
    }
}
