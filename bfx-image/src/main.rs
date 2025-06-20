mod methods;
mod models;

use bfx_core::logging::setup_logging;
use bfx_core::service::database::{Db, require_db};
use bfx_core::service::s3::require_s3;
use bfx_core::service::start_service;
use bfx_proto::image::image_server::{Image, ImageServer};
use bfx_proto::image::{
    GetImageBulkRequest, GetImageBulkResponse, GetImageRequest, GetImageResponse,
    RequestUploadRequest, RequestUploadResponse, SetImageRefRequest, SetImageRefResponse,
    UseImageTicketRequest, UseImageTicketResponse,
};
use s3::Bucket;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let service = ImageService {
        db: require_db().await?,
        public_bucket: Arc::new(require_s3(true).await?),
    };

    start_service(ImageServer::new(service)).await?;

    Ok(())
}

#[derive(Clone)]
struct ImageService {
    db: Db,
    public_bucket: Arc<Bucket>,
}

#[tonic::async_trait]
impl Image for ImageService {
    async fn request_upload(
        &self,
        request: Request<RequestUploadRequest>,
    ) -> Result<Response<RequestUploadResponse>, Status> {
        self.request_upload(request).await
    }

    async fn use_image_ticket(
        &self,
        request: Request<UseImageTicketRequest>,
    ) -> Result<Response<UseImageTicketResponse>, Status> {
        self.use_image_ticket(request).await
    }

    async fn get_image(
        &self,
        request: Request<GetImageRequest>,
    ) -> Result<Response<GetImageResponse>, Status> {
        self.get_image(request).await
    }

    async fn get_image_bulk(
        &self,
        request: Request<GetImageBulkRequest>,
    ) -> Result<Response<GetImageBulkResponse>, Status> {
        self.get_image_bulk(request).await
    }

    async fn set_image_ref(
        &self,
        request: Request<SetImageRefRequest>,
    ) -> Result<Response<SetImageRefResponse>, Status> {
        self.set_image_ref(request).await
    }
}
