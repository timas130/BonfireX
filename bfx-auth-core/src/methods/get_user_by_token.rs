use crate::AuthCoreService;
use crate::models::user::RawUser;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::UserContext;
use bfx_proto::auth::{GetUserByTokenReply, GetUserByTokenRequest, Session};
use tonic::{Code, Request, Response, Status};

impl AuthCoreService {
    /// Get a user by their access token
    ///
    /// # Errors
    ///
    /// - If the access token is invalid or expired
    /// - Miscellaneous internal errors
    pub async fn get_user_by_token(
        &self,
        request: Request<GetUserByTokenRequest>,
    ) -> Result<Response<GetUserByTokenReply>, Status> {
        let request = request.into_inner();

        let token = request.access_token;

        let session = sqlx::query!(
            "select
                 u.*,
                 u.id as user_id,
                 s.id as session_id,
                 s.created_at as session_created_at,
                 s.expires_at as session_expires_at,
                 uc.ip,
                 uc.user_agent,
                 uc.lang_id
             from auth_core.sessions s
             inner join auth_core.users u on u.id = s.user_id
             inner join auth_core.user_contexts uc on uc.id = s.last_user_context_id
             where access_token = $1 and expires_at > now()",
            token,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?;

        let Some(session) = session else {
            return Err(Status::coded(
                Code::PermissionDenied,
                ErrorCode::InvalidToken,
            ));
        };

        if !session.active {
            return Err(Status::coded(
                Code::PermissionDenied,
                ErrorCode::UserNotActive,
            ));
        }

        if session.banned {
            return Err(Status::coded(Code::PermissionDenied, ErrorCode::UserBanned));
        }

        Ok(Response::new(GetUserByTokenReply {
            user: Some(
                RawUser {
                    id: session.user_id,
                    email: session.email,
                    permission_level: session.permission_level,
                    banned: session.banned,
                    active: session.active,
                    email_verification_sent_at: session.email_verification_sent_at,
                    email_verification_code: session.email_verification_code,
                    password: session.password,
                    created_at: session.created_at,
                }
                .into(),
            ),
            session: Some(Session {
                id: session.session_id,
                user_id: session.user_id,
                user_context: Some(UserContext {
                    ip: session.ip.to_string(),
                    user_agent: session.user_agent,
                    lang_id: session.lang_id,
                }),
                expires_at: Some(session.session_expires_at.into()),
                created_at: Some(session.session_created_at.into()),
            }),
        }))
    }
}
