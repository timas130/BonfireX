use crate::AuthCoreService;
use crate::models::login_attempt::RawLoginAttempt;
use crate::models::session::RawSession;
use crate::models::user::RawUser;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::UserContext;
use bfx_proto::auth::login_email_reply::LoginResult;
use bfx_proto::auth::{LoginAttemptStatus, LoginEmailReply, LoginEmailRequest, Tokens};
use chrono::{TimeDelta, Utc};
use nanoid::nanoid;
use sqlx::types::ipnet::IpNet;
use std::str::FromStr;
use tonic::{Code, Request, Response, Status};

impl AuthCoreService {
    /// Log into an account with an email and password
    ///
    /// # Errors
    ///
    /// - If the email is invalid
    /// - If the user (by IP) has tried to log in too many times
    /// - If the user doesn't exist
    /// - If the password is incorrect
    /// - If the user is not active (email not verified) or is banned
    /// - Miscellaneous internal errors
    pub async fn login_email(
        &self,
        request: Request<LoginEmailRequest>,
    ) -> Result<Response<LoginEmailReply>, Status> {
        let request = request.into_inner();

        let user_context = request
            .user_context
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

        self.check_email(&request.email)?;
        self.check_login_attempts(&user_context).await?;

        let user = sqlx::query_as!(
            RawUser,
            "select * from auth_core.users where email = $1",
            request.email,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?;

        let Some(user) = user else {
            return Err(Status::coded(
                Code::FailedPrecondition,
                ErrorCode::UserNotFound,
            ));
        };

        let Some(password) = user.password else {
            return Err(Status::coded(
                Code::FailedPrecondition,
                ErrorCode::PasswordNotSet,
            ));
        };

        let password_ok = self
            .verify_password(&request.password, &password)
            .map_err(Status::anyhow)?;

        if !password_ok {
            self.create_login_attempt(
                user.id,
                &user_context,
                LoginAttemptStatus::IncorrectPassword,
            )
            .await?;

            return Err(Status::coded(
                Code::PermissionDenied,
                ErrorCode::IncorrectPassword,
            ));
        }

        if !user.active {
            return Err(Status::coded(
                Code::PermissionDenied,
                ErrorCode::UserNotActive,
            ));
        }

        if user.banned {
            return Err(Status::coded(Code::PermissionDenied, ErrorCode::UserBanned));
        }

        let login_attempt = self
            .create_login_attempt(user.id, &user_context, LoginAttemptStatus::Success)
            .await?;

        let session = self
            .create_session(
                user.id,
                Some(login_attempt.id),
                login_attempt.user_context_id,
            )
            .await?;

        Ok(Response::new(LoginEmailReply {
            login_result: Some(LoginResult::Tokens(Tokens {
                access_token: session.access_token,
                session_id: session.id,
                login_attempt_id: login_attempt.id,
            })),
        }))
    }

    fn parse_ip(ip: &str) -> Result<IpNet, Status> {
        IpNet::from_str(ip).map_err(|_| Status::coded(Code::InvalidArgument, ErrorCode::Internal))
    }

    async fn check_login_attempts(&self, user_context: &UserContext) -> Result<(), Status> {
        let ip = Self::parse_ip(&user_context.ip)?;

        let num_attempts = sqlx::query_scalar!(
            "select count(*) from auth_core.login_attempts \
             inner join auth_core.user_contexts \
                 on user_contexts.id = login_attempts.user_context_id \
             where \
                 user_contexts.ip = $1 and \
                 login_attempts.created_at > now() - interval '1 hour'",
            ip,
        )
        .fetch_one(&self.db)
        .await
        .map_err(Status::db)?
        .unwrap_or(0);

        if num_attempts >= 10 {
            return Err(Status::coded(
                Code::ResourceExhausted,
                ErrorCode::TooManyLoginAttempts,
            ));
        }

        Ok(())
    }

    /// Add a new login attempt to the database
    async fn create_login_attempt(
        &self,
        user_id: i64,
        user_context: &UserContext,
        status: LoginAttemptStatus,
    ) -> Result<RawLoginAttempt, Status> {
        let ip = Self::parse_ip(&user_context.ip)?;

        let login_attempt = sqlx::query_as!(
            RawLoginAttempt,
            "with ctx as ( \
                 insert into auth_core.user_contexts (ip, user_agent) \
                 values ($2, $3) \
                 on conflict (ip, user_agent) do nothing \
                 returning id \
             ) \
             insert into auth_core.login_attempts (user_id, user_context_id, status) \
             select $1, id, $4 from ctx \
             returning *",
            user_id,
            ip,
            user_context.user_agent.as_str(),
            status as i32,
        )
        .fetch_one(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(login_attempt)
    }

    async fn create_session(
        &self,
        user_id: i64,
        login_attempt_id: Option<i64>,
        user_context_id: i64,
    ) -> Result<RawSession, Status> {
        let access_token = nanoid!(32);
        let expires_at = Utc::now() + TimeDelta::days(14);

        let session = sqlx::query_as!(
            RawSession,
            "insert into auth_core.sessions ( \
                 user_id, login_attempt_id, last_user_context_id, access_token, expires_at \
             ) \
             values ($1, $2, $3, $4, $5) \
             returning *",
            user_id,
            login_attempt_id,
            user_context_id,
            access_token,
            expires_at,
        )
        .fetch_one(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(session)
    }
}
