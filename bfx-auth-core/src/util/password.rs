use crate::AuthCoreService;
use bfx_core::status::{ErrorCode, StatusExt};
use tonic::Status;
use zxcvbn::Score;

impl AuthCoreService {
    /// Check if a password is strong enough
    ///
    /// # Arguments
    ///
    /// - `password`: The password
    /// - `user_inputs`: Any other user inputs (e.g., username, email)
    ///
    /// # Errors
    ///
    /// - If the password is weak
    pub fn check_password(&self, password: &str, user_inputs: &[&str]) -> Result<(), Status> {
        let entropy = zxcvbn::zxcvbn(password, user_inputs);
        if entropy.score() < Score::Three {
            Err(Status::coded(
                tonic::Code::Unauthenticated,
                ErrorCode::WeakPassword,
            ))
        } else {
            Ok(())
        }
    }
}
