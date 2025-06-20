use crate::NotificationEmailService;
use crate::custom_header::CustomHeader;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::notification::email::{SendEmailReply, SendEmailRequest};
use lettre::message::{Mailbox, MultiPart};
use lettre::{AsyncTransport, Message};
use nanoid::nanoid;
use std::str::FromStr;
use tonic::{Code, Request, Response, Status};
use tracing::{info, warn};

impl NotificationEmailService {
    pub async fn send_email(
        &self,
        request: Request<SendEmailRequest>,
    ) -> Result<Response<SendEmailReply>, Status> {
        let request = request.into_inner();

        // check email validity
        let address = Self::parse_email(&request.to)
            .map_err(|_| Status::coded(Code::InvalidArgument, ErrorCode::EmailInvalid))?;
        let address_validation_error = self.check_email(&address).await?;
        if let Some(_error) = address_validation_error {
            return Err(Status::coded(
                Code::InvalidArgument,
                ErrorCode::EmailInvalid,
            ));
        }

        // prepare To header
        let address = lettre::Address::from_str(address.as_str())
            .map_err(|_| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;
        let to_mailbox = Mailbox::new(request.to_name, address);

        // prepare Message-Id
        let message_id = format!("<{}@{}>", nanoid!(), &self.hostname);

        // prepare body
        let body_html = request.body_html;
        let body_plain = ammonia::Builder::empty().clean(&body_html).to_string();

        // insert into log for audit
        let unsubscribe_token = nanoid!(36);
        let email_log = sqlx::query!(
            "insert into notification_email.email_log
             (message_id, destination, subject, unsubscribe_token)
             values ($1, $2, $3, $4)
             returning *",
            message_id,
            to_mailbox.to_string(),
            request.subject,
            unsubscribe_token,
        )
        .fetch_one(&self.db)
        .await
        .map_err(Status::db)?;

        // compose and send the message
        let message = Message::builder()
            .message_id(Some(message_id.clone()))
            .from(self.from.clone())
            .to(to_mailbox)
            .subject(request.subject)
            .header(CustomHeader(
                "X-Bonfire-Message-Id",
                email_log.id.to_string(),
            ))
            .multipart(MultiPart::alternative_plain_html(body_plain, body_html))
            .map_err(|err| {
                Status::coded(Code::InvalidArgument, ErrorCode::Internal).with_source(err)
            })?;

        if self.debug {
            let message = message.formatted();
            let message = String::from_utf8_lossy(&message);
            info!("sending message:\n{message}");
        } else {
            self.mailer.send(message).await.map_err(|err| {
                warn!(%err, "failed to send email");
                Status::coded(Code::Internal, ErrorCode::Internal).with_source(err)
            })?;
        }

        Ok(Response::new(SendEmailReply { id: message_id }))
    }
}
