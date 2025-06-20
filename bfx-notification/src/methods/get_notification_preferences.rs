use crate::NotificationService;
use crate::models::preferences::RawNotificationPreferences;
use bfx_core::status::StatusExt;
use bfx_proto::notification::{GetNotificationPreferencesReply, GetNotificationPreferencesRequest};
use tonic::{Request, Response, Status};

impl NotificationService {
    /// Get the user's notification preferences or the default ones
    ///
    /// # Errors
    ///
    /// - Miscellaneous internal errors
    pub async fn get_notification_preferences(
        &self,
        request: Request<GetNotificationPreferencesRequest>,
    ) -> Result<Response<GetNotificationPreferencesReply>, Status> {
        let request = request.into_inner();

        let preferences = self
            .get_raw_notification_preferences(request.user_id)
            .await?;

        Ok(Response::new(GetNotificationPreferencesReply {
            preferences: Some(preferences.into()),
        }))
    }

    /// Get the user's notification preferences or the default ones
    ///
    /// # Errors
    ///
    /// - Miscellaneous internal errors
    pub async fn get_raw_notification_preferences(
        &self,
        user_id: i64,
    ) -> Result<RawNotificationPreferences, Status> {
        let preferences = sqlx::query_as!(
            RawNotificationPreferences,
            "select * from notification.preferences where user_id = $1",
            user_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?
        .unwrap_or_else(|| RawNotificationPreferences {
            user_id,
            ..Default::default()
        });

        Ok(preferences)
    }
}
