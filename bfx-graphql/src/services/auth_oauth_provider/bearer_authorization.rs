//! RFC 6750 authorization

use axum::Form;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use o2o::o2o;
use serde::Deserialize;

const REJECTION_RESP: (StatusCode, [(&str, &str); 1]) =
    (StatusCode::UNAUTHORIZED, [("WWW-Authenticate", "Bearer")]);

pub struct OAuthBearerToken(pub String);

impl<S> FromRequestParts<S> for OAuthBearerToken
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, [(&'static str, &'static str); 1]);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // by header
        let by_header =
            Option::<TypedHeader<Authorization<Bearer>>>::from_request_parts(parts, state)
                .await
                .map_err(|_| REJECTION_RESP)?;

        if let Some(header) = by_header {
            return Ok(Self(header.token().into()));
        }

        // by query param
        let by_query = form_urlencoded::parse(parts.uri.query().unwrap_or_default().as_bytes())
            .find(|(k, _)| k == "access_token");

        if let Some((_, token)) = by_query {
            return Ok(Self(token.into_owned()));
        }

        Err(REJECTION_RESP)
    }
}

#[derive(o2o)]
#[map_owned(OAuthBearerToken)]
pub struct OAuthBearerTokenForm(pub String);

impl<S> FromRequest<S> for OAuthBearerTokenForm
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, [(&'static str, &'static str); 1]);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        #[derive(Deserialize)]
        struct BearerAuthForm {
            access_token: String,
        }

        // by header or query
        let (mut parts, body) = req.into_parts();
        let by_parts = OAuthBearerToken::from_request_parts(&mut parts, state).await;
        if let Ok(token) = by_parts {
            return Ok(Self(token.0));
        }

        // by form
        let form = Form::<BearerAuthForm>::from_request(Request::from_parts(parts, body), state)
            .await
            .map_err(|_| REJECTION_RESP)?;

        Ok(Self(form.0.access_token))
    }
}
