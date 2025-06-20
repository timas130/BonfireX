mod blacklist_refresher;
mod custom_header;
mod mailer;
mod methods;

use crate::mailer::{Mailer, require_mailer};
use anyhow::anyhow;
use bfx_core::logging::setup_logging;
use bfx_core::service::database::{Db, require_db};
use bfx_core::service::environment::require_env;
use bfx_core::service::start_service;
use bfx_proto::notification::email::notification_email_server::{
    NotificationEmail, NotificationEmailServer,
};
use bfx_proto::notification::email::{
    CheckValidEmailReply, CheckValidEmailRequest, SendEmailReply, SendEmailRequest,
    SetEmailBlockedReply, SetEmailBlockedRequest,
};
use hickory_resolver::Resolver;
use hickory_resolver::name_server::TokioConnectionProvider;
use lettre::message::Mailbox;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let service = NotificationEmailService {
        db: require_db().await?,
        resolver: Arc::new(Resolver::builder_tokio()?.build()),
        hostname: require_env("EMAIL_HOSTNAME")?,
        from: Mailbox::from_str(&require_env("EMAIL_FROM")?)
            .map_err(|err| anyhow!("invalid EMAIL_FROM: {err}"))?,
        debug: require_env("EMAIL_DEBUG")
            .map(|val| val == "1")
            .unwrap_or(false),
        mailer: require_mailer()?,
    };

    service.clone().start_blacklist_refresher();

    start_service(NotificationEmailServer::new(service)).await?;

    Ok(())
}

#[derive(Clone)]
struct NotificationEmailService {
    db: Db,
    resolver: Arc<Resolver<TokioConnectionProvider>>,

    hostname: String,
    from: Mailbox,
    debug: bool,
    mailer: Mailer,
}

#[tonic::async_trait]
impl NotificationEmail for NotificationEmailService {
    async fn send_email(
        &self,
        request: Request<SendEmailRequest>,
    ) -> Result<Response<SendEmailReply>, Status> {
        self.send_email(request).await
    }

    async fn set_email_blocked(
        &self,
        request: Request<SetEmailBlockedRequest>,
    ) -> Result<Response<SetEmailBlockedReply>, Status> {
        self.set_email_blocked(request).await
    }

    async fn check_valid_email(
        &self,
        request: Request<CheckValidEmailRequest>,
    ) -> Result<Response<CheckValidEmailReply>, Status> {
        self.check_valid_email(request).await
    }
}
