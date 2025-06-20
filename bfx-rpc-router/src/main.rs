mod forwarder;
mod registry;
mod registry_service;

use crate::forwarder::ForwardingService;
use crate::registry::RegistryMap;
use bfx_core::logging::setup_logging;
use bfx_core::service::get_tcp_listener;
use bfx_proto::router_registry::router_registry_server::RouterRegistryServer;
use http::Request;
use registry_service::RouterRegistryService;
use std::sync::Arc;
use tonic::body::Body;
use tonic::service::{AxumBody, Routes};
use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;
use tower::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();

    let registry = Arc::new(RegistryMap::default());

    let service = RouterRegistryService::new(registry.clone());
    let forwarder = ForwardingService::new(registry);

    let listener = get_tcp_listener().await?;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(bfx_proto::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    let router = Routes::default()
        .add_service(reflection_service)
        .add_service(RouterRegistryServer::new(service))
        .into_axum_router()
        .fallback_service(forwarder.map_request(|req: Request<AxumBody>| req.map(Body::new)));

    Server::builder()
        .add_routes(router.into())
        .serve_with_incoming(TcpIncoming::from(listener))
        .await?;

    Ok(())
}
