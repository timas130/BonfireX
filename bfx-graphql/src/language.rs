use axum::http::{HeaderName, HeaderValue};
use axum_extra::headers::{Error, Header};
use fluent_langneg::{
    LanguageIdentifier, NegotiationStrategy, accepted_languages, convert_vec_str_to_langids_lossy,
    negotiate_languages,
};
use itertools::Itertools;
use std::iter::once;

pub struct AcceptLanguage(Vec<LanguageIdentifier>);

impl Header for AcceptLanguage {
    fn name() -> &'static HeaderName {
        static HEADER_NAME: HeaderName = HeaderName::from_static("accept-language");
        &HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.last().ok_or_else(Error::invalid)?;
        Ok(Self(accepted_languages::parse(
            value.to_str().map_err(|_| Error::invalid())?,
        )))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        let value = HeaderValue::from_str(&self.0.iter().map(ToString::to_string).join(","));
        if let Ok(value) = value {
            values.extend(once(value));
        }
    }
}

pub const DEFAULT_LANGUAGE: &str = "en";

impl AcceptLanguage {
    /// Find the best language match for the client
    ///
    /// # Panics
    ///
    /// - If [`DEFAULT_LANGUAGE`] is not a valid language identifier (impossible)
    #[must_use]
    pub fn best_match(&self) -> LanguageIdentifier {
        let supported = convert_vec_str_to_langids_lossy(["en"]);
        let default = DEFAULT_LANGUAGE.parse().unwrap();

        let lang = negotiate_languages(
            &self.0,
            &supported,
            Some(&default),
            NegotiationStrategy::Filtering,
        );

        (*lang.first().unwrap()).clone()
    }
}
