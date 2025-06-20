use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::services::profile::user_profile::GProfile;
use async_graphql::{Context, Object};
use bfx_proto::profile::SetNoteRequest;
use bfx_proto::profile::profile_client::ProfileClient;

#[derive(Default)]
pub struct SetProfileNoteMutation;

#[Object]
impl SetProfileNoteMutation {
    /// Update a personal note for a user
    async fn set_profile_note(
        &self,
        ctx: &Context<'_>,
        user_id: i64,
        note: String,
    ) -> Result<GProfile, RespError> {
        let mut profile: ProfileClient<_> = ctx.service();

        let for_user = ctx.require_user()?;

        let profile = profile
            .set_note(SetNoteRequest {
                note,
                profile_id: user_id,
                user_id: for_user.id,
            })
            .await?
            .into_inner()
            .profile
            .ok_or_else(RespError::missing_field)?;

        Ok(GProfile::from(profile))
    }
}
