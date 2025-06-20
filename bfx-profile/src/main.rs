mod methods;
pub mod models;

use bfx_core::logging::setup_logging;
use bfx_core::service::client::require_router;
use bfx_core::service::database::{Db, require_db};
use bfx_core::service::start_service;
use bfx_proto::profile::profile_server::{Profile, ProfileServer};
use bfx_proto::profile::{
    GetProfileBulkReply, GetProfileBulkRequest, GetProfileReply, GetProfileRequest, SetNoteReply,
    SetNoteRequest, UpdateProfileReply, UpdateProfileRequest, UpdateUsernameReply,
    UpdateUsernameRequest,
};
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let service = ProfileService {
        db: require_db().await?,
        router: require_router()?,
    };

    start_service(ProfileServer::new(service)).await?;

    Ok(())
}

#[derive(Clone)]
struct ProfileService {
    db: Db,
    router: Channel,
}

#[tonic::async_trait]
impl Profile for ProfileService {
    async fn get_profile(
        &self,
        request: Request<GetProfileRequest>,
    ) -> Result<Response<GetProfileReply>, Status> {
        self.get_profile(request).await
    }

    async fn get_profile_bulk(
        &self,
        request: Request<GetProfileBulkRequest>,
    ) -> Result<Response<GetProfileBulkReply>, Status> {
        self.get_profile_bulk(request).await
    }

    async fn update_profile(
        &self,
        request: Request<UpdateProfileRequest>,
    ) -> Result<Response<UpdateProfileReply>, Status> {
        self.update_profile(request).await
    }

    async fn update_username(
        &self,
        request: Request<UpdateUsernameRequest>,
    ) -> Result<Response<UpdateUsernameReply>, Status> {
        self.update_username(request).await
    }

    async fn set_note(
        &self,
        request: Request<SetNoteRequest>,
    ) -> Result<Response<SetNoteReply>, Status> {
        self.set_note(request).await
    }
}
