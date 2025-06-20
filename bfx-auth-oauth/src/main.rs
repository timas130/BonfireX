mod client;
mod methods;
pub mod models;

use crate::client::OAuthClients;
use bfx_core::logging::setup_logging;
use bfx_core::service::client::require_router;
use bfx_core::service::database::{Db, require_db};
use bfx_core::service::start_service;
use bfx_proto::auth::auth_o_auth_server::{AuthOAuth, AuthOAuthServer};
use bfx_proto::auth::{
    BindOAuthReply, BindOAuthRequest, FinishOAuthFlowReply, FinishOAuthFlowRequest,
    GetAuthSourcesReply, GetAuthSourcesRequest, StartOAuthFlowReply, StartOAuthFlowRequest,
    UnbindAuthSourceReply, UnbindAuthSourceRequest,
};
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let service = AuthOAuthService {
        db: require_db().await?,
        router: require_router()?,
        clients: OAuthClients::new().await?,
    };

    start_service(AuthOAuthServer::new(service)).await?;

    Ok(())
}

pub struct AuthOAuthService {
    db: Db,
    router: Channel,
    clients: OAuthClients,
}

#[tonic::async_trait]
impl AuthOAuth for AuthOAuthService {
    async fn start_oauth_flow(
        &self,
        request: Request<StartOAuthFlowRequest>,
    ) -> Result<Response<StartOAuthFlowReply>, Status> {
        self.start_oauth_flow(request).await
    }

    async fn finish_oauth_flow(
        &self,
        request: Request<FinishOAuthFlowRequest>,
    ) -> Result<Response<FinishOAuthFlowReply>, Status> {
        self.finish_oauth_flow(request).await
    }

    async fn bind_oauth(
        &self,
        request: Request<BindOAuthRequest>,
    ) -> Result<Response<BindOAuthReply>, Status> {
        self.bind_oauth(request).await
    }

    async fn get_auth_sources(
        &self,
        request: Request<GetAuthSourcesRequest>,
    ) -> Result<Response<GetAuthSourcesReply>, Status> {
        self.get_auth_sources(request).await
    }

    async fn unbind_auth_source(
        &self,
        request: Request<UnbindAuthSourceRequest>,
    ) -> Result<Response<UnbindAuthSourceReply>, Status> {
        self.unbind_auth_source(request).await
    }
}
