use crate::error::RespError;
use async_graphql::Context;
use bfx_core::service::id_encryption::IdEncryptor;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::UserContext;
use bfx_proto::auth::{GetUserByTokenReply, PermissionLevel, User};
use bfx_proto::factory::BuildableService;
use std::sync::Arc;
use tonic::transport::Channel;
use tonic::{Code, Status};

#[derive(Clone)]
pub struct GlobalContext {
    pub router: Channel,
    pub id_encryptor: Arc<IdEncryptor>,
}

pub struct LocalContext {
    pub user_context: UserContext,
    // basically user+session
    pub user: Option<GetUserByTokenReply>,
}

pub trait ContextExt {
    /// Get the user that authorized the request or throw an error
    ///
    /// # Errors
    ///
    /// - If the user is not logged in
    fn require_user(&self) -> Result<&User, Box<Status>>;

    /// Get the user that authorized the request
    fn user(&self) -> Option<&User>;

    /// Get request metadata (IP, user agent, etc.)
    fn user_context(&self) -> &UserContext;

    /// Check if the user either matches `user_id` or is an admin
    ///
    /// # Errors
    ///
    /// - If the user is not logged in
    /// - If the user is not an admin and `user_id` doesn't match the logged-in user's ID
    fn require_self_or_admin(&self, user_id: i64) -> Result<(), RespError>;
}

impl ContextExt for Context<'_> {
    fn require_user(&self) -> Result<&User, Box<Status>> {
        self.user()
            .ok_or_else(|| Status::coded(Code::Unauthenticated, ErrorCode::AccessDenied).into())
    }

    fn user(&self) -> Option<&User> {
        let req = self.data_unchecked::<LocalContext>();

        req.user.as_ref().and_then(|user| user.user.as_ref())
    }

    fn user_context(&self) -> &UserContext {
        let req = self.data_unchecked::<LocalContext>();

        &req.user_context
    }

    fn require_self_or_admin(&self, user_id: i64) -> Result<(), RespError> {
        let auth_user_id = self.user().map(|user| user.id);
        if auth_user_id == Some(user_id) {
            return Ok(());
        }

        let permission_level = self
            .user()
            .map_or(PermissionLevel::User as i32, |user| user.permission_level);
        if permission_level >= PermissionLevel::Admin.into() {
            return Ok(());
        }

        Err(Status::coded(Code::PermissionDenied, ErrorCode::AccessDenied).into())
    }
}

pub trait ServiceFactory {
    fn service<T: BuildableService>(&self) -> T;
}

impl ServiceFactory for Context<'_> {
    fn service<T: BuildableService>(&self) -> T {
        self.data_unchecked::<GlobalContext>().service()
    }
}

impl ServiceFactory for GlobalContext {
    fn service<T: BuildableService>(&self) -> T {
        T::new(self.router.clone())
    }
}
