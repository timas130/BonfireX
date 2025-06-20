use async_graphql::Enum;
use bfx_proto::auth::PermissionLevel;
use o2o::o2o;

/// The global level of permissions for a user
#[derive(Copy, Clone, Eq, PartialEq, Enum, o2o)]
#[graphql(name = "PermissionLevel")]
#[from(PermissionLevel)]
#[into(PermissionLevel)]
pub enum GPermissionLevel {
    /// A regular user
    User,
    /// A member of staff with admin powers
    Admin,
    /// A system user, not a person
    System,
}

impl From<i32> for GPermissionLevel {
    fn from(value: i32) -> Self {
        let permission_level = PermissionLevel::try_from(value).unwrap_or(PermissionLevel::User);
        permission_level.into()
    }
}
