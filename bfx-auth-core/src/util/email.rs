use crate::AuthCoreService;
use bfx_core::status::{ErrorCode, StatusExt};
use tonic::{Code, Status};
use validator::ValidateEmail;

impl AuthCoreService {
    /// Check if an email is valid
    ///
    /// # Errors
    ///
    /// - If the email is invalid
    pub fn check_email(&self, email: &str) -> Result<(), Status> {
        if email.validate_email() {
            Ok(())
        } else {
            Err(Status::coded(
                Code::InvalidArgument,
                ErrorCode::EmailInvalid,
            ))
        }
    }
}
