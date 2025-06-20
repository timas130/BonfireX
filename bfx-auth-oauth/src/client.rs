use bfx_core::service::environment::require_env;
use bfx_core::status::{ErrorCode, StatusExt};
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::{
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, IssuerUrl, RedirectUrl,
};
use reqwest::redirect::Policy;
use std::sync::Arc;
use tonic::{Code, Status};

type Client = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

pub struct OAuthClients {
    pub http_client: reqwest::Client,
    pub google: Arc<Client>,
}

impl OAuthClients {
    pub async fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()?;

        Ok(Self {
            google: Arc::new(
                Self::from_provider_metadata(&client, "google", "https://accounts.google.com")
                    .await?,
            ),
            http_client: client,
        })
    }

    pub async fn from_provider_metadata(
        client: &reqwest::Client,
        provider_id: &str,
        issuer: &str,
    ) -> anyhow::Result<Client> {
        let provider_id_upper = provider_id.to_uppercase();

        let client_id = require_env(format!("OAUTH_{provider_id_upper}_CLIENT_ID"))?;
        let client_secret = require_env(format!("OAUTH_{provider_id_upper}_CLIENT_SECRET"))?;

        let frontend_root = require_env("FRONTEND_ROOT")?;
        let redirect_uri = format!("{frontend_root}/oauth/{provider_id}/callback");

        let provider_metadata: CoreProviderMetadata =
            CoreProviderMetadata::discover_async(IssuerUrl::new(issuer.to_string())?, client)
                .await?;
        let client = Client::from_provider_metadata(
            provider_metadata,
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri)?);

        Ok(client)
    }

    pub fn get_provider(&self, issuer: &str) -> Result<&Client, Status> {
        match issuer {
            "https://accounts.google.com" => Ok(self.google.as_ref()),
            _ => Err(Status::coded(
                Code::InvalidArgument,
                ErrorCode::UnknownProvider,
            )),
        }
    }

    pub fn get_provider_name(issuer: &str) -> &str {
        match issuer {
            "https://accounts.google.com" => "Google",
            iss => iss,
        }
    }
}
