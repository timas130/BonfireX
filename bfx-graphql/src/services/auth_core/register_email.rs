use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::services::auth_core::login_email::GLoginResultTokens;
use async_graphql::{Context, Object};
use bfx_proto::auth::auth_core_client::AuthCoreClient;
use bfx_proto::auth::{CreateUserRequest, SendVerificationEmailRequest};
use bfx_proto::notification::notification_client::NotificationClient;
use bfx_proto::notification::{NotificationPreferences, SetNotificationPreferencesRequest};

#[derive(Default)]
pub struct RegisterEmailMutation;

#[Object]
impl RegisterEmailMutation {
    /// Register a new user with an email and password
    ///
    /// This also sends a verification email to the user.
    /// The email must be verified before the user can use the returned tokens.
    async fn register_email(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<GLoginResultTokens, RespError> {
        let mut auth_core: AuthCoreClient<_> = ctx.service();
        let mut notification: NotificationClient<_> = ctx.service();

        let create_reply = auth_core
            .create_user(CreateUserRequest {
                email: Some(email.clone()),
                active: false,
                password: Some(password),
                user_context: Some(ctx.user_context().clone()),
            })
            .await?
            .into_inner();

        let user = create_reply.user.ok_or_else(RespError::missing_field)?;
        notification
            .set_notification_preferences(SetNotificationPreferencesRequest {
                user_id: user.id,
                preferences: Some(NotificationPreferences {
                    lang_id: ctx.user_context().lang_id.clone(),
                }),
            })
            .await?;

        auth_core
            .send_verification_email(SendVerificationEmailRequest {
                email,
                user_context: Some(ctx.user_context().clone()),
            })
            .await?;

        Ok(create_reply
            .tokens
            .ok_or_else(RespError::missing_field)?
            .into())
    }
}
