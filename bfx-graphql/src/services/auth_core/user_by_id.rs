use crate::error::RespError;
use crate::id_encryption::IdEncryptor;
use crate::models::user::GUser;
use async_graphql::{Context, ID, Object};
use bfx_core::service::id_encryption::IdType;

#[derive(Default)]
pub struct UserByIdQuery;

#[Object]
impl UserByIdQuery {
    /// Get a user by their ID
    #[graphql(cache_control(max_age = 3600))]
    async fn user_by_id(&self, ctx: &Context<'_>, id: ID) -> Result<Option<GUser>, RespError> {
        let id = ctx.decrypt_id(IdType::User, &id)?;
        GUser::from_id(ctx, id).await
    }
}
