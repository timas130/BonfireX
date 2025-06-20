use crate::context::ServiceFactory;
use crate::data_loader;
use crate::error::RespError;
use crate::services::profile::user_profile::GProfile;
use async_graphql::dataloader::Loader;
use bfx_proto::profile::get_profile_request::Request;
use bfx_proto::profile::profile_client::ProfileClient;
use bfx_proto::profile::{GetProfileBulkRequest, GetProfileRequest};
use std::collections::HashMap;

data_loader!(ProfileLoader);

// key is GetProfileRequest: (user_id, for_user_id)
impl Loader<(i64, Option<i64>)> for ProfileLoader {
    type Value = GProfile;
    type Error = RespError;

    async fn load(
        &self,
        keys: &[(i64, Option<i64>)],
    ) -> Result<HashMap<(i64, Option<i64>), Self::Value>, Self::Error> {
        let mut profile: ProfileClient<_> = self.ctx.service();

        Ok(profile
            .get_profile_bulk(GetProfileBulkRequest {
                requests: keys
                    .iter()
                    .map(|&(user_id, for_user_id)| GetProfileRequest {
                        request: Some(Request::UserId(user_id)),
                        for_user_id,
                    })
                    .collect(),
            })
            .await?
            .into_inner()
            .profiles
            .into_iter()
            .filter_map(|r| {
                r.request
                    .and_then(|req| r.profile.map(|profile| (req, profile)))
            })
            .map(|(req, profile)| ((profile.user_id, req.for_user_id), profile.into()))
            .collect())
    }
}
