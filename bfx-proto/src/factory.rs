use crate::auth::auth_core_client::AuthCoreClient;
use crate::auth::auth_o_auth_client::AuthOAuthClient;
use crate::auth::auth_o_auth_provider_client::AuthOAuthProviderClient;
use crate::auth::password_recovery_client::PasswordRecoveryClient;
use crate::image::image_client::ImageClient;
use crate::markdown::markdown_client::MarkdownClient;
use crate::notification::email::notification_email_client::NotificationEmailClient;
use crate::notification::notification_client::NotificationClient;
use crate::profile::profile_client::ProfileClient;
use crate::router_registry::router_registry_client::RouterRegistryClient;
use crate::translation::translation_client::TranslationClient;
use tonic::transport::Channel;

pub trait BuildableService {
    fn new(router: Channel) -> Self;
}

macro_rules! impl_buildable_service {
    ($service:ident) => {
        impl BuildableService for $service<Channel> {
            fn new(router: Channel) -> Self {
                Self::new(router)
            }
        }
    };
    ($($service:ident),* $(,)?) => {
        $(
            impl_buildable_service!($service);
        )*
    }
}

impl_buildable_service!(
    AuthCoreClient,
    PasswordRecoveryClient,
    NotificationEmailClient,
    ImageClient,
    ProfileClient,
    RouterRegistryClient,
    TranslationClient,
    NotificationClient,
    AuthOAuthClient,
    AuthOAuthProviderClient,
    MarkdownClient,
);
