mod permission_level;

use crate::context::ContextExt;
use crate::error::RespError;
use crate::models::user::permission_level::GPermissionLevel;
use crate::services::auth_core::data_loaders::UserLoader;
use crate::services::auth_oauth::auth_sources::GAuthSource;
use crate::services::profile::user_profile::GProfile;
use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, SimpleObject};
use bfx_graphql_derive::complex_object_ext;
use bfx_proto::auth::User;
use o2o::o2o;

/// A user
#[derive(Clone, SimpleObject, o2o)]
#[graphql(complex, name = "User")]
#[from_owned(User)]
pub struct GUser {
    #[graphql(skip)]
    #[map(id)]
    pub _id: i64,
    #[graphql(skip)]
    #[map(email)]
    pub _email: Option<String>,
    /// Global permission level
    #[from(~.into())]
    #[graphql(cache_control(max_age = 3600))]
    pub permission_level: GPermissionLevel,
    /// Whether the user is permanently and globally banned
    #[graphql(cache_control(max_age = 600))]
    pub banned: bool,
}

impl GUser {
    /// Get a user by their ID
    ///
    /// # Errors
    ///
    /// - Miscellaneous internal errors
    pub async fn from_id(ctx: &Context<'_>, id: i64) -> Result<Option<Self>, RespError> {
        let loader = ctx.data_unchecked::<DataLoader<UserLoader>>();
        loader.load_one(id).await.map(|opt| opt.map(From::from))
    }
}

#[allow(clippy::unused_async, clippy::used_underscore_items)]
#[complex_object_ext]
impl GUser {
    /// Unique ID of the user
    #[graphql(cache_control(max_age = 86400))]
    id!(_id => id, User);

    /// The user's email address (only visible to admins)
    #[graphql(cache_control(max_age = 60, private))]
    async fn email(&self, ctx: &Context<'_>) -> Result<Option<&str>, async_graphql::Error> {
        ctx.require_self_or_admin(self._id)?;

        Ok(self._email.as_deref())
    }

    /// The user's profile
    #[graphql(cache_control(max_age = 60))]
    async fn profile(&self, ctx: &Context<'_>) -> Result<Option<GProfile>, RespError> {
        self._profile(ctx).await
    }

    /// External accounts bound to this user
    #[graphql(cache_control(max_age = 60, private))]
    async fn auth_sources(&self, ctx: &Context<'_>) -> Result<Vec<GAuthSource>, RespError> {
        self._auth_sources(ctx).await
    }
}
