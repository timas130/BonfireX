use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub id: i64,
    pub owner_id: i64,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub display_name: String,
    pub privacy_url: Option<String>,
    pub tos_url: Option<String>,
    pub official: bool,
    pub allowed_scopes: Vec<String>,
    pub enforce_code_challenge: bool,
    pub created_at: DateTime<Utc>,
}

impl From<ClientInfo> for bfx_proto::auth::RpInfo {
    fn from(value: ClientInfo) -> Self {
        Self {
            id: value.id,
            display_name: value.display_name,
            privacy_url: value.privacy_url,
            tos_url: value.tos_url,
            official: value.official,
        }
    }
}
