use crate::services::auth_core::{AuthCoreMutation, AuthCoreQuery};
use crate::services::auth_oauth_provider::{AuthOAuthProviderMutation, AuthOAuthProviderQuery};
use crate::services::auth_password_recovery::{
    AuthPasswordRecoveryMutation, AuthPasswordRecoveryQuery,
};
use async_graphql::{EmptySubscription, MergedObject, Schema};

pub type GSchema = Schema<GlobalQuery, GlobalMutation, EmptySubscription>;

#[derive(MergedObject, Default)]
#[graphql(name = "Query")]
pub struct GlobalQuery(
    AuthCoreQuery,
    AuthOAuthProviderQuery,
    AuthPasswordRecoveryQuery,
);

#[derive(MergedObject, Default)]
#[graphql(name = "Mutation")]
pub struct GlobalMutation(
    AuthCoreMutation,
    AuthOAuthProviderMutation,
    AuthPasswordRecoveryMutation,
);
