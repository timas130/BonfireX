use crate::context::ContextExt;
use crate::models::user::GUser;
use async_graphql::{Context, Object};

#[derive(Default)]
pub struct MeQuery;

#[Object]
impl MeQuery {
    /// Get the currently logged-in user
    #[graphql(cache_control(max_age = 3600, private))]
    async fn me(&self, ctx: &Context<'_>) -> Option<GUser> {
        ctx.user().cloned().map(From::from)
    }
}
