use crate::context::ServiceFactory;
use crate::error::RespError;
use async_graphql::{Context, Object, SimpleObject};
use bfx_proto::auth::password_recovery_client::PasswordRecoveryClient;
use bfx_proto::auth::{CheckPasswordResetReply, CheckPasswordResetTokenRequest};
use o2o::o2o;

#[derive(Default)]
pub struct CheckPasswordResetTokenQuery;

#[derive(SimpleObject, o2o)]
#[from_owned(CheckPasswordResetReply)]
pub struct CheckPasswordResetTokenResponse {
    // note that this might be null even for valid tokens
    // see [`CheckPasswordResetReply`]
    username: Option<String>,
}

#[Object]
impl CheckPasswordResetTokenQuery {
    /// Get information about a password reset token
    #[graphql(cache_control(private))]
    async fn check_password_reset_token(
        &self,
        ctx: &Context<'_>,
        token: String,
    ) -> Result<CheckPasswordResetTokenResponse, RespError> {
        let mut password_recovery: PasswordRecoveryClient<_> = ctx.service();

        let resp = password_recovery
            .check_password_reset_token(CheckPasswordResetTokenRequest { token })
            .await?
            .into_inner();

        Ok(resp.into())
    }
}
