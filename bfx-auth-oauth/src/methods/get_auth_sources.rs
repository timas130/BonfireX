use crate::AuthOAuthService;
use crate::models::raw_auth_source::RawAuthSource;
use bfx_core::status::StatusExt;
use bfx_proto::auth::{GetAuthSourcesReply, GetAuthSourcesRequest};
use tonic::{Request, Response, Status};

impl AuthOAuthService {
    /// Get the list of external auth sources for a user
    ///
    /// # Errors
    ///
    /// - Miscellaneous internal errors
    pub async fn get_auth_sources(
        &self,
        request: Request<GetAuthSourcesRequest>,
    ) -> Result<Response<GetAuthSourcesReply>, Status> {
        let request = request.into_inner();

        let auth_sources = sqlx::query_as!(
            RawAuthSource,
            "select * from auth_oauth.auth_sources
             where user_id = $1
             order by created_at",
            request.user_id,
        )
        .fetch_all(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(Response::new(GetAuthSourcesReply {
            auth_sources: auth_sources.into_iter().map(Into::into).collect(),
        }))
    }
}
