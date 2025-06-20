use crate::TranslationService;
use fluent::FluentResource;
use fluent::concurrent::FluentBundle;
use futures_util::TryStreamExt;
use sqlx::types::chrono::{DateTime, Utc};
use tonic::codegen::tokio_stream::StreamExt;
use tracing::{error, info, warn};
use unic_langid::LanguageIdentifier;

pub struct FluentBundleExt {
    pub bundle: FluentBundle<FluentResource>,
    resource_ids: Vec<(i64, DateTime<Utc>)>,
}

impl TranslationService {
    /// Reload localization resources from the database
    ///
    /// # Errors
    ///
    /// - If the database transaction fails
    /// - If parsing language identifiers fails
    /// - If parsing Fluent resources fails
    #[allow(clippy::too_many_lines)]
    pub async fn reload_localization(&self) -> anyhow::Result<()> {
        struct TranslationResourceDesc {
            id: i64,
            lang_id: LanguageIdentifier,
            modified_at: DateTime<Utc>,
        }

        let _lock = self.reload_lock.lock().await;

        let mut tx = self.db.begin().await?;

        // find what resources need to be reloaded
        let mut ids_to_reload = Vec::new();
        sqlx::query!("select id, lang_id, modified_at from translation.resources for update")
            .fetch(&mut *tx)
            .map(|r| {
                r.and_then(|record| {
                    record.lang_id
                        .parse()
                        .map_err(|err| {
                            warn!(record.id, record.lang_id, %err, "invalid lang_id in translation.resources");
                            sqlx::Error::Decode(Box::new(err))
                        })
                        .map(|lang_id| TranslationResourceDesc { id: record.id, lang_id, modified_at: record.modified_at })
                })
            })
            .map_ok(|desc| {
                let Some(bundle) = self.bundles.get(&desc.lang_id) else {
                    ids_to_reload.push(desc.id);
                    return;
                };

                let resource_id = bundle.resource_ids
                    .iter()
                    .find(|(id, _)| *id == desc.id);

                match resource_id {
                    None => ids_to_reload.push(desc.id),
                    Some((_, modified_at)) => {
                        if desc.modified_at > *modified_at {
                            ids_to_reload.push(desc.id);
                        }
                    }
                }
            })
            .try_collect::<()>()
            .await?;

        // reload the resources
        let mut updated_resource_count = 0;

        sqlx::query!(
            "select id, lang_id, source, modified_at
             from translation.resources
             where id = any($1)",
            &ids_to_reload
        )
        .fetch(&mut *tx)
        .map_ok(|record| {
            let lang_id: Result<LanguageIdentifier, _> = record.lang_id.parse();
            let Ok(lang_id) = lang_id else {
                error!(
                    record.id,
                    record.lang_id,
                    "desync with db: invalid lang_id found in translation.resources"
                );
                return;
            };

            let mut bundle =
                self.bundles
                    .entry(lang_id.clone())
                    .or_insert_with(|| FluentBundleExt {
                        bundle: FluentBundle::new_concurrent(vec![lang_id]),
                        resource_ids: Vec::new(),
                    });

            let resource = FluentResource::try_new(record.source);
            let resource = match resource {
                Ok(resource) => resource,
                Err((resource, errors)) => {
                    error!(
                        record.id,
                        record.lang_id,
                        ?errors,
                        "failed to parse fluent resource"
                    );
                    resource
                }
            };

            updated_resource_count += 1;

            bundle.bundle.add_resource_overriding(resource);
            let resource_id = bundle
                .resource_ids
                .iter_mut()
                .find(|(id, _)| *id == record.id);
            if let Some((_, modified_at)) = resource_id {
                *modified_at = record.modified_at;
            } else {
                bundle.resource_ids.push((record.id, record.modified_at));
            }
        })
        .try_collect::<()>()
        .await?;

        if updated_resource_count > 0 {
            info!(
                updated = updated_resource_count,
                "synced translation resources successfully"
            );
        }

        Ok(())
    }
}
