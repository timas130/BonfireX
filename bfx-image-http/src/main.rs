mod endpoints;
pub mod util;

use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::{Extension, Router};
use bfx_core::logging::setup_logging;
use bfx_core::service::database::{Db, require_db};
use bfx_core::service::get_tcp_listener;
use bfx_core::service::s3::require_s3;
use s3::Bucket;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let app = Router::new()
        .route("/upload", post(endpoints::upload))
        .layer(DefaultBodyLimit::max(4 * 1024 * 1024))
        .layer(Extension(ImageHttpService {
            db: require_db().await?,
            bucket: Arc::new(require_s3(false).await?),
        }));

    let listener = get_tcp_listener().await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct ImageHttpService {
    db: Db,
    bucket: Arc<Bucket>,
}
