use crate::ProfileService;
use crate::models::profile::RawProfile;
use bfx_core::status::StatusExt;
use bfx_proto::profile::{
    BulkProfileDetails, GetProfileBulkReply, GetProfileBulkRequest, GetProfileRequest,
    ProfileDetails, get_profile_request,
};
use futures_util::TryStreamExt;
use std::collections::HashMap;
use tonic::{Request, Response, Status};

#[derive(sqlx::Type)]
#[sqlx(type_name = "profile.profile_request")]
struct ProfileRequest {
    user_id: Option<i64>,
    username: Option<String>,
    for_user_id: Option<i64>,
}

impl ProfileService {
    pub async fn get_profile_bulk(
        &self,
        request: Request<GetProfileBulkRequest>,
    ) -> Result<Response<GetProfileBulkReply>, Status> {
        let request = request.into_inner();

        // convert requests to db type `profile.profile_request`
        let requests = request
            .requests
            .into_iter()
            .filter_map(|request| match request.request? {
                get_profile_request::Request::UserId(user_id) => Some(ProfileRequest {
                    user_id: Some(user_id),
                    username: None,
                    for_user_id: request.for_user_id,
                }),
                get_profile_request::Request::Username(username) => Some(ProfileRequest {
                    user_id: None,
                    username: Some(username),
                    for_user_id: request.for_user_id,
                }),
            })
            .collect::<Vec<_>>();

        // extract user ids and lowercased usernames from requests
        let user_ids = requests
            .iter()
            .filter_map(|r| r.user_id)
            .collect::<Vec<_>>();
        let usernames = requests
            .iter()
            .filter_map(|r| r.username.as_ref())
            .map(|u| u.to_lowercase())
            .collect::<Vec<_>>();

        // fetch all the profiles
        let profiles = sqlx::query_as!(
            RawProfile,
            "select
                 p.user_id as \"user_id!\",
                 p.display_name,
                 p.username as \"username!\",
                 p.avatar_id,
                 p.bio as \"bio!\",
                 p.cover_id,
                 p.created_at as \"created_at!\",
                 null as \"note\"
             from profile.profiles p
             where p.user_id = any($1) or lower(p.username) = any($2)",
            user_ids as Vec<i64>,
            usernames as Vec<String>,
        )
        .fetch(&self.db)
        .map_err(Status::db)
        .map_ok(|profile| (profile.user_id, profile.into()))
        .try_collect::<HashMap<_, ProfileDetails>>()
        .await?;

        // fetch the notes and join them with the profile_requests and profiles
        let result = sqlx::query!(
            "select
                 p.user_id,
                 pr.user_id as pr_user_id,
                 pr.username as pr_username,
                 pr.for_user_id,
                 n.note
             from unnest($1::profile.profile_request[]) pr
             inner join profile.profiles p on p.user_id = pr.user_id or lower(p.username) = lower(pr.username)
             left join profile.notes n on n.profile_id = p.user_id and n.user_id = pr.for_user_id",
            requests as Vec<ProfileRequest>,
        )
        .fetch(&self.db)
        .map_ok(|pr| {
            let mut profile = profiles.get(&pr.user_id)?.clone();
            profile.note = if pr.for_user_id.is_some() {
                Some(pr.note.unwrap_or_default())
            } else {
                None
            };

            Some(BulkProfileDetails {
                request: Some(GetProfileRequest {
                    request: Some(if let Some(user_id) = pr.pr_user_id {
                        get_profile_request::Request::UserId(user_id)
                    } else {
                        get_profile_request::Request::Username(
                            pr.pr_username.expect("pr_user_id and pr_username are null")
                        )
                    }),
                    for_user_id: pr.for_user_id,
                }),
                profile: Some(profile),
            })
        })
        .try_filter_map(|r| futures_util::future::ready(Ok(r)))
        .try_collect::<Vec<_>>()
        .await
        .map_err(Status::db)?;

        Ok(Response::new(GetProfileBulkReply { profiles: result }))
    }
}
