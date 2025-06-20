use crate::registry::RegistryMap;
use bfx_proto::router_registry::router_registry_server::RouterRegistry;
use bfx_proto::router_registry::{
    HealthPingRequest, HealthPingResponse, RegisterServiceReply, RegisterServiceRequest,
    RegistrationInfo, UnregisterServiceReply, UnregisterServiceRequest,
};
use std::sync::Arc;
use tonic::transport::Endpoint;
use tonic::{Request, Response, Status};

pub struct RouterRegistryService {
    registry: Arc<RegistryMap>,
}

impl RouterRegistryService {
    pub const fn new(registry: Arc<RegistryMap>) -> Self {
        Self { registry }
    }
}

fn parse_registration_info(info: Option<RegistrationInfo>) -> Result<(String, Endpoint), Status> {
    let info = info.ok_or_else(|| Status::invalid_argument("missing info"))?;
    let addr = info
        .address
        .parse()
        .map_err(|_| Status::invalid_argument("invalid address"))?;
    Ok((info.service_name, addr))
}

#[tonic::async_trait]
impl RouterRegistry for RouterRegistryService {
    async fn register_service(
        &self,
        request: Request<RegisterServiceRequest>,
    ) -> Result<Response<RegisterServiceReply>, Status> {
        let request = request.into_inner();
        let (service_name, addr) = parse_registration_info(request.info)?;

        self.registry.register(service_name, addr);
        Ok(Response::new(RegisterServiceReply {}))
    }

    async fn health_ping(
        &self,
        request: Request<HealthPingRequest>,
    ) -> Result<Response<HealthPingResponse>, Status> {
        let request = request.into_inner();
        let (service_name, addr) = parse_registration_info(request.info)?;

        let result = self.registry.ping(service_name, addr);
        if result {
            Ok(Response::new(HealthPingResponse {}))
        } else {
            Err(Status::not_found("service not registered"))
        }
    }

    async fn unregister_service(
        &self,
        request: Request<UnregisterServiceRequest>,
    ) -> Result<Response<UnregisterServiceReply>, Status> {
        let request = request.into_inner();
        let (service_name, addr) = parse_registration_info(request.info)?;

        self.registry.unregister(service_name, addr);

        Ok(Response::new(UnregisterServiceReply {}))
    }
}
