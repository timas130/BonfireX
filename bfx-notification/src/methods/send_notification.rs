use crate::NotificationService;
use crate::definition::{EmailDefinition, NotificationDefinition};
use crate::models::notification::NotificationData;
use bfx_core::log_if_error::LogIfErrorExt;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::ParamValue;
use bfx_proto::auth::User;
use bfx_proto::auth::auth_core_client::AuthCoreClient;
use bfx_proto::notification::email::SendEmailRequest;
use bfx_proto::notification::email::notification_email_client::NotificationEmailClient;
use bfx_proto::notification::{NotificationParams, SendNotificationReply, SendNotificationRequest};
use bfx_proto::translation::ConditionalString;
use bfx_proto::translation::translation_client::TranslationClient;
use prost::Message;
use std::collections::HashMap;
use tonic::{Code, Request, Response, Status};

impl NotificationService {
    /// Send a notification to a user
    ///
    /// # Errors
    ///
    /// - If the definition is invalid
    /// - If rendering the template fails somewhere
    /// - If the user is not found and `user_override` is not provider and `email` is used
    /// - Miscellaneous internal errors
    pub async fn send_notification(
        &self,
        request: Request<SendNotificationRequest>,
    ) -> Result<Response<SendNotificationReply>, Status> {
        let request = request.into_inner();

        let definition = request.definition;
        let definition: NotificationDefinition =
            serde_yml::from_str(&definition).map_err(|err| {
                Status::coded(
                    Code::InvalidArgument,
                    ErrorCode::InvalidNotificationDefinition,
                )
                .with_source(err)
            })?;

        let preferences = self
            .get_raw_notification_preferences(request.user_id)
            .await?;

        if let Some(ref email_definition) = definition.email {
            let mut auth_core = AuthCoreClient::new(self.router.clone());

            let user = if let Some(user_override) = request.user_override {
                user_override
            } else {
                auth_core
                    .get_user_by_id(request.user_id)
                    .await?
                    .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::UserNotFound))?
            };

            self.send_email_notification(
                user,
                preferences.lang_id.clone(),
                request.params.clone(),
                email_definition.clone(),
            )
            .await?;
        }

        let data = NotificationData { definition };
        let params = NotificationParams {
            params: request.params,
        }
        .encode_to_vec();

        let notification = sqlx::query!(
            "insert into notification.notifications
             (user_id, definition_id, data, params)
             values ($1, $2, $3, $4)
             returning id",
            request.user_id,
            data.definition.id.clone(),
            serde_json::to_value(data)
                .map_err(From::from)
                .map_err(Status::anyhow)?,
            params,
        )
        .fetch_one(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(Response::new(SendNotificationReply {
            id: notification.id,
        }))
    }

    async fn send_email_notification(
        &self,
        user: User,
        lang_id: String,
        context: HashMap<String, ParamValue>,
        definition: EmailDefinition,
    ) -> Result<(), Status> {
        let Some(to) = user.email else {
            return Ok(());
        };

        let mut translation = TranslationClient::new(self.router.clone());
        let mut notification_email = NotificationEmailClient::new(self.router.clone());

        let subject = translation
            .render_string_set_ext(lang_id.clone(), definition.subject.into(), context.clone())
            .await?;

        let body = definition.body.into();
        let body = if definition.include_template {
            Self::wrap_string_set_in_template(body)
        } else {
            body
        };

        let body = translation
            .render_string_set_ext(lang_id, body, context)
            .await?;

        notification_email
            .send_email(SendEmailRequest {
                to,
                to_name: None,
                subject,
                body_html: body,
            })
            .await
            .log_if_error("sending notification email");

        Ok(())
    }

    fn wrap_string_set_in_template(body: Vec<ConditionalString>) -> Vec<ConditionalString> {
        body.into_iter()
            .map(|mut c| {
                c.value = format!(
                    "<p>{{{{ t(\"email-greeting\") }}}}</p>\n\n\
                     {}\n\n\
                     <p>{{{{ t(\"email-footer\") }}}}</p>",
                    c.value
                );
                c
            })
            .collect()
    }
}
