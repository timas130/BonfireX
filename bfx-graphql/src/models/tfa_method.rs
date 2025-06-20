use async_graphql::Enum;
use bfx_proto::auth::TfaMethod;
use o2o::o2o;

/// Two-factor authentication method
#[derive(Copy, Clone, Eq, PartialEq, Hash, Enum, o2o)]
#[graphql(name = "TfaMethod")]
#[from(TfaMethod)]
pub enum GTfaMethod {
    /// Magic email link
    EmailLink,
    /// Time-based one-time password
    Totp,
    /// Static recovery code
    RecoveryCode,
}
