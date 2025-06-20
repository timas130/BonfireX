use crate::context::ServiceFactory;
use crate::error::RespError;
use crate::models::user::GUser;
use async_graphql::{Context, Object};
use bfx_proto::auth::VerifyEmailRequest;
use bfx_proto::auth::auth_core_client::AuthCoreClient;

#[derive(Default)]
pub struct VerifyEmailMutation;

#[Object]
impl VerifyEmailMutation {
    /// Verify an email address using the token from the message
    pub async fn verify_email(&self, ctx: &Context<'_>, token: String) -> Result<GUser, RespError> {
        let mut auth_core: AuthCoreClient<_> = ctx.service();

        let verified_user = auth_core
            .verify_email(VerifyEmailRequest { token })
            .await?
            .into_inner()
            .user
            .ok_or_else(RespError::missing_field)?;

        Ok(verified_user.into())
    }
}
