use crate::ParamValue;
use crate::notification::email::SendEmailRequest;
use crate::notification::email::notification_email_client::NotificationEmailClient;
use crate::translation::translation_client::TranslationClient;
use crate::translation::{RenderTemplateRequest, TranslateRequest};
use std::collections::HashMap;
use tonic::Status;
use tonic::transport::Channel;

/// Send an email using the translation and notification services
///
/// # Errors
///
/// - If the template rendering fails
/// - If the translation fails
/// - If the email sending fails
/// - If the gRPC connections fail
pub async fn send_email(
    router: &Channel,
    to: String,
    subject_key: &str,
    source: &str,
    lang_id: String,
    context: HashMap<String, ParamValue>,
) -> Result<(), Status> {
    let mut translation = TranslationClient::new(router.clone());
    let mut email = NotificationEmailClient::new(router.clone());

    let message = translation
        .render_template(RenderTemplateRequest {
            source: source.into(),
            lang_id: lang_id.clone(),
            context: context.clone(),
        })
        .await?
        .into_inner()
        .output;

    email
        .send_email(SendEmailRequest {
            to,
            subject: translation
                .translate(TranslateRequest {
                    key: subject_key.into(),
                    lang: lang_id,
                    params: context,
                })
                .await?
                .into_inner()
                .text,
            body_html: message,
            ..Default::default()
        })
        .await?;

    Ok(())
}
