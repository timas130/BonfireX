use crate::NotificationService;
use crate::models::preferences::RawNotificationPreferences;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::notification::{SetNotificationPreferencesReply, SetNotificationPreferencesRequest};
use tonic::{Code, Request, Response, Status};

impl NotificationService {
    /// Set the user's notification preferences
    ///
    /// # Errors
    ///
    /// - Miscellaneous internal errors
    pub async fn set_notification_preferences(
        &self,
        request: Request<SetNotificationPreferencesRequest>,
    ) -> Result<Response<SetNotificationPreferencesReply>, Status> {
        let request = request.into_inner();

        let preferences = request
            .preferences
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

        let preferences = sqlx::query_as!(
            RawNotificationPreferences,
            "insert into notification.preferences
             (user_id, lang_id)
             values ($1, $2)
             on conflict (user_id) do update
             set lang_id = $2
             returning *",
            request.user_id,
            preferences.lang_id,
        )
        .fetch_one(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(Response::new(SetNotificationPreferencesReply {
            preferences: Some(preferences.into()),
        }))
    }
}
