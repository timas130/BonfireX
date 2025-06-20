use crate::ext_impl;
use crate::image::image_client::ImageClient;
use crate::image::{GetImageRequest, ImageDesc, SetImageRefRequest, UseImageTicketRequest};
use tonic::Status;

ext_impl!(ImageClient, {
    pub async fn get_image_ext(
        &mut self,
        image_id: Option<i64>,
        ref_id: String,
    ) -> Result<Option<ImageDesc>, Status> {
        let Some(image_id) = image_id else {
            return Ok(None);
        };

        self.get_image(GetImageRequest {
            image_id,
            ref_id: Some(ref_id),
        })
        .await
        .map(|resp| resp.into_inner().image)
    }

    pub async fn set_image_ref_ext(
        &mut self,
        image_id: Option<i64>,
        ref_id: String,
        exists: bool,
    ) -> Result<(), Status> {
        let Some(image_id) = image_id else {
            return Ok(());
        };

        self.set_image_ref(SetImageRefRequest {
            image_id,
            ref_id,
            exists,
        })
        .await?;

        Ok(())
    }

    pub async fn use_image_ticket_ext(
        &mut self,
        ticket: String,
        user_id: i64,
        ref_id: String,
    ) -> Result<ImageDesc, Status> {
        self.use_image_ticket(UseImageTicketRequest {
            ticket,
            user_id,
            ref_id: Some(ref_id),
        })
        .await
        .and_then(|resp| {
            resp.into_inner()
                .image
                .ok_or_else(|| Status::unknown("use_image_ticket response image is none"))
        })
    }
});
