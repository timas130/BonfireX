//! Utilities for working with [`Status`]

use sqlx::Error as SqlxError;
use std::error::Error as StdError;
use std::str::FromStr;
use strum::{Display, EnumString};
use tonic::{Code, Status};
use tracing::warn;

/// Extension trait for converting common error types into gRPC Status
pub trait StatusExt {
    #[must_use]
    fn coded(code: Code, error: ErrorCode) -> Self;

    #[must_use]
    fn db(err: SqlxError) -> Self
    where
        Self: Sized,
    {
        warn!(?err, "database error");
        Self::coded(Code::Internal, ErrorCode::Database).with_source(err)
    }

    #[must_use]
    fn anyhow(err: anyhow::Error) -> Self
    where
        Self: Sized,
    {
        warn!(?err, "internal error");
        Self::coded(Code::Internal, ErrorCode::Internal)
    }

    #[must_use]
    fn with_source<T: StdError + Send + Sync + 'static>(self, err: T) -> Self;

    #[must_use]
    fn with_details(self, err: &str) -> Self;

    #[must_use]
    fn to_error_code(&self) -> Option<ErrorCode>;
}

impl StatusExt for Status {
    fn coded(code: Code, error: ErrorCode) -> Self {
        Self::new(code, format!("{error}"))
    }

    fn with_source<T: StdError + Send + Sync + 'static>(self, err: T) -> Self {
        Self::new(self.code(), format!("{}: {err}", self.message()))
    }

    fn with_details(self, err: &str) -> Self {
        Self::new(self.code(), format!("{}: {err}", self.message()))
    }

    fn to_error_code(&self) -> Option<ErrorCode> {
        let code = self.message().split(':').next()?;
        ErrorCode::from_str(code).ok()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Display, EnumString)]
pub enum ErrorCode {
    Database,
    Internal,
    AccessDenied,
    EmailExists,
    UserNotFound,
    PasswordNotSet,
    IncorrectPassword,
    EmailInvalid,
    WeakPassword,
    TooManyLoginAttempts,
    UserNotActive,
    UserBanned,
    InvalidToken,
    ExpiredToken,
    InvalidLanguage,
    LanguageNotFound,
    TranslationKeyNotFound,
    TranslationErrors,
    InvalidEmailContents,
    InvalidTemplate,
    TemplateRenderingFailed,
    UserAlreadyActive,
    TooManyRequests,
    EmailCodeNotFound,
    EmailCodeExpired,
    TicketNotFound,
    ImageNotUploaded,
    ImageNotFound,
    ProfileNotFound,
    DisplayNameTooLong,
    BioTooLong,
    UsernameTaken,
    InvalidUsernameCharacters,
    UsernameTooShort,
    UsernameTooLong,
    NoteTooLong,
    RecoveryTokenNotFound,
    RecoveryTokenUsed,
    RecoveryTokenExpired,
    InvalidId,
    InvalidNotificationDefinition,
    NoTemplateMatched,
    UnknownProvider,
    FlowNotFound,
    ProviderError,
    ProviderEmailError,
    OAuthAlreadyBound,
    AuthSourceNotFound,
    MissingParameter,
    InvalidParameter,
    ClientNotFound,
    InvalidRedirectUri,
    InvalidScope,
}
