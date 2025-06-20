use crate::AuthCoreService;
use crate::models::user::RawUser;
use bfx_core::status::StatusExt;
use bfx_proto::auth::{GetUserByEmailReply, GetUserByEmailRequest};
use tonic::{Request, Response, Status};

impl AuthCoreService {
    /// Get a user by their email
    ///
    /// # Errors
    ///
    /// - Miscellaneous internal errors
    pub async fn get_user_by_email(
        &self,
        request: Request<GetUserByEmailRequest>,
    ) -> Result<Response<GetUserByEmailReply>, Status> {
        let request = request.into_inner();

        let user = sqlx::query_as!(
            RawUser,
            "select *
             from auth_core.users
             where email = $1",
            request.email
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?
        .map(Into::into);

        Ok(Response::new(GetUserByEmailReply { user }))
    }
}
