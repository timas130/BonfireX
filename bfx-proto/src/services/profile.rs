use crate::ext_impl;
use crate::profile::profile_client::ProfileClient;
use crate::profile::{GetProfileRequest, ProfileDetails, get_profile_request};
use tonic::Status;

ext_impl!(ProfileClient, {
    pub async fn get_profile_by_id(&mut self, id: i64) -> Result<Option<ProfileDetails>, Status> {
        Ok(self
            .get_profile(GetProfileRequest {
                request: Some(get_profile_request::Request::UserId(id)),
                for_user_id: None,
            })
            .await?
            .into_inner()
            .profile)
    }
});
