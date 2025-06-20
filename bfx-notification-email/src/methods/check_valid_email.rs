use crate::NotificationEmailService;
use bfx_core::status::StatusExt;
use bfx_proto::notification::email::{
    CheckValidEmailReply, CheckValidEmailRequest, EmailValidationError,
};
use email_address::{EmailAddress, Options};
use hickory_resolver::Name;
use std::str::FromStr;
use tonic::{Request, Response, Status};

impl NotificationEmailService {
    pub fn parse_email(email: &str) -> Result<EmailAddress, EmailValidationError> {
        #[allow(clippy::needless_update)]
        let address = EmailAddress::parse_with_options(
            email,
            Options {
                minimum_sub_domains: 2,
                allow_domain_literal: false,
                allow_display_text: false,
                // Options is not non_exhaustive, but let's future-proof
                ..Default::default()
            },
        );
        let Ok(address) = address else {
            return Err(EmailValidationError::ParsingError);
        };

        Ok(address)
    }

    pub async fn check_email(
        &self,
        address: &EmailAddress,
    ) -> Result<Option<EmailValidationError>, Status> {
        let email_blocked = sqlx::query!(
            "select id from notification_email.blocked_emails where email = $1",
            address.as_ref(),
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?
        .is_some();

        if email_blocked {
            return Ok(Some(EmailValidationError::BlockedByUser));
        }

        let domain = address.domain();
        let name = Name::from_str(domain);
        let Ok(name) = name else {
            return Ok(Some(EmailValidationError::ParsingError));
        };

        let email_domain_blocked = sqlx::query!(
            "select id from notification_email.blocked_email_domains
             where domain = any($1)
             limit 1",
            &[
                domain.to_owned(),
                name.to_ascii(),
                name.trim_to(2).to_ascii(),
                name.trim_to(3).to_ascii(),
            ]
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?
        .is_some();

        if email_domain_blocked || name.is_localhost() {
            return Ok(Some(EmailValidationError::BlockedDomainOrMx));
        }

        let mx_records = self.resolver.mx_lookup(domain).await;
        // fail silently if mx lookup fails
        if let Ok(records) = mx_records {
            let domains = records
                .into_iter()
                .flat_map(|record| {
                    let exchange = record.exchange();
                    [
                        exchange.to_ascii(),
                        exchange.trim_to(2).to_ascii(),
                        exchange.trim_to(3).to_ascii(),
                    ]
                })
                .collect::<Vec<_>>();

            let mx_domain_blocked = sqlx::query!(
                "select id from notification_email.blocked_email_domains
                 where domain = any($1) or (domain || '.') = any($1)
                 limit 1",
                &domains,
            )
            .fetch_optional(&self.db)
            .await
            .map_err(Status::db)?
            .is_some();

            if mx_domain_blocked {
                return Ok(Some(EmailValidationError::BlockedDomainOrMx));
            }
        }

        Ok(None)
    }

    pub async fn check_valid_email(
        &self,
        request: Request<CheckValidEmailRequest>,
    ) -> Result<Response<CheckValidEmailReply>, Status> {
        let request = request.into_inner();
        let email = request.email;

        let address = match Self::parse_email(&email) {
            Ok(address) => address,
            Err(error) => {
                return Ok(Response::new(CheckValidEmailReply {
                    valid: false,
                    error: Some(error.into()),
                }));
            }
        };

        let error = self.check_email(&address).await?;

        Ok(Response::new(CheckValidEmailReply {
            valid: error.is_none(),
            error: error.map(Into::into),
        }))
    }
}
