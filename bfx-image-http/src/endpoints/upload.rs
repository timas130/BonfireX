use crate::ImageHttpService;
use crate::util::image_dimensions::resize_dimensions;
use axum::Extension;
use axum::body::Bytes;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use fast_image_resize::Resizer;
use image::{DynamicImage, ImageReader, Limits};
use jpegxl_rs::encode::{EncoderResult, EncoderSpeed};
use jpegxl_rs::encoder_builder;
use serde::Deserialize;
use std::io::Cursor;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tracing::{Instrument, info, info_span, warn};

#[derive(Deserialize)]
pub struct UploadQuery {
    ticket: String,
}

struct ImageData {
    full: Vec<u8>,
    full_dim: (u32, u32),
    thumbnail: Vec<u8>,
    thumbnail_dim: (u32, u32),
    blur: Vec<u8>,
}

#[allow(clippy::cast_possible_wrap)]
pub async fn upload(
    Query(query): Query<UploadQuery>,
    Extension(service): Extension<ImageHttpService>,
    body: Bytes,
) -> Result<impl IntoResponse, StatusCode> {
    let len = body.len();

    // check if the ticket is valid
    let mut db_tx = service
        .db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ticket = sqlx::query!(
        "select * from image.image_tickets
         where ticket = $1 and
               created_at > now() - interval '1 hour'
         for update",
        query.ticket
    )
    .fetch_optional(&mut *db_tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if ticket.image_id.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    // start transcoding the image in a separate thread
    let (tx, rx) = oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(decode_image(body));
    });

    // wait for the image to be decoded or timeout
    let image_data = timeout(Duration::from_secs(15), rx)
        .instrument(info_span!("transcoding image", len))
        .await;
    let Ok(image_data) = image_data else {
        warn!(len, "transcoding image timed out");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let image_data = image_data.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    info!(
        src = len,
        full = image_data.full.len(),
        thumb = image_data.thumbnail.len(),
        blur = image_data.blur.len(),
        "transcoded image"
    );

    // get the id of the new image for insertion into s3
    #[allow(clippy::cast_possible_truncation)]
    let image = sqlx::query!(
        "insert into image.images
         (full_width, full_height, full_size, thumbnail_width, thumbnail_height, thumbnail_size, blur_data)
         values ($1, $2, $3, $4, $5, $6, $7)
         returning id",
        image_data.full_dim.0 as i32,
        image_data.full_dim.1 as i32,
        image_data.full.len() as i32,
        image_data.thumbnail_dim.0 as i32,
        image_data.thumbnail_dim.1 as i32,
        image_data.thumbnail.len() as i32,
        image_data.blur,
    )
    .fetch_one(&mut *db_tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // upload to s3
    upload_to_s3(&service, image_data, image.id).await?;

    // mark the ticket as uploaded
    sqlx::query!(
        "update image.image_tickets
         set image_id = $1, created_at = now()
         where id = $2",
        image.id,
        ticket.id,
    )
    .execute(&mut *db_tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    db_tx
        .commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok("ok")
}

async fn upload_to_s3(
    service: &ImageHttpService,
    image: ImageData,
    image_id: i64,
) -> Result<(), StatusCode> {
    let retry_policy = ExponentialBackoff::from_millis(10).map(jitter).take(3);

    let fut1 = Retry::spawn(retry_policy.clone(), || async {
        service
            .bucket
            .put_object_with_content_type(
                &format!("images/{image_id}_full.jxl"),
                &image.full,
                "image/jxl",
            )
            .await
    });
    let fut2 = Retry::spawn(retry_policy, || async {
        service
            .bucket
            .put_object_with_content_type(
                &format!("images/{image_id}_thumbnail.jxl"),
                &image.thumbnail,
                "image/jxl",
            )
            .await
    });

    tokio::try_join!(fut1, fut2).map_err(|err| {
        warn!(%err, "s3 upload failed");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}

fn decode_image(image_bytes: Bytes) -> Result<ImageData, StatusCode> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(5 * 1024);
    limits.max_image_height = Some(5 * 1024);
    limits.max_alloc = Some(90 * 1024 * 1024);

    let span = info_span!("decoding and resizing image", len = image_bytes.len()).entered();
    let mut image = ImageReader::new(Cursor::new(image_bytes));
    image.limits(limits);

    let mut image = image
        .with_guessed_format()
        .map_err(|err| {
            warn!(%err, "decoding image failed");
            StatusCode::BAD_REQUEST
        })?
        .decode()
        .map_err(|err| {
            warn!(%err, "decoding image failed");
            StatusCode::BAD_REQUEST
        })?;

    if image.width() > 2048 || image.height() > 2048 {
        let mut resizer = Resizer::new();
        let old_image = image;
        let (w, h) = resize_dimensions(old_image.width(), old_image.height(), 2048, 2048);
        image = DynamicImage::new(w, h, old_image.color());

        resizer
            .resize(&old_image, &mut image, None)
            .map_err(|err| {
                warn!(%err, "resizing image failed");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }
    drop(span);

    let thumbnail_image = image.thumbnail(512, 512);

    let blur_image = image.thumbnail_exact(8, 8);

    let mut encoder = encoder_builder()
        .quality(3.)
        // encoder speed 5 (out of 10, default is 7)
        // it's significantly faster and basically as good as 7
        .speed(EncoderSpeed::Hare)
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let span = info_span!(
        "encoding image",
        type = "full",
        width = image.width(),
        height = image.height()
    )
    .entered();
    let image = image.into_rgb8();
    let result: EncoderResult<u8> = encoder
        .encode(&image, image.width(), image.height())
        .map_err(|err| {
            warn!(%err, type = "full", "encoding image failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let full_dim = (image.width(), image.height());
    drop((image, span));

    let span = info_span!(
        "encoding image",
        type = "thumbnail",
        width = thumbnail_image.width(),
        height = thumbnail_image.height()
    )
    .entered();
    let thumbnail_image = thumbnail_image.into_rgb8();
    let thumbnail_result: EncoderResult<u8> = encoder
        .encode(
            &thumbnail_image,
            thumbnail_image.width(),
            thumbnail_image.height(),
        )
        .map_err(|err| {
            warn!(%err, type = "thumbnail", "encoding image failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let thumbnail_dim = (thumbnail_image.width(), thumbnail_image.height());
    drop((thumbnail_image, span));

    let span = info_span!(
        "encoding image",
        type = "blur",
        width = blur_image.width(),
        height = blur_image.height()
    )
    .entered();
    let blur_image = blur_image.into_rgb8();
    let blur_result: EncoderResult<u8> = encoder
        .encode(&blur_image, blur_image.width(), blur_image.height())
        .map_err(|err| {
            warn!(%err, type = "blue", "encoding image failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    drop((blur_image, span));

    Ok(ImageData {
        full: result.data,
        full_dim,
        thumbnail: thumbnail_result.data,
        thumbnail_dim,
        blur: blur_result.data,
    })
}
