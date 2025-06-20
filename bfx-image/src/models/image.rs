use crate::ImageService;
use bfx_core::status::StatusExt;
use bfx_proto::image::{ImageDesc, ImageVariant};
use sqlx::types::chrono::{DateTime, Utc};
use tonic::Status;

pub struct RawImage {
    pub id: i64,
    pub full_width: i32,
    pub full_height: i32,
    pub full_size: i32,
    pub thumbnail_width: i32,
    pub thumbnail_height: i32,
    pub thumbnail_size: i32,
    pub blur_data: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

impl ImageService {
    pub async fn raw_to_image(&self, raw: RawImage) -> Result<ImageDesc, Status> {
        let full = self
            .presign_get(
                format!("images/{}_full.jxl", raw.id),
                raw.full_width,
                raw.full_height,
                raw.full_size,
            )
            .await?;
        let thumbnail = self
            .presign_get(
                format!("images/{}_thumbnail.jxl", raw.id),
                raw.thumbnail_width,
                raw.thumbnail_height,
                raw.thumbnail_size,
            )
            .await?;

        Ok(ImageDesc {
            id: raw.id,
            full: Some(full),
            thumbnail: Some(thumbnail),
            blur_data: raw.blur_data,
            created_at: Some(raw.created_at.into()),
        })
    }

    #[allow(clippy::cast_sign_loss)]
    async fn presign_get(
        &self,
        path: String,
        w: i32,
        h: i32,
        size: i32,
    ) -> Result<ImageVariant, Status> {
        let url = self
            .public_bucket
            .presign_get(path, 3600 * 24, None)
            .await
            .map_err(|err| Status::anyhow(err.into()))?;

        Ok(ImageVariant {
            width: w as u32,
            height: h as u32,
            size: size as u64,
            url,
        })
    }
}
