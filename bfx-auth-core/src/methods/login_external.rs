use crate::AuthCoreService;
use crate::models::user::RawUser;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{LoginAttemptStatus, LoginExternalReply, LoginExternalRequest};
use tonic::{Code, Request, Response, Status};

impl AuthCoreService {
    /// Simulates a successful login (for an external auth method)
    ///
    /// # Errors
    ///
    /// - If the user doesn't exist
    /// - If [`AuthCoreService::check_login_attempts`] fails
    /// - If the user is banned
    /// - Miscellaneous internal errors
    pub async fn login_external(
        &self,
        request: Request<LoginExternalRequest>,
    ) -> Result<Response<LoginExternalReply>, Status> {
        let request = request.into_inner();

        let user_context = request
            .user_context
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

        self.check_login_attempts(&user_context).await?;

        let user = RawUser::by_id(self, request.user_id)
            .await?
            .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::UserNotFound))?;

        if user.banned {
            return Err(Status::coded(Code::PermissionDenied, ErrorCode::UserBanned));
        }

        let login_attempt = self
            .create_login_attempt(request.user_id, &user_context, LoginAttemptStatus::Success)
            .await?;

        let session = self
            .create_session(
                request.user_id,
                Some(login_attempt.id),
                login_attempt.user_context_id,
            )
            .await?;

        self.send_login_notification(user, user_context).await;

        Ok(Response::new(LoginExternalReply {
            tokens: Some(session.into()),
        }))
    }
}
