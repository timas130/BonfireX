use bfx_core::logging::setup_logging;
use bfx_core::service::client::require_router;
use bfx_proto::translation::WriteFileRequest;
use bfx_proto::translation::translation_client::TranslationClient;
use std::path::Component;
use tracing::{info, warn};
use walkdir::{DirEntry, WalkDir};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let router = require_router()?;
    let mut translation = TranslationClient::new(router);

    let translations_dir =
        std::env::var("TRANSLATIONS_DIR").unwrap_or_else(|_| "./translations".to_string());
    let translations_dir = std::path::PathBuf::from(translations_dir).canonicalize()?;

    if !translations_dir.is_dir() {
        anyhow::bail!("translations directory not found: {translations_dir:?}");
    }

    let walk = WalkDir::new(&translations_dir);
    let mut resources = walk
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().unwrap_or_default() == "ftl")
        .map(DirEntry::into_path)
        .collect::<Vec<_>>();

    resources.sort();

    info!("found {} resources", resources.len());

    for resource in resources {
        let path = resource.strip_prefix(&translations_dir)?;

        let lang_id = path.components().next();
        let Some(lang_id) = lang_id else {
            warn!("invalid resource path: {resource:?}");
            continue;
        };
        let Component::Normal(lang_id) = lang_id else {
            warn!("invalid resource path (must start with language id): {resource:?}");
            continue;
        };

        let source = std::fs::read_to_string(&resource);
        let Ok(source) = source else {
            warn!(
                "failed to read resource {resource:?}: {}",
                source.unwrap_err()
            );
            continue;
        };

        let path = path.to_string_lossy().into_owned();
        let lang_id = lang_id.to_string_lossy().into_owned();

        info!("writing resource {path}");

        translation
            .write_file(WriteFileRequest {
                path,
                lang_id,
                source,
            })
            .await?;
    }

    info!("all done");

    Ok(())
}
