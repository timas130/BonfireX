use crate::context::ContextExt;
use crate::error::RespError;
use crate::models::user::GUser;
use crate::services::markdown::Markdown;
use crate::services::profile::data_loaders::ProfileLoader;
use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, SimpleObject};
use bfx_graphql_derive::complex_object_ext;
use bfx_proto::profile::ProfileDetails;
use o2o::o2o;

#[derive(Clone, SimpleObject, o2o)]
#[graphql(complex, name = "Profile")]
#[from_owned(ProfileDetails)]
pub struct GProfile {
    #[graphql(skip)]
    pub user_id: i64,
    /// Username of this user (for mentions)
    ///
    /// Matches `[a-zA-Z0-9][a-zA-Z0-9_]{1,23}[a-zA-Z0-9]`
    pub username: String,
    /// The display name of this user
    ///
    /// Use the username if none
    pub display_name: Option<String>,
    /// Description of this user
    #[from(Markdown::new(~))]
    pub bio: Markdown,
    /// Optional note left by the current user for this profile
    ///
    /// This is null only if the user isn't logged in
    pub note: Option<String>,
    #[graphql(skip)]
    pub avatar: Option<i64>,
    #[graphql(skip)]
    pub cover: Option<i64>,
}

#[complex_object_ext]
impl GProfile {
    /// ID of the user that owns this profile
    id!(user_id => id, User);
    /// Avatar image of this profile
    optional_image!(avatar);
    /// Cover image of this profile
    optional_image!(cover);
}

impl GUser {
    pub(crate) async fn _profile(&self, ctx: &Context<'_>) -> Result<Option<GProfile>, RespError> {
        let profile_loader = ctx.data_unchecked::<DataLoader<ProfileLoader>>();
        let for_user_id = ctx.user().map(|user| user.id);

        profile_loader.load_one((self._id, for_user_id)).await
    }
}
