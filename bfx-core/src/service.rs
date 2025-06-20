//! Utilities for running a service

pub mod client;
pub mod database;
pub mod environment;
#[cfg(feature = "id_encryption")]
pub mod id_encryption;
mod router;
#[cfg(feature = "s3")]
pub mod s3;

use crate::service::router::register_service;
use std::convert::Infallible;
use std::env;
use std::net::{Ipv6Addr, SocketAddr};
use tokio::net::TcpListener;
use tonic::body::Body;
use tonic::codegen::Service;
use tonic::codegen::http::Request;
use tonic::server::NamedService;
use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;
use tracing::{debug, info};

/// Creates and binds a TCP listener using the port specified in the `PORT`
/// environment variable
///
/// If the variable is not set (or is not a number), the OS assigns the port randomly.
///
/// # Errors
///
/// - If the TCP listener fails to bind
pub async fn get_tcp_listener() -> Result<TcpListener, tokio::io::Error> {
    let port = env::var("PORT")
        .map_err(|_| "PORT environment variable not set")
        .and_then(|port| {
            port.parse::<u16>()
                .map_err(|_| "PORT environment variable is not a number")
        });

    let socket: SocketAddr = match port {
        Ok(port) => (Ipv6Addr::UNSPECIFIED, port).into(),
        Err(reason) => {
            debug!("could not get listening port from environment: {reason}");
            (Ipv6Addr::UNSPECIFIED, 0).into()
        }
    };

    let listener = TcpListener::bind(socket).await?;
    info!(addr = ?listener.local_addr()?, "listening");
    Ok(listener)
}

/// Registers and starts a gRPC service
///
/// # Errors
///
/// - If the TCP listener fails to bind (see [`get_tcp_listener`])
/// - If getting the local address of the TCP listener fails (unlikely)
/// - If the service fails to start
pub async fn start_service<S>(server: S) -> anyhow::Result<()>
where
    S: Service<Request<Body>, Error = Infallible> + NamedService + Clone + Send + Sync + 'static,
    S::Response: axum::response::IntoResponse,
    S::Future: Send + 'static,
{
    let listener = get_tcp_listener().await?;

    let local_addr = listener.local_addr()?;
    let registration_task = tokio::spawn(register_service::<S>(local_addr));

    Server::builder()
        .add_service(server)
        .serve_with_incoming(TcpIncoming::from(listener))
        .await?;

    registration_task.abort();

    Ok(())
}
