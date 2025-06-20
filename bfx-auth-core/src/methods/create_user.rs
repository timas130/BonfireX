use crate::AuthCoreService;
use crate::models::user::RawUser;
use bfx_core::service::database::DbResultExt;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{CreateUserReply, CreateUserRequest, LoginAttemptStatus, User};
use tonic::{Code, Request, Response, Status};

impl AuthCoreService {
    /// Create a new user
    ///
    /// # Errors
    ///
    /// - If the email is invalid
    /// - If the password is too weak
    /// - If the email already exists
    /// - Miscellaneous internal errors
    pub async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        let request = request.into_inner();

        let user_context = request
            .user_context
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

        if let Some(email) = &request.email {
            self.check_email(email).await?;
        }

        let password = if let Some(password) = request.password {
            #[allow(clippy::option_if_let_else)]
            let user_inputs = match &request.email {
                Some(email) => &[email.as_str()] as &[&str],
                None => &[] as &[&str],
            };
            self.check_password(&password, user_inputs)?;

            Some(self.hash_password(&password).map_err(Status::anyhow)?)
        } else {
            None
        };

        let result = sqlx::query_as!(
            RawUser,
            "insert into auth_core.users (email, active, password) \
             values ($1, $2, $3) \
             returning *",
            request.email,
            request.active,
            password,
        )
        .fetch_one(&self.db)
        .await;

        if result.is_unique_violation() {
            return Err(Status::coded(Code::AlreadyExists, ErrorCode::EmailExists));
        }

        let result: User = result.map_err(Status::db)?.into();

        let login_attempt = self
            .create_login_attempt(result.id, &user_context, LoginAttemptStatus::Success)
            .await?;
        let session = self
            .create_session(
                result.id,
                Some(login_attempt.id),
                login_attempt.user_context_id,
            )
            .await?;

        Ok(Response::new(CreateUserReply {
            user: Some(result),
            tokens: Some(session.into()),
        }))
    }
}
