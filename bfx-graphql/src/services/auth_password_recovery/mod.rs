mod check_password_recovery_token;
mod request_password_recovery;
mod reset_password;

use crate::services::auth_password_recovery::check_password_recovery_token::CheckPasswordResetTokenQuery;
use crate::services::auth_password_recovery::request_password_recovery::RequestPasswordRecoveryMutation;
use crate::services::auth_password_recovery::reset_password::ResetPasswordMutation;
use async_graphql::MergedObject;

#[derive(Default, MergedObject)]
pub struct AuthPasswordRecoveryQuery(CheckPasswordResetTokenQuery);

#[derive(Default, MergedObject)]
pub struct AuthPasswordRecoveryMutation(RequestPasswordRecoveryMutation, ResetPasswordMutation);
