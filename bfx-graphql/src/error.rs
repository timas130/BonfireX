use async_graphql::ErrorExtensionValues;
use bfx_core::status::ErrorCode;
use std::str::FromStr;
use tonic::{Code, Status};

#[derive(Clone, Debug)]
pub struct RespError {
    code: Option<Code>,
    error_code: ErrorCode,
    message: String,
    source: Option<String>,
}

impl RespError {
    #[cold]
    #[must_use]
    pub fn missing_field() -> Self {
        Self {
            code: Some(Code::Unknown),
            error_code: ErrorCode::Internal,
            message: "backend returned missing field".to_string(),
            source: None,
        }
    }

    #[cold]
    #[must_use]
    pub fn out_of_sync() -> Self {
        Self {
            code: Some(Code::Unknown),
            error_code: ErrorCode::Internal,
            message: "backend out of sync".to_string(),
            source: None,
        }
    }
}

impl From<RespError> for async_graphql::Error {
    fn from(value: RespError) -> Self {
        let mut err = Self::new(value.message);
        err.extensions = Some({
            let mut extensions = ErrorExtensionValues::default();
            extensions.set("code", value.error_code.to_string());
            if let Some(code) = value.code {
                extensions.set("status_code", code as i32);
            }
            if let Some(source) = value.source {
                extensions.set("source", source);
            }
            extensions
        });
        err
    }
}

impl From<Status> for RespError {
    fn from(value: Status) -> Self {
        let message = value.message();

        let mut message_split = message.splitn(2, ": ");
        let error_code = message_split
            .next()
            .and_then(|code| ErrorCode::from_str(code).ok());
        let error_source = message_split.next().take_if(|_| error_code.is_some());

        Self {
            code: Some(value.code()),
            error_code: error_code.unwrap_or(ErrorCode::Internal),
            message: message.to_string(),
            source: error_source.map(Into::into),
        }
    }
}

impl<T> From<Box<T>> for RespError
where
    T: Into<Self>,
{
    fn from(value: Box<T>) -> Self {
        (*value).into()
    }
}
