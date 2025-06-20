//! Utilities for working with [`Status`]

use sqlx::Error;
use strum::Display;
use tonic::{Code, Status};
use tracing::warn;

/// Extension trait for converting common error types into gRPC Status
pub trait StatusExt {
    fn coded(code: Code, error: ErrorCode) -> Self;

    fn db(err: Error) -> Self
    where
        Self: Sized,
    {
        warn!(?err, "database error");
        Self::coded(Code::Internal, ErrorCode::Database)
    }

    fn anyhow(err: anyhow::Error) -> Self
    where
        Self: Sized,
    {
        warn!(?err, "internal error");
        Self::coded(Code::Internal, ErrorCode::Internal)
    }
}

impl StatusExt for Status {
    fn coded(code: Code, error: ErrorCode) -> Self {
        Self::new(code, format!("{error}"))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Display)]
pub enum ErrorCode {
    Database,
    Internal,
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
}
