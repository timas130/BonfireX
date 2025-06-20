use crate::AuthCoreService;
use crate::models::user::RawUser;
use bfx_core::status::StatusExt;
use bfx_proto::auth::{GetUsersByIdsReply, GetUsersByIdsRequest, User};
use futures_util::TryStreamExt;
use tonic::{Request, Response, Status};

impl AuthCoreService {
    /// Get multiple users by their IDs
    ///
    /// # Errors
    ///
    /// - Miscellaneous internal errors
    pub async fn get_users_by_ids(
        &self,
        request: Request<GetUsersByIdsRequest>,
    ) -> Result<Response<GetUsersByIdsReply>, Status> {
        let user_ids = request.into_inner().ids;

        let users = sqlx::query_as!(
            RawUser,
            "select * \
             from auth_core.users \
             where id = any($1)",
            &user_ids,
        )
        .fetch(&self.db)
        .map_ok(Into::into)
        .try_collect::<Vec<User>>()
        .await
        .map_err(Status::db)?;

        Ok(Response::new(GetUsersByIdsReply { users }))
    }
}
