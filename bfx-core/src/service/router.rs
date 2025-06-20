use bfx_proto::router_registry::router_registry_client::RouterRegistryClient;
use bfx_proto::router_registry::{HealthPingRequest, RegisterServiceRequest, RegistrationInfo};
use std::net::SocketAddr;
use tonic::Code;
use tonic::server::NamedService;
use tonic::transport::{Channel, Endpoint};
use tracing::{info, warn};

/// Get the endpoint where the router registry can be reached
///
/// This function either reads the `ROUTER_URI` environment variable
/// for a connection string, or uses `grpc://127.0.0.1:5000` as a default.
///
/// # Errors
///
/// If the environment variable URI couldn't be parsed.
pub fn get_router_endpoint() -> anyhow::Result<Endpoint> {
    let uri = std::env::var("ROUTER_URI").unwrap_or_else(|_| "grpc://127.0.0.1:5000".to_string());

    Ok(Endpoint::from_shared(uri)?)
}

/// Keep a service registered on the router registry forever
///
/// # Arguments
///
/// - `local_addr` is the address where the service can be reached by the router registry.
/// - The service name comes from `S::NAME` ([`NamedService::NAME`]).
pub async fn register_service<S: NamedService>(local_addr: SocketAddr) -> ! {
    let registration_info = RegistrationInfo {
        service_name: S::NAME.to_string(),
        address: format!("grpc://{local_addr}"),
    };

    loop {
        let mut registry = first_register_service(&registration_info).await;

        #[rustfmt::skip]
        info!(
            registration_info.service_name,
            registration_info.address,
            "service registered on router"
        );

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        try_ping_loop(&registration_info, &mut registry).await;
    }
}

async fn get_route_registry_service() -> anyhow::Result<RouterRegistryClient<Channel>> {
    Ok(RouterRegistryClient::connect(get_router_endpoint()?).await?)
}

/// Register a service for the first time
async fn first_register_service(info: &RegistrationInfo) -> RouterRegistryClient<Channel> {
    loop {
        let registry = get_route_registry_service().await;
        let Ok(mut registry) = registry else {
            warn!(
                err = %registry.unwrap_err(),
                "failed to connect to router, retrying in 3 seconds"
            );
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            continue;
        };

        let registration_result = registry
            .register_service(RegisterServiceRequest {
                info: Some(info.clone()),
            })
            .await;
        if let Err(err) = registration_result {
            warn!(%err, "failed to register service, retrying in 3 seconds");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            continue;
        }

        break registry;
    }
}

/// Keep renewing router registry registration
///
/// # Returns
///
/// When this function returns, [`first_register_service`] should be run again.
async fn try_ping_loop(info: &RegistrationInfo, client: &mut RouterRegistryClient<Channel>) {
    loop {
        let ping_result = client
            .health_ping(HealthPingRequest {
                info: Some(info.clone()),
            })
            .await;
        if let Err(err) = ping_result {
            if err.code() == Code::NotFound {
                warn!("failed to renew router registration, reregistering");
                return;
            }

            warn!(%err, "failed to ping router, retrying in 1 second");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }

        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    }
}
