use crate::services::auth_oauth_provider::accept_authorization::AcceptAuthorizationMutation;
use crate::services::auth_oauth_provider::get_authorization_info::GetAuthorizationInfoQuery;
use async_graphql::MergedObject;

mod accept_authorization;
mod bearer_authorization;
mod get_authorization_info;
pub mod get_jwk_set;
pub mod get_openid_metadata;
pub mod token_endpoint;
pub mod userinfo_endpoint;

#[derive(Default, MergedObject)]
pub struct AuthOAuthProviderQuery(GetAuthorizationInfoQuery);

#[derive(Default, MergedObject)]
pub struct AuthOAuthProviderMutation(AcceptAuthorizationMutation);
