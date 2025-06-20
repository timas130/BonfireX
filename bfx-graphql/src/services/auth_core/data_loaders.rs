use crate::context::ServiceFactory;
use crate::data_loader;
use crate::error::RespError;
use async_graphql::dataloader::Loader;
use bfx_proto::auth::auth_core_client::AuthCoreClient;
use bfx_proto::auth::{GetUsersByIdsRequest, User};
use std::collections::HashMap;

data_loader!(UserLoader);

impl Loader<i64> for UserLoader {
    type Value = User;
    type Error = RespError;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let mut auth_core: AuthCoreClient<_> = self.ctx.service();

        let users = auth_core
            .get_users_by_ids(GetUsersByIdsRequest { ids: keys.to_vec() })
            .await?
            .into_inner()
            .users
            .into_iter()
            .map(|user| (user.id, user))
            .collect();

        Ok(users)
    }
}
