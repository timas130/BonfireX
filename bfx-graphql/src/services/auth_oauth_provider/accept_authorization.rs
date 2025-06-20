use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::id_encryption::IdEncryptor;
use async_graphql::{Context, ID, Object, SimpleObject};
use bfx_core::service::id_encryption::IdType;
use bfx_proto::auth::auth_o_auth_provider_client::AuthOAuthProviderClient;
use bfx_proto::auth::{AcceptAuthorizationReply, AcceptAuthorizationRequest};
use o2o::o2o;

#[derive(SimpleObject, o2o)]
#[from_owned(AcceptAuthorizationReply)]
pub struct AcceptAuthorizationResponse {
    redirect_to: String,
}

#[derive(Default)]
pub struct AcceptAuthorizationMutation;

#[Object]
impl AcceptAuthorizationMutation {
    async fn accept_authorization(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<AcceptAuthorizationResponse, RespError> {
        let mut auth_oauth_provider: AuthOAuthProviderClient<_> = ctx.service();

        let user = ctx.require_user()?;
        let flow_id = ctx.decrypt_id(IdType::OAuthFlow, &id)?;

        let resp = auth_oauth_provider
            .accept_authorization(AcceptAuthorizationRequest {
                flow_id,
                user_id: user.id,
            })
            .await?
            .into_inner();

        Ok(resp.into())
    }
}
