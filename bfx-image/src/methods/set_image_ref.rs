use crate::ImageService;
use bfx_core::status::StatusExt;
use bfx_proto::image::{SetImageRefRequest, SetImageRefResponse};
use tonic::{Request, Response, Status};

impl ImageService {
    pub async fn set_image_ref(
        &self,
        request: Request<SetImageRefRequest>,
    ) -> Result<Response<SetImageRefResponse>, Status> {
        let request = request.into_inner();

        if request.exists {
            self.add_image_ref(request.image_id, request.ref_id).await?;
        } else {
            self.remove_image_ref(request.image_id, request.ref_id)
                .await?;
        }

        Ok(Response::new(SetImageRefResponse {}))
    }

    pub async fn add_image_ref(&self, image_id: i64, ref_id: String) -> Result<(), Status> {
        sqlx::query!(
            "insert into image.image_refs
             (image_id, ref_id, created_at)
             values ($1, $2, now())
             on conflict (image_id, ref_id) do nothing",
            image_id,
            ref_id,
        )
        .execute(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(())
    }

    async fn remove_image_ref(&self, image_id: i64, ref_id: String) -> Result<(), Status> {
        sqlx::query!(
            "delete from image.image_refs
             where image_id = $1 and ref_id = $2",
            image_id,
            ref_id,
        )
        .execute(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(())
    }
}
