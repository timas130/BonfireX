pub mod auth_sources;
mod bind_oauth;
pub mod finish_oauth_flow;
mod start_oauth_flow;
mod unbind_auth_source;

use crate::services::auth_oauth::bind_oauth::BindOAuthMutation;
use crate::services::auth_oauth::finish_oauth_flow::FinishOAuthFlowMutation;
use crate::services::auth_oauth::start_oauth_flow::StartOAuthFlowMutation;
use crate::services::auth_oauth::unbind_auth_source::UnbindAuthSourceMutation;
use async_graphql::MergedObject;

#[derive(Default, MergedObject)]
pub struct AuthOAuthQuery;

#[derive(Default, MergedObject)]
pub struct AuthOAuthMutation(
    StartOAuthFlowMutation,
    FinishOAuthFlowMutation,
    BindOAuthMutation,
    UnbindAuthSourceMutation,
);
