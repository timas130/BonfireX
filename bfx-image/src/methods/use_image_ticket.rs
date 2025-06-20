use crate::ImageService;
use crate::models::image::RawImage;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::image::{UseImageTicketRequest, UseImageTicketResponse};
use tonic::{Code, Request, Response, Status};

impl ImageService {
    pub async fn use_image_ticket(
        &self,
        request: Request<UseImageTicketRequest>,
    ) -> Result<Response<UseImageTicketResponse>, Status> {
        let request = request.into_inner();

        let mut tx = self.db.begin().await.map_err(Status::db)?;

        // look up the ticket
        let ticket = sqlx::query!(
            "select *
             from image.image_tickets
             where ticket = $1 and
                   user_id = $2 and
                   created_at > now() - interval '1 hour'
             for update",
            request.ticket,
            request.user_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(Status::db)?
        .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::TicketNotFound))?;

        // check that the ticket has been used
        if ticket.image_id.is_none() {
            tx.rollback().await.map_err(Status::db)?;
            return Err(Status::coded(Code::NotFound, ErrorCode::ImageNotUploaded));
        }

        // get the image to return it
        let image = sqlx::query_as!(
            RawImage,
            "select *
             from image.images
             where id = $1",
            ticket.image_id,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Status::db)?;

        tx.commit().await.map_err(Status::db)?;

        // add image ref if requested
        if let Some(ref_id) = request.ref_id {
            self.add_image_ref(image.id, ref_id).await?;
        }

        // return the image
        let image = self.raw_to_image(image).await?;

        Ok(Response::new(UseImageTicketResponse { image: Some(image) }))
    }
}
