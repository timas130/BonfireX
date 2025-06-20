use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::models::ok::OkResp;
use async_graphql::{Context, Object};
use bfx_proto::auth::ResetPasswordRequest;
use bfx_proto::auth::password_recovery_client::PasswordRecoveryClient;

#[derive(Default)]
pub struct ResetPasswordMutation;

#[Object]
impl ResetPasswordMutation {
    async fn reset_password(
        &self,
        ctx: &Context<'_>,
        token: String,
        new_password: String,
    ) -> Result<OkResp, RespError> {
        let mut password_recovery: PasswordRecoveryClient<_> = ctx.service();

        password_recovery
            .reset_password(ResetPasswordRequest {
                token,
                new_password,
                user_context: Some(ctx.user_context().clone()),
            })
            .await?;

        Ok(OkResp)
    }
}
