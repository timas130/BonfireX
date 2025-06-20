use crate::AuthOAuthProviderService;
use crate::models::client_info::ClientInfo;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::auth::{GetAuthorizationInfoReply, GetAuthorizationInfoRequest};
use nanoid::nanoid;
use openidconnect::RedirectUrl;
use openidconnect::core::{CoreAuthErrorResponseType, CoreResponseType};
use serde_json::Value;
use tonic::{Code, Request, Response, Status};
use tracing::info;

#[rustfmt::skip]
macro_rules! missing_param {
    (val $name:literal) => {{
        use tonic::Code;
        use bfx_core::status::{ErrorCode, StatusExt};
        Status::coded(Code::InvalidArgument, ErrorCode::MissingParameter)
            .with_details(concat!("missing parameter `", $name, "`"))
    }};
    ($name:literal) => {
        || missing_param!(val $name)
    };
}

#[rustfmt::skip]
macro_rules! invalid_param {
    (val $name:literal) => {{
        use tonic::Code;
        use bfx_core::status::{ErrorCode, StatusExt};
        Status::coded(Code::InvalidArgument, ErrorCode::InvalidParameter)
            .with_details(concat!("invalid value for parameter `", $name, "`"))
    }};
    ($name:literal) => {
        |_| invalid_param!(val $name)
    };
}

macro_rules! get_param {
    ($query:expr, $name:literal) => {
        $query.remove($name).ok_or_else(missing_param!($name))?
    };
}

macro_rules! is_subset {
    ($a:expr, $b:expr) => {
        $a.iter().all(|x| $b.iter().any(|y| x == y))
    };
}

pub(crate) use {get_param, invalid_param, is_subset, missing_param};

// also applies to nonce
const MAX_STATE_LEN: usize = 256;

const SUPPORTED_SCOPES: &[&str] = &["openid", "email", "profile", "offline_access"];

impl AuthOAuthProviderService {
    /// Get information for displaying a page on `/openid/authorize`
    ///
    /// # Errors
    ///
    /// - If some param is missing or invalid
    /// - If `response_type` is not `code`
    /// - If one of the `scope`s is not supported
    /// - If `client_id` does not exist
    /// - If `code_challenge_method` is not `S256` (if required or specified)
    pub async fn get_authorization_info(
        &self,
        request: Request<GetAuthorizationInfoRequest>,
    ) -> Result<Response<GetAuthorizationInfoReply>, Status> {
        let request = request.into_inner();

        let mut query = request.query;

        // gather parameters
        let scope = get_param!(query, "scope");
        let response_type = get_param!(query, "response_type");
        let client_id = get_param!(query, "client_id");
        let redirect_uri = query.remove("redirect_uri");
        let state = query.remove("state");
        let nonce = query.remove("nonce");
        let code_challenge = query.remove("code_challenge");
        let code_challenge_method = query.remove("code_challenge_method");
        let prompt = query.remove("prompt");

        // parse and validate some parameters
        let scopes = scope.split(' ').collect::<Vec<_>>();
        let response_type: CoreResponseType = serde_json::from_value(Value::String(response_type))
            .map_err(invalid_param!("response_type"))?;

        //// check param length
        if state.as_ref().map_or(0, String::len) > MAX_STATE_LEN {
            return Err(invalid_param!(val "state"));
        }
        if nonce.as_ref().map_or(0, String::len) > MAX_STATE_LEN {
            return Err(invalid_param!(val "nonce"));
        }

        //// check response type
        if response_type != CoreResponseType::Code {
            return Err(
                Status::coded(Code::InvalidArgument, ErrorCode::InvalidParameter)
                    .with_details("response_type must be `code`"),
            );
        }

        //// check scopes
        // the spec says that provider SHOULD ignore unsupported scopes,
        // but for now we'll return an error
        if scopes.iter().any(|scope| !SUPPORTED_SCOPES.contains(scope)) {
            return Err(
                Status::coded(Code::InvalidArgument, ErrorCode::InvalidParameter)
                    .with_details("unsupported scope"),
            );
        }

        // get client
        let client = sqlx::query_as!(
            ClientInfo,
            "select * from auth_oauth_provider.clients where client_id = $1",
            client_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?
        .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::ClientNotFound))?;

        // check redirect_uri
        let redirect_uri = if let Some(redirect_uri) = redirect_uri {
            if !client.redirect_uris.contains(&redirect_uri) {
                return Err(Status::coded(
                    Code::InvalidArgument,
                    ErrorCode::InvalidRedirectUri,
                ));
            }
            RedirectUrl::new(redirect_uri).map_err(invalid_param!("redirect_uri"))?
        } else if client.redirect_uris.len() == 1 {
            RedirectUrl::new(client.redirect_uris[0].clone())
                .map_err(invalid_param!("redirect_uri"))?
        } else {
            return Err(missing_param!(val "redirect_uri"));
        };

