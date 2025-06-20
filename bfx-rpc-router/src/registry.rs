use arc_swap::ArcSwap;
use dashmap::{DashMap, Entry};
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tonic::transport::Endpoint;
use tracing::info;

/// A service endpoint with its socket address and last ping timestamp
struct ServiceEndpoint {
    addr: Endpoint,
    last_ping: ArcSwap<Instant>,
}
impl From<Endpoint> for ServiceEndpoint {
    fn from(value: Endpoint) -> Self {
        Self {
            addr: value,
            last_ping: ArcSwap::from_pointee(Instant::now()),
        }
    }
}
impl Hash for ServiceEndpoint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr.uri().hash(state);
    }
}
impl PartialEq for ServiceEndpoint {
    fn eq(&self, other: &Self) -> bool {
        self.addr.uri() == other.addr.uri()
    }
}
impl Eq for ServiceEndpoint {}

#[derive(Default)]
struct ServiceInfo {
    endpoints: HashSet<ServiceEndpoint>,
}

/// Registry of all services and their endpoints (including health check data)
#[derive(Default)]
pub struct RegistryMap {
    map: DashMap<String, ServiceInfo>,
}

impl RegistryMap {
    /// Registers a new service endpoint
    ///
    /// [`RegistrationInfo`]: bfx_proto::router_registry::RegistrationInfo
    pub fn register(&self, service_name: String, addr: Endpoint) {
        info!(service_name, addr = ?addr.uri(), "endpoint registered");

        self.map
            .entry(service_name)
            .or_default()
            .endpoints
            .insert(addr.into());
    }

    /// Removes a service endpoint from the registry
    pub fn unregister(&self, service_name: String, addr: Endpoint) {
        info!(service_name, addr = ?addr.uri(), "endpoint unregistered");

        self.map.entry(service_name).and_modify(|service| {
            service.endpoints.remove(&addr.into());
        });
    }

    /// Updates the last ping time for a service endpoint
    ///
    /// # Returns
    /// `true` if the service was updated, `false` if it doesn't exist.
    pub fn ping(&self, service_name: String, addr: Endpoint) -> bool {
        let entry = self.map.entry(service_name);
        match entry {
            Entry::Occupied(mut entry) => {
                let entry = entry.get_mut().endpoints.get(&addr.into());
                entry.is_some_and(|entry| {
                    entry.last_ping.store(Arc::new(Instant::now()));
                    true
                })
            }
            Entry::Vacant(_) => false,
        }
    }

    /// Retrieve all active endpoints for a service
    ///
    /// Removes endpoints that haven't pinged in the last 5 seconds
    pub fn get_endpoints(&self, service_name: &str) -> Vec<Endpoint> {
        self.map
            .get_mut(service_name)
            .map(|mut service| {
                let ping_deadline = Instant::now() - Duration::from_secs(5);

                service
                    .endpoints
                    .retain(|endpoint| **endpoint.last_ping.load() > ping_deadline);

                service
                    .endpoints
                    .iter()
                    .map(|endpoint| endpoint.addr.clone())
                    .collect()
            })
            .unwrap_or_default()
    }
}
