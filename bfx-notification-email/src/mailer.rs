use bfx_core::service::environment::require_env;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, Tokio1Executor};

pub type Mailer = AsyncSmtpTransport<Tokio1Executor>;

pub fn require_mailer() -> anyhow::Result<Mailer> {
    Ok(
        AsyncSmtpTransport::<Tokio1Executor>::relay(&require_env("EMAIL_HOST")?)?
            .credentials(Credentials::new(
                require_env("EMAIL_USER")?,
                require_env("EMAIL_PASSWORD")?,
            ))
            .build(),
    )
}
