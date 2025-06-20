use crate::AuthCoreService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::notification::email::notification_email_client::NotificationEmailClient;
use bfx_proto::notification::email::{CheckValidEmailRequest, EmailValidationError};
use tonic::{Code, Status};
use validator::ValidateEmail;

impl AuthCoreService {
    /// Check if an email is valid and is not blacklisted
    ///
    /// # Errors
    ///
    /// - If the email is invalid
    /// - If `bfx-notification-email` decides that the email is blacklisted
    /// - Miscellaneous internal errors
    pub async fn check_email(&self, email: &str) -> Result<(), Status> {
        self.check_email_simple(email)?;

        let mut notification_email = NotificationEmailClient::new(self.router.clone());
        let resp = notification_email
            .check_valid_email(CheckValidEmailRequest {
                email: email.to_string(),
            })
            .await?
            .into_inner();

        if !resp.valid {
            return Err(
                Status::coded(Code::InvalidArgument, ErrorCode::EmailInvalid).with_details(
                    resp.error
                        .and_then(|e| EmailValidationError::try_from(e).ok())
                        .map_or("Unknown", |e| e.as_str_name()),
                ),
            );
        }

        Ok(())
    }

    /// Check if an email is valid without sending a request to `bfx-notification-email`
    ///
    /// # Errors
    ///
    /// - If the email is invalid
    pub fn check_email_simple(&self, email: &str) -> Result<(), Status> {
        if !email.validate_email() {
            return Err(Status::coded(
                Code::InvalidArgument,
                ErrorCode::EmailInvalid,
            ));
        }

        Ok(())
    }
}
