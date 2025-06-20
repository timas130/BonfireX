use crate::AuthOAuthProviderService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{GetJwkSetReply, GetJwkSetRequest};
use openidconnect::PrivateSigningKey;
use openidconnect::core::CoreJsonWebKeySet;
use tonic::{Code, Request, Response, Status};

impl AuthOAuthProviderService {
    /// `/openid/jwks` endpoint
    ///
    /// # Errors
    ///
    /// - Miscellaneous internal errors
    pub fn get_jwk_set(
        &self,
        _request: Request<GetJwkSetRequest>,
    ) -> Result<Response<GetJwkSetReply>, Status> {
        let jwks = CoreJsonWebKeySet::new(vec![self.rs_256_signing_key.as_verification_key()]);

        Ok(Response::new(GetJwkSetReply {
            json: serde_json::to_string(&jwks).map_err(|err| {
                Status::coded(Code::Internal, ErrorCode::Internal).with_source(err)
            })?,
        }))
    }
}
