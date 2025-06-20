use bfx_core::service::database::Db;
use bfx_proto::auth::auth_core_server::AuthCore;
use bfx_proto::auth::{
    ChangePasswordReply, ChangePasswordRequest, CreateUserReply, CreateUserRequest,
    GetUserByEmailReply, GetUserByEmailRequest, GetUserByTokenReply, GetUserByTokenRequest,
    GetUsersByIdsReply, GetUsersByIdsRequest, LoginEmailReply, LoginEmailRequest,
    LoginExternalReply, LoginExternalRequest, SendVerificationEmailReply,
    SendVerificationEmailRequest, VerifyEmailReply, VerifyEmailRequest,
};
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

mod methods;
pub mod models;
mod util;

#[derive(Debug)]
pub struct AuthCoreService {
    pub db: Db,
    pub router: Channel,

    pub frontend_root: String,
}

#[tonic::async_trait]
impl AuthCore for AuthCoreService {
    async fn get_users_by_ids(
        &self,
        request: Request<GetUsersByIdsRequest>,
    ) -> Result<Response<GetUsersByIdsReply>, Status> {
        self.get_users_by_ids(request).await
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        self.create_user(request).await
    }

    async fn login_email(
        &self,
        request: Request<LoginEmailRequest>,
    ) -> Result<Response<LoginEmailReply>, Status> {
        self.login_email(request).await
    }

    async fn get_user_by_token(
        &self,
        request: Request<GetUserByTokenRequest>,
    ) -> Result<Response<GetUserByTokenReply>, Status> {
        self.get_user_by_token(request).await
    }

    async fn send_verification_email(
        &self,
        request: Request<SendVerificationEmailRequest>,
    ) -> Result<Response<SendVerificationEmailReply>, Status> {
        self.send_verification_email(request).await
    }

    async fn verify_email(
        &self,
        request: Request<VerifyEmailRequest>,
    ) -> Result<Response<VerifyEmailReply>, Status> {
        self.verify_email(request).await
    }

    async fn change_password(
        &self,
        request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordReply>, Status> {
        self.change_password(request).await
    }

    async fn get_user_by_email(
        &self,
        request: Request<GetUserByEmailRequest>,
    ) -> Result<Response<GetUserByEmailReply>, Status> {
        self.get_user_by_email(request).await
    }

    async fn login_external(
        &self,
        request: Request<LoginExternalRequest>,
    ) -> Result<Response<LoginExternalReply>, Status> {
        self.login_external(request).await
    }
}
