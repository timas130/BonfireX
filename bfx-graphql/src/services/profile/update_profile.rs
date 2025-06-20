use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::services::profile::user_profile::GProfile;
use async_graphql::{Context, InputObject, Object, SimpleObject};
use bfx_proto::profile::UpdateProfileRequest;
use bfx_proto::profile::profile_client::ProfileClient;

#[derive(Default)]
pub struct UpdateProfileMutation;

/// New profile details for updating
#[derive(InputObject)]
struct UpdateProfileInput {
    /// New profile description
    bio: Option<String>,
    /// New display name
    display_name: Option<String>,
    /// Ticket for the new avatar image
    avatar_ticket: Option<String>,
    /// Ticket for the new profile cover image
    cover_ticket: Option<String>,
}

/// Response from `update_profile`
#[derive(SimpleObject)]
struct UpdateProfileResponse {
    /// The updated profile
    profile: GProfile,
    /// Error codes if some part of the update failed
    errors: Vec<String>,
}

#[Object]
impl UpdateProfileMutation {
    /// Update any profile information except for the username and notes
    async fn update_profile(
        &self,
        ctx: &Context<'_>,
        input: UpdateProfileInput,
    ) -> Result<UpdateProfileResponse, RespError> {
        let mut profile: ProfileClient<_> = ctx.service();

        let user = ctx.require_user()?;

        let resp = profile
            .update_profile(UpdateProfileRequest {
                user_id: user.id,
                for_user_id: Some(user.id),
                bio: input.bio,
                display_name: input.display_name,
                avatar_ticket: input.avatar_ticket,
                cover_ticket: input.cover_ticket,
            })
            .await?
            .into_inner();

        Ok(UpdateProfileResponse {
            profile: resp.profile.ok_or_else(RespError::missing_field)?.into(),
            errors: resp.errors,
        })
    }
}
