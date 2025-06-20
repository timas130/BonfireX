use crate::NotificationEmailService;
use itertools::Itertools;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{Instrument, info, info_span, warn};

static BLACKLIST_URLS: &[&str] = &[
    // as you can see, I don't care about false positives
    "https://raw.githubusercontent.com/FGRibreau/mailchecker/master/list.txt",
    "https://raw.githubusercontent.com/micke/valid_email2/refs/heads/main/config/disposable_email_domains.txt",
    "https://raw.githubusercontent.com/7c/fakefilter/refs/heads/main/txt/data.txt",
    "https://raw.githubusercontent.com/disposable/disposable-email-domains/master/domains.txt",
];

impl NotificationEmailService {
    pub fn start_blacklist_refresher(self) {
        tokio::spawn(async move {
            loop {
                if let Err(err) = self.refresh_blacklist().await {
                    warn!(err = %err, "failed to refresh blacklist");
                }
                sleep(Duration::from_secs(60 * 60)).await;
            }
        });
    }

    pub async fn refresh_blacklist(&self) -> anyhow::Result<()> {
        // download the blacklists
        let blacklisted_domains = async move {
            let mut blacklisted_domains = HashSet::new();

            for url in BLACKLIST_URLS {
                let response = reqwest::get(*url).await;
                let Ok(response) = response else {
                    warn!(
                        url,
                        err = %response.unwrap_err(),
                        "failed to download blacklist"
                    );
                    continue;
                };

                let text = response.text().await;
                let Ok(text) = text else {
                    warn!(
                        url,
                        err = %text.unwrap_err(),
                        "failed to download blacklist"
                    );
                    continue;
                };

                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    blacklisted_domains.insert(line.to_string());
                }
            }

            info!(domains = blacklisted_domains.len(), "downloaded blacklists");

            Ok::<_, anyhow::Error>(blacklisted_domains)
        }
        .instrument(info_span!("downloading blacklists"))
        .await?;

        // insert the domains into the database
        async move {
            let domain_chunks = blacklisted_domains.into_iter().chunks(u16::MAX as usize);
            let domain_chunks = domain_chunks.into_iter();

            let mut queries = vec![];
            for domains in domain_chunks {
                let mut query = sqlx::QueryBuilder::new(
                    "insert into notification_email.blocked_email_domains (domain) ",
                );

                query.push_values(domains, |mut query, domain| {
                    query.push_bind(domain);
                });

                query.push(" on conflict (domain) do nothing");

                queries.push(query);
            }

            for mut query in queries {
                query.build().execute(&self.db).await?;
            }

            Ok::<_, anyhow::Error>(())
        }
        .instrument(info_span!("refreshing blacklist in db"))
        .await?;

        Ok(())
    }
}
