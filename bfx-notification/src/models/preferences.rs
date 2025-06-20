use bfx_proto::notification::NotificationPreferences;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct RawNotificationPreferences {
    pub user_id: i64,
    pub lang_id: String,
    pub created_at: DateTime<Utc>,
}

impl Default for RawNotificationPreferences {
    fn default() -> Self {
        Self {
            user_id: 0,
            lang_id: "en".to_string(),
            created_at: Utc::now(),
        }
    }
}

impl From<RawNotificationPreferences> for NotificationPreferences {
    fn from(value: RawNotificationPreferences) -> Self {
        Self {
            lang_id: value.lang_id,
        }
    }
}
