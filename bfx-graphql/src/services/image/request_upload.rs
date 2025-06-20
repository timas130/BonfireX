use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use async_graphql::{Context, Object};
use bfx_proto::image::RequestUploadRequest;
use bfx_proto::image::image_client::ImageClient;

#[derive(Default)]
pub struct RequestUploadMutation;

#[Object]
impl RequestUploadMutation {
    /// Request an upload ticket to upload a resource
    async fn request_upload(&self, ctx: &Context<'_>) -> Result<String, RespError> {
        let mut image: ImageClient<_> = ctx.service();

        let user = ctx.require_user()?;

        let ticket = image
            .request_upload(RequestUploadRequest { user_id: user.id })
            .await?
            .into_inner()
            .ticket;

        Ok(ticket)
    }
}
