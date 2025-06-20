#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

mod methods;
mod models;
mod util;

use bfx_core::logging::setup_logging;
use bfx_core::service::database::{Db, require_db};
use bfx_core::service::start_service;
use bfx_proto::auth::auth_core_server::{AuthCore, AuthCoreServer};
use bfx_proto::auth::{
    CreateUserReply, CreateUserRequest, GetUserByTokenReply, GetUserByTokenRequest,
    GetUsersByIdsReply, GetUsersByIdsRequest, LoginEmailReply, LoginEmailRequest,
};
use tonic::{Request, Response, Status};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let service = AuthCoreService {
        db: require_db().await?,
    };

    start_service(AuthCoreServer::new(service)).await?;

    Ok(())
}

pub struct AuthCoreService {
    db: Db,
}

#[tonic::async_trait]
impl AuthCore for AuthCoreService {
    async fn get_users_by_ids(
        &self,
        request: Request<GetUsersByIdsRequest>,
    ) -> Result<Response<GetUsersByIdsReply>, Status> {
        self.get_users_by_ids(request).await
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        self.create_user(request).await
    }

    async fn login_email(
        &self,
        request: Request<LoginEmailRequest>,
    ) -> Result<Response<LoginEmailReply>, Status> {
        self.login_email(request).await
    }

    async fn get_user_by_token(
        &self,
        request: Request<GetUserByTokenRequest>,
    ) -> Result<Response<GetUserByTokenReply>, Status> {
        self.get_user_by_token(request).await
    }
}
