use bfx_core::logging::setup_logging;
use bfx_core::service::client::require_router;
use bfx_core::service::database::require_db;
use bfx_core::service::start_service;
use bfx_notification::NotificationService;
use bfx_proto::notification::notification_server::NotificationServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let service = NotificationService {
        db: require_db().await?,
        router: require_router()?,
    };

    start_service(NotificationServer::new(service)).await?;

    Ok(())
}
