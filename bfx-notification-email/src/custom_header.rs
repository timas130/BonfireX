use lettre::message::header::{Header, HeaderName, HeaderValue};
use std::error::Error as StdError;

type BoxError = Box<dyn StdError + Send + Sync>;

#[derive(Clone)]
pub struct CustomHeader(pub &'static str, pub String);

impl Header for CustomHeader {
    fn name() -> HeaderName {
        unimplemented!()
    }

    fn parse(_s: &str) -> Result<Self, BoxError> {
        unimplemented!()
    }

    fn display(&self) -> HeaderValue {
        HeaderValue::new(HeaderName::new_from_ascii_str(self.0), self.1.clone())
    }
}
