use crate::context::ServiceFactory;
use crate::data_loader;
use crate::error::RespError;
use crate::services::image::image::GImage;
use async_graphql::dataloader::Loader;
use bfx_proto::image::image_client::ImageClient;
use bfx_proto::image::{GetImageBulkRequest, GetImageRequest};
use std::collections::HashMap;

data_loader!(ImageLoader);

impl Loader<i64> for ImageLoader {
    type Value = GImage;
    type Error = RespError;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let mut image: ImageClient<_> = self.ctx.service();

        Ok(image
            .get_image_bulk(GetImageBulkRequest {
                requests: keys
                    .iter()
                    .map(|&id| GetImageRequest {
                        image_id: id,
                        ref_id: None,
                    })
                    .collect(),
            })
            .await?
            .into_inner()
            .responses
            .into_iter()
            .filter_map(|resp| resp.image.map(From::from))
            .map(|image: GImage| (image.inner.id, image))
            .collect())
    }
}
