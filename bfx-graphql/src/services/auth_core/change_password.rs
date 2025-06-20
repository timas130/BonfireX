use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::models::ok::OkResp;
use async_graphql::{Context, Object};
use bfx_proto::auth::ChangePasswordRequest;
use bfx_proto::auth::auth_core_client::AuthCoreClient;

#[derive(Default)]
pub struct ChangePasswordMutation;

#[Object]
impl ChangePasswordMutation {
    /// Change the current user's password knowing the old password
    async fn change_password(
        &self,
        ctx: &Context<'_>,
        old_password: String,
        new_password: String,
    ) -> Result<OkResp, RespError> {
        let mut auth_core: AuthCoreClient<_> = ctx.service();

        let user = ctx.require_user()?;

        auth_core
            .change_password(ChangePasswordRequest {
                user_id: user.id,
                old_password: Some(old_password),
                new_password,
                user_context: Some(ctx.user_context().clone()),
                terminate_all_sessions: false,
            })
            .await?;

        Ok(OkResp)
    }
}
