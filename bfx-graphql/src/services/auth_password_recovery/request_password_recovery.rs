use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::models::ok::OkResp;
use async_graphql::{Context, Object};
use bfx_proto::auth::RequestPasswordRecoveryRequest;
use bfx_proto::auth::password_recovery_client::PasswordRecoveryClient;

#[derive(Default)]
pub struct RequestPasswordRecoveryMutation;

#[Object]
impl RequestPasswordRecoveryMutation {
    async fn request_password_recovery(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> Result<OkResp, RespError> {
        let mut password_recovery: PasswordRecoveryClient<_> = ctx.service();

        password_recovery
            .request_password_recovery(RequestPasswordRecoveryRequest {
                user_context: Some(ctx.user_context().clone()),
                email,
            })
            .await?;

        Ok(OkResp)
    }
}
