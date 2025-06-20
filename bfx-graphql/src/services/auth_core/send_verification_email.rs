use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::models::ok::OkResp;
use async_graphql::{Context, Object};
use bfx_proto::auth::SendVerificationEmailRequest;
use bfx_proto::auth::auth_core_client::AuthCoreClient;

#[derive(Default)]
pub struct SendVerificationEmailMutation;

#[Object]
impl SendVerificationEmailMutation {
    /// Resend the verification email to an unverified user
    async fn send_verification_email(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> Result<OkResp, RespError> {
        let mut auth_core: AuthCoreClient<_> = ctx.service();

        auth_core
            .send_verification_email(SendVerificationEmailRequest {
                email,
                user_context: Some(ctx.user_context().clone()),
            })
            .await?;

        Ok(OkResp)
    }
}
