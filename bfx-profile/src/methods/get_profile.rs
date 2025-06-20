use crate::ProfileService;
use crate::models::profile::RawProfile;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::profile::{GetProfileReply, GetProfileRequest, get_profile_request};
use tonic::{Code, Request, Response, Status};

impl ProfileService {
    pub async fn get_profile(
        &self,
        request: Request<GetProfileRequest>,
    ) -> Result<Response<GetProfileReply>, Status> {
        let request = request.into_inner();

        let (user_id, username) = match request.request {
            Some(get_profile_request::Request::UserId(user_id)) => (Some(user_id), None),
            Some(get_profile_request::Request::Username(username)) => (None, Some(username)),
            None => return Err(Status::coded(Code::InvalidArgument, ErrorCode::Internal)),
        };

        let profile = sqlx::query_as!(
            RawProfile,
            "select p.*, n.note as \"note?\"
             from profile.profiles p
             left join profile.notes n on n.profile_id = p.user_id and n.user_id = $3
             where p.user_id = $1 or p.username = $2
             limit 1",
            user_id,
            username,
            request.for_user_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?
        .map(Into::into);

        Ok(Response::new(GetProfileReply { profile }))
    }
}
