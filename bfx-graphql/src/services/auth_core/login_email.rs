use crate::context::{ContextExt, ServiceFactory};
use crate::error::RespError;
use crate::id_encryption::IdEncryptor;
use async_graphql::{ComplexObject, Context, Enum, ID, Object, SimpleObject, Union};
use bfx_core::service::id_encryption::IdType;
use bfx_proto::auth::auth_core_client::AuthCoreClient;
use bfx_proto::auth::{LoginEmailRequest, TfaChallenge, TfaMethod, Tokens, login_email_reply};
use o2o::o2o;

#[derive(Default)]
pub struct LoginEmailMutation;

///
#[derive(Union, o2o)]
#[graphql(name = "LoginResult")]
#[from_owned(login_email_reply::LoginResult)]
pub enum GLoginResult {
    Tokens(#[from(~.into())] GLoginResultTokens),
    TfaChallenge(#[from(~.into())] GLoginResultTfaChallenge),
}

#[derive(SimpleObject, o2o)]
#[graphql(complex, name = "LoginResultTokens")]
#[from_owned(Tokens)]
pub struct GLoginResultTokens {
    pub access_token: String,
    #[graphql(skip)]
    pub session_id: i64,
    #[graphql(skip)]
    pub login_attempt_id: Option<i64>,
}
#[ComplexObject]
impl GLoginResultTokens {
    async fn session_id(&self, ctx: &Context<'_>) -> ID {
        ctx.encrypt_id(IdType::Session, self.session_id)
    }

    async fn login_attempt_id(&self, ctx: &Context<'_>) -> Option<ID> {
        self.login_attempt_id
            .map(|id| ctx.encrypt_id(IdType::LoginAttempt, id))
    }
}

#[derive(SimpleObject, o2o)]
#[graphql(name = "TfaChallenge")]
#[from_owned(TfaChallenge)]
pub struct GLoginResultTfaChallenge {
    pub tfa_wait_token: String,
    #[from(~.into_iter().filter_map(|m| TfaMethod::try_from(m).map(From::from).ok()).collect())]
    pub methods: Vec<GTfaMethod>,
}

#[derive(Enum, o2o, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[graphql(name = "TfaMethod")]
#[map(TfaMethod)]
pub enum GTfaMethod {
    EmailLink,
    Totp,
    RecoveryCode,
}

#[Object]
impl LoginEmailMutation {
    /// Log into an account with an email and password
    async fn login_email(
        &self,
        ctx: &Context<'_>,
        email: String,
        password: String,
    ) -> Result<GLoginResult, RespError> {
        let mut auth_core: AuthCoreClient<_> = ctx.service();

        let result = auth_core
            .login_email(LoginEmailRequest {
                email,
                password,
                user_context: Some(ctx.user_context().clone()),
            })
            .await?
            .into_inner();

        Ok(result
            .login_result
            .ok_or_else(RespError::missing_field)?
            .into())
    }
}
