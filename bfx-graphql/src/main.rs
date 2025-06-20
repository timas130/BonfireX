use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Extension, Router};
use axum_client_ip::{ClientIp, ClientIpSource};
use axum_extra::TypedHeader;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::{Authorization, UserAgent};
use bfx_core::logging::setup_logging;
use bfx_core::service::client::require_router;
use bfx_core::service::environment::require_env;
use bfx_core::service::get_tcp_listener;
use bfx_core::service::id_encryption::require_id_encryptor;
use bfx_graphql::context::{GlobalContext, LocalContext};
use bfx_graphql::language::{AcceptLanguage, DEFAULT_LANGUAGE};
use bfx_graphql::schema::GSchema;
use bfx_graphql::services::auth_core::data_loaders::UserLoader;
use bfx_graphql::services::auth_oauth_provider::get_jwk_set::get_jwk_set;
use bfx_graphql::services::auth_oauth_provider::get_openid_metadata::get_openid_metadata;
use bfx_graphql::services::auth_oauth_provider::token_endpoint::token_endpoint;
use bfx_graphql::services::auth_oauth_provider::userinfo_endpoint::{
    userinfo_endpoint_get, userinfo_endpoint_post,
};
use bfx_graphql::services::image::data_loaders::ImageLoader;
use bfx_graphql::services::profile::data_loaders::ProfileLoader;
use bfx_proto::UserContext;
use bfx_proto::auth::GetUserByTokenRequest;
use bfx_proto::auth::auth_core_client::AuthCoreClient;
use ipnet::IpNet;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tonic::Response;

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/").finish())
}

async fn graphql_handler(
    Extension(context): Extension<GlobalContext>,
    Extension(schema): Extension<GSchema>,
    ClientIp(ip): ClientIp,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    accept_language: Option<TypedHeader<AcceptLanguage>>,
    authorization: Option<TypedHeader<Authorization<Bearer>>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    // authenticate the request
    let user = if let Some(authorization) = authorization {
        let token = authorization.token().to_string();

        let mut auth_core = AuthCoreClient::new(context.router.clone());
        auth_core
            .get_user_by_token(GetUserByTokenRequest {
                access_token: token,
            })
            .await
            .ok()
            .map(Response::into_inner)
    } else {
        None
    };

    let local_context = LocalContext {
        user_context: UserContext {
            ip: IpNet::from(ip).to_string(),
            user_agent: user_agent.to_string(),
            lang_id: accept_language.map_or_else(
                || DEFAULT_LANGUAGE.to_string(),
                |al| al.best_match().to_string(),
            ),
        },
        user,
    };

    // execution
    let req = req.into_inner().data(local_context);
    schema.execute(req).await.into()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let context = GlobalContext {
        router: require_router()?,
        id_encryptor: Arc::new(require_id_encryptor()?),
    };

    #[allow(clippy::default_trait_access)]
    let schema = GSchema::build(Default::default(), Default::default(), Default::default())
        .data(context.clone())
        .data(UserLoader::data_loader(context.clone()))
        .data(ProfileLoader::data_loader(context.clone()))
        .data(ImageLoader::data_loader(context.clone()))
        .finish();

    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .route(
            "/.well-known/openid-configuration",
            get(get_openid_metadata),
        )
        .route("/openid/jwks", get(get_jwk_set))
        .route("/openid/token", post(token_endpoint))
        .route(
            "/openid/userinfo",
            get(userinfo_endpoint_get).post(userinfo_endpoint_post),
        )
        .layer(Extension(context))
        .layer(Extension(schema))
        .layer(ClientIpSource::from_str(&require_env("CLIENT_IP_SOURCE")?)?.into_extension());

    let listener = get_tcp_listener().await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
