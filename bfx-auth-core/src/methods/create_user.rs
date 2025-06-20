use crate::AuthCoreService;
use crate::models::user::RawUser;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{CreateUserReply, CreateUserRequest};
use sqlx::error::ErrorKind;
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

        if let Some(email) = &request.email {
            self.check_email(email)?;
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

        if let Err(err) = &result {
            if let Some(err) = err.as_database_error() {
                if err.kind() == ErrorKind::UniqueViolation {
                    return Err(Status::coded(Code::AlreadyExists, ErrorCode::EmailExists));
                }
            }
        }

        let result = result.map_err(Status::db)?.into();

        Ok(Response::new(CreateUserReply { user: Some(result) }))
    }
}
