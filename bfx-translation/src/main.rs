mod lang_id_ext;
mod localization_cache;
mod methods;

use crate::localization_cache::FluentBundleExt;
use bfx_core::logging::setup_logging;
use bfx_core::service::database::{Db, require_db};
use bfx_core::service::start_service;
use bfx_proto::translation::translation_server::{Translation, TranslationServer};
use bfx_proto::translation::{
    RenderStringSetRequest, RenderTemplateReply, RenderTemplateRequest, TranslateReply,
    TranslateRequest, WriteFileReply, WriteFileRequest,
};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::error;
use unic_langid::LanguageIdentifier;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let service = TranslationService {
        db: require_db().await?,
        bundles: Arc::new(DashMap::new()),
        reload_lock: Arc::new(Mutex::new(())),
    };

    service.reload_localization().await?;

    let service_clone = service.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            let result = service_clone.reload_localization().await;
            if let Err(err) = result {
                error!(%err, "failed to reload localization");
            }
        }
    });

    start_service(TranslationServer::new(service)).await?;

    Ok(())
}

#[derive(Clone)]
pub struct TranslationService {
    db: Db,
    bundles: Arc<DashMap<LanguageIdentifier, FluentBundleExt>>,
    reload_lock: Arc<Mutex<()>>,
}

#[tonic::async_trait]
impl Translation for TranslationService {
    async fn translate(
        &self,
        request: Request<TranslateRequest>,
    ) -> Result<Response<TranslateReply>, Status> {
        self.translate(request)
    }

    async fn write_file(
        &self,
        request: Request<WriteFileRequest>,
    ) -> Result<Response<WriteFileReply>, Status> {
        self.write_file(request).await
    }

    async fn render_template(
        &self,
        request: Request<RenderTemplateRequest>,
    ) -> Result<Response<RenderTemplateReply>, Status> {
        self.render_template(request)
    }

    async fn render_string_set(
        &self,
        request: Request<RenderStringSetRequest>,
    ) -> Result<Response<RenderTemplateReply>, Status> {
        self.render_string_set(request)
    }
}
