use crate::service::router::get_router_endpoint;
use tonic::transport::Channel;

/// Get a lazy connection to the router service
///
/// # Errors
///
/// - If the router endpoint URI is invalid
pub fn require_router() -> anyhow::Result<Channel> {
    Ok(get_router_endpoint()?.connect_lazy())
}
