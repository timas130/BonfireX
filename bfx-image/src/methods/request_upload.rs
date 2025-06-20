use crate::ImageService;
use bfx_core::status::StatusExt;
use bfx_proto::image::{RequestUploadRequest, RequestUploadResponse};
use nanoid::nanoid;
use tonic::{Request, Response, Status};

impl ImageService {
    pub async fn request_upload(
        &self,
        request: Request<RequestUploadRequest>,
    ) -> Result<Response<RequestUploadResponse>, Status> {
        let request = request.into_inner();

        let ticket = nanoid!();

        sqlx::query!(
            "insert into image.image_tickets
             (ticket, user_id, created_at)
             values ($1, $2, now())",
            ticket,
            request.user_id,
        )
        .execute(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(Response::new(RequestUploadResponse { ticket }))
    }
}
