use crate::ImageService;
use crate::models::image::RawImage;
use bfx_core::status::StatusExt;
use bfx_proto::image::{GetImageBulkRequest, GetImageBulkResponse, GetImageResponse};
use futures_util::TryStreamExt;
use tonic::{Request, Response, Status};

impl ImageService {
    pub async fn get_image_bulk(
        &self,
        request: Request<GetImageBulkRequest>,
    ) -> Result<Response<GetImageBulkResponse>, Status> {
        let request = request.into_inner();

        let resp = sqlx::query_as!(
            RawImage,
            "select *
             from image.images
             where id = any($1)",
            request
                .requests
                .iter()
                .map(|r| r.image_id)
                .collect::<Vec<_>>() as Vec<i64>,
        )
        .fetch(&self.db)
        .map_err(Status::db)
        .and_then(|raw| self.raw_to_image(raw))
        .map_ok(|image| GetImageResponse { image: Some(image) })
        .try_collect::<Vec<_>>()
        .await?;

        Ok(Response::new(GetImageBulkResponse { responses: resp }))
    }
}
