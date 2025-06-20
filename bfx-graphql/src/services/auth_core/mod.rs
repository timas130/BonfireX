use crate::services::auth_core::change_password::ChangePasswordMutation;
use crate::services::auth_core::login_email::LoginEmailMutation;
use crate::services::auth_core::me::MeQuery;
use crate::services::auth_core::register_email::RegisterEmailMutation;
use crate::services::auth_core::send_verification_email::SendVerificationEmailMutation;
use crate::services::auth_core::user_by_id::UserByIdQuery;
use crate::services::auth_core::verify_email::VerifyEmailMutation;
use async_graphql::MergedObject;

mod change_password;
pub mod data_loaders;
pub mod login_email;
mod me;
mod register_email;
mod send_verification_email;
mod user_by_id;
mod verify_email;

#[derive(MergedObject, Default)]
pub struct AuthCoreQuery(MeQuery, UserByIdQuery);

#[derive(MergedObject, Default)]
pub struct AuthCoreMutation(
    ChangePasswordMutation,
    LoginEmailMutation,
    RegisterEmailMutation,
    SendVerificationEmailMutation,
    VerifyEmailMutation,
);