        // check scopes
        let disallowed_scope = scopes.iter().find(|scope| {
            !client
                .allowed_scopes
                .iter()
                .any(|allowed_scope| allowed_scope == **scope)
        });
        if let Some(disallowed_scope) = disallowed_scope {
            return Err(
                Status::coded(Code::InvalidArgument, ErrorCode::InvalidScope)
                    .with_details(&format!("disallowed scope `{disallowed_scope}`")),
            );
        }

        // check code_challenge
        if client.enforce_code_challenge && code_challenge.is_none() {
            return Err(missing_param!(val "code_challenge"));
        }

        if let Some(code_challenge) = &code_challenge {
            if code_challenge_method.as_deref() != Some("S256") {
                return Err(invalid_param!(val "code_challenge_method"));
            }

            if code_challenge.len() != 43 {
                return Err(invalid_param!(val "code_challenge"));
            }
        }

        //// all checks passed
        // prompt=none is MTI
        let default_redirect_to = if prompt.as_deref() == Some("none") {
            Some(self.make_error_redirect(
                &redirect_uri,
                state.as_deref(),
                CoreAuthErrorResponseType::InteractionRequired,
            ))
        } else {
            None
        };

        let Some(user_id) = request.user_id else {
            // if not logged in, just should the rp info
            return Ok(Response::new(GetAuthorizationInfoReply {
                rp_info: Some(client.into()),
                scopes: scopes.into_iter().map(String::from).collect(),
                flow_id: None,
                redirect_to: default_redirect_to,
            }));
        };

        let existing_grant = sqlx::query!(
            "select * from auth_oauth_provider.grants where client_id = $1 and user_id = $2",
            client.id,
            user_id,
        )
        .fetch_optional(&self.db)
        .await
        .map_err(Status::db)?;

        match existing_grant {
            Some(grant) if is_subset!(scopes, grant.scopes) => {
                info!(grant.id, "authorized client with an existing grant");

                let code = format!("BF/C/{}", nanoid!(32));

                let flow = sqlx::query!(
                    "insert into auth_oauth_provider.flows
                     (client_id, grant_id, user_id, redirect_uri, scopes, state, nonce,
                      code_challenge, code_challenge_method, code)
                     values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                     returning id",
                    client.id,
                    grant.id,
                    user_id,
                    redirect_uri.as_str(),
                    &scopes as &[&str],
                    state,
                    nonce,
                    code_challenge,
                    code_challenge_method,
                    code,
                )
                .fetch_one(&self.db)
                .await
                .map_err(Status::db)?;

                Ok(Response::new(GetAuthorizationInfoReply {
                    rp_info: Some(client.into()),
                    scopes: scopes.into_iter().map(String::from).collect(),
                    flow_id: Some(flow.id),
                    redirect_to: Some(self.make_code_redirect(
                        &redirect_uri,
                        &code,
                        state.as_deref(),
                    )),
                }))
            }
            grant => {
                let flow = sqlx::query!(
                    "insert into auth_oauth_provider.flows
                     (client_id, grant_id, user_id, redirect_uri, scopes, state, nonce,
                      code_challenge, code_challenge_method)
                     values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                     returning id",
                    client.id,
                    grant.map(|g| g.id),
                    user_id,
                    redirect_uri.as_str(),
                    &scopes as &[&str],
                    state,
                    nonce,
                    code_challenge,
                    code_challenge_method,
                )
                .fetch_one(&self.db)
                .await
                .map_err(Status::db)?;

                Ok(Response::new(GetAuthorizationInfoReply {
                    rp_info: Some(client.into()),
                    scopes: scopes.into_iter().map(String::from).collect(),
                    flow_id: Some(flow.id),
                    redirect_to: default_redirect_to,
                }))
            }
        }
    }

    #[must_use]
    pub fn make_code_redirect(
        &self,
        redirect_url: &RedirectUrl,
        code: &str,
        state: Option<&str>,
    ) -> String {
        let mut url = redirect_url.url().clone();

        url.query_pairs_mut()
            .append_pair("code", code)
            .append_pair("iss", &self.issuer);

        if let Some(state) = state {
            url.query_pairs_mut().append_pair("state", state);
        }

        url.to_string()
    }

    // CoreAuthErrorResponseType is cheap in terms of memory, who cares
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn make_error_redirect(
        &self,
        redirect_url: &RedirectUrl,
        state: Option<&str>,
        error: CoreAuthErrorResponseType,
    ) -> String {
        let mut url = redirect_url.url().clone();

        url.query_pairs_mut()
            .append_pair("error", error.as_ref())
            .append_pair("iss", &self.issuer);

        if let Some(state) = state {
            url.query_pairs_mut().append_pair("state", state);
        }

        url.to_string()
    }
}
