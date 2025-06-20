use bfx_auth_core::AuthCoreService;
use bfx_core::logging::setup_logging;
use bfx_core::service::client::require_router;
use bfx_core::service::database::require_db;
use bfx_core::service::environment::require_env;
use bfx_core::service::start_service;
use bfx_proto::auth::auth_core_server::AuthCoreServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let router = require_router()?;
    let service = AuthCoreService {
        db: require_db().await?,
        router,

        frontend_root: require_env("FRONTEND_ROOT")?,
    };

    start_service(AuthCoreServer::new(service)).await?;

    Ok(())
}
