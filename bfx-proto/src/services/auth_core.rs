use crate::auth::auth_core_client::AuthCoreClient;
use crate::auth::{GetUsersByIdsRequest, User};
use crate::ext_impl;
use tonic::Status;

ext_impl!(AuthCoreClient, {
    pub async fn get_user_by_id(&mut self, id: i64) -> Result<Option<User>, Status> {
        Ok(self
            .get_users_by_ids(GetUsersByIdsRequest { ids: vec![id] })
            .await?
            .into_inner()
            .users
            .into_iter()
            .next())
    }
});
