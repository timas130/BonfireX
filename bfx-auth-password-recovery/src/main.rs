mod methods;

use bfx_core::logging::setup_logging;
use bfx_core::service::client::require_router;
use bfx_core::service::database::{Db, require_db};
use bfx_core::service::environment::require_env;
use bfx_core::service::start_service;
use bfx_proto::auth::password_recovery_server::{PasswordRecovery, PasswordRecoveryServer};
use bfx_proto::auth::{
    CheckPasswordResetReply, CheckPasswordResetTokenRequest, RequestPasswordRecoveryReply,
    RequestPasswordRecoveryRequest, ResetPasswordReply, ResetPasswordRequest,
};
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let service = PasswordRecoveryService {
        db: require_db().await?,
        router: require_router()?,
        frontend_root: require_env("FRONTEND_ROOT")?,
    };

    start_service(PasswordRecoveryServer::new(service)).await?;

    Ok(())
}

struct PasswordRecoveryService {
    db: Db,
    router: Channel,
    frontend_root: String,
}

#[tonic::async_trait]
impl PasswordRecovery for PasswordRecoveryService {
    async fn request_password_recovery(
        &self,
        request: Request<RequestPasswordRecoveryRequest>,
    ) -> Result<Response<RequestPasswordRecoveryReply>, Status> {
        self.request_password_recovery(request).await
    }

    async fn check_password_reset_token(
        &self,
        request: Request<CheckPasswordResetTokenRequest>,
    ) -> Result<Response<CheckPasswordResetReply>, Status> {
        self.check_password_reset_token(request).await
    }

    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordReply>, Status> {
        self.reset_password(request).await
    }
}
