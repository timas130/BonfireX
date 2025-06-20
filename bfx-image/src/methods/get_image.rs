use crate::ImageService;
use crate::models::image::RawImage;
use bfx_core::status::StatusExt;
use bfx_proto::image::{GetImageRequest, GetImageResponse};
use tonic::{Request, Response, Status};

impl ImageService {
    pub async fn get_image(
        &self,
        request: Request<GetImageRequest>,
    ) -> Result<Response<GetImageResponse>, Status> {
        let request = request.into_inner();

        let raw_image = sqlx::query_as!(
            RawImage,
            "select *
             from image.images
             where id = $1",
            request.image_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?;

        let Some(raw_image) = raw_image else {
            return Ok(Response::new(GetImageResponse { image: None }));
        };
        let image = self.raw_to_image(raw_image).await?;

        Ok(Response::new(GetImageResponse { image: Some(image) }))
    }
}
