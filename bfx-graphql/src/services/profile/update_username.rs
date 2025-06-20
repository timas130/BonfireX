use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::services::profile::user_profile::GProfile;
use async_graphql::{Context, Object};
use bfx_proto::profile::UpdateUsernameRequest;
use bfx_proto::profile::profile_client::ProfileClient;

#[derive(Default)]
pub struct UpdateUsernameMutation;

#[Object]
impl UpdateUsernameMutation {
    /// Change the username of the current user (and create a profile if needed)
    async fn update_username(
        &self,
        ctx: &Context<'_>,
        new_username: String,
    ) -> Result<GProfile, RespError> {
        let mut profile: ProfileClient<_> = ctx.service();

        let user = ctx.require_user()?;

        let profile = profile
            .update_username(UpdateUsernameRequest {
                user_id: user.id,
                username: new_username,
                for_user_id: Some(user.id),
            })
            .await?
            .into_inner()
            .profile
            .ok_or_else(RespError::missing_field)?;

        Ok(GProfile::from(profile))
    }
}
