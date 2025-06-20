use bfx_proto::router_registry::router_registry_client::RouterRegistryClient;
use bfx_proto::router_registry::{HealthPingRequest, RegisterServiceRequest, RegistrationInfo};
use std::net::SocketAddr;
use std::process;
use tonic::Code;
use tonic::server::NamedService;
use tonic::transport::{Channel, Endpoint};
use tracing::{error, info, warn};

async fn get_route_registry_service() -> anyhow::Result<RouterRegistryClient<Channel>> {
    let uri = std::env::var("ROUTER_URI").unwrap_or_else(|_| "grpc://127.0.0.1:5000".to_string());

    let endpoint = Endpoint::from_shared(uri)?;

    Ok(RouterRegistryClient::connect(endpoint).await?)
}

pub async fn register_service<S: NamedService>(local_addr: SocketAddr) {
    let registration_info = RegistrationInfo {
        service_name: S::NAME.to_string(),
        address: format!("grpc://{local_addr}"),
    };

    let mut registry = loop {
        let registry = get_route_registry_service().await;
        let Ok(mut registry) = registry else {
            warn!(
                err = ?registry.unwrap_err(),
                "failed to connect to router, retrying in 3 seconds"
            );
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            continue;
        };

        let registration_result = registry
            .register_service(RegisterServiceRequest {
                info: Some(registration_info.clone()),
            })
            .await;
        if let Err(err) = registration_result {
            warn!(?err, "failed to register service, retrying in 3 seconds");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            continue;
        }

        break registry;
    };

    #[rustfmt::skip]
    info!(
        registration_info.service_name,
        registration_info.address,
        "service registered on router"
    );
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    loop {
        let ping_result = registry
            .health_ping(HealthPingRequest {
                info: Some(registration_info.clone()),
            })
            .await;
        if let Err(err) = ping_result {
            if err.code() == Code::NotFound {
                error!("failed to renew router registration, exiting");
                process::exit(1);
            } else {
                warn!(?err, "failed to ping router, retrying in 1 second");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    }
}
