use crate::NotificationEmailService;
use bfx_core::status::StatusExt;
use bfx_proto::notification::email::{SetEmailBlockedReply, SetEmailBlockedRequest};
use tonic::{Request, Response, Status};

impl NotificationEmailService {
    pub async fn set_email_blocked(
        &self,
        request: Request<SetEmailBlockedRequest>,
    ) -> Result<Response<SetEmailBlockedReply>, Status> {
        let request = request.into_inner();

        if request.blocked {
            sqlx::query!(
                "delete from notification_email.blocked_emails where email = $1",
                request.email,
            )
            .execute(&self.db)
            .await
            .map_err(Status::db)?;
        } else {
            sqlx::query!(
                "insert into notification_email.blocked_emails (email) values ($1)",
                request.email,
            )
            .execute(&self.db)
            .await
            .map_err(Status::db)?;
        }

        Ok(Response::new(SetEmailBlockedReply {}))
    }
}
