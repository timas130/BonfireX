pub mod definition;
mod methods;
pub mod models;

use bfx_core::service::database::Db;
use bfx_proto::notification::notification_server::Notification;
use bfx_proto::notification::{
    GetNotificationPreferencesReply, GetNotificationPreferencesRequest, SendNotificationReply,
    SendNotificationRequest, SetNotificationPreferencesReply, SetNotificationPreferencesRequest,
};
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

pub struct NotificationService {
    pub db: Db,
    pub router: Channel,
}

#[tonic::async_trait]
impl Notification for NotificationService {
    async fn send_notification(
        &self,
        request: Request<SendNotificationRequest>,
    ) -> Result<Response<SendNotificationReply>, Status> {
        self.send_notification(request).await
    }

    async fn get_notification_preferences(
        &self,
        request: Request<GetNotificationPreferencesRequest>,
    ) -> Result<Response<GetNotificationPreferencesReply>, Status> {
        self.get_notification_preferences(request).await
    }

    async fn set_notification_preferences(
        &self,
        request: Request<SetNotificationPreferencesRequest>,
    ) -> Result<Response<SetNotificationPreferencesReply>, Status> {
        self.set_notification_preferences(request).await
    }
}
