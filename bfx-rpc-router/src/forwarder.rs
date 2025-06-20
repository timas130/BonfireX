use crate::registry::RegistryMap;
use http::{Request, Response, Uri};
use moka::future::Cache;
use rand::prelude::IndexedRandom;
use std::convert::Infallible;
use std::future::ready;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::body::Body;
use tonic::server::NamedService;
use tonic::transport::{Channel, Endpoint};
use tower::{Service, ServiceExt};
use tracing::warn;

#[derive(Clone)]
pub struct ForwardingService {
    registry: Arc<RegistryMap>,
    channel_cache: Cache<Uri, Channel>,
}
impl ForwardingService {
    pub fn new(registry: Arc<RegistryMap>) -> Self {
        Self {
            registry,
            channel_cache: Cache::new(1024),
        }
    }

    async fn get_channel(&self, addr: &Endpoint) -> Result<Channel, tonic::transport::Error> {
        let cached = self.channel_cache.get(addr.uri()).await;
        if let Some(cached) = cached {
            return Ok(cached);
        }

        let channel = addr.connect().await?;
        self.channel_cache
            .insert(addr.uri().clone(), channel.clone())
            .await;

        Ok(channel)
    }
}

impl Service<Request<Body>> for ForwardingService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let error_resp = |status: u16| {
            Response::builder()
                .status(status)
                .body(Body::empty())
                .unwrap()
        };

        let mut path = req.uri().path().split('/');
        let _ = path.next();

        let service_name = path.next();
        let Some(service_name) = service_name else {
            return Box::pin(ready(Ok(error_resp(400))));
        };

        let endpoints = self.registry.get_endpoints(service_name);
        let endpoint = endpoints.choose(&mut rand::rng());
        let Some(endpoint) = endpoint else {
            return Box::pin(ready(Ok(error_resp(503))));
        };

        let endpoint = endpoint.clone();
        let self_clone = self.clone();

        Box::pin(async move {
            let channel = self_clone.get_channel(&endpoint).await;
            let Ok(channel) = channel else {
                warn!(
                    endpoint = ?endpoint.uri(),
                    error = ?channel.unwrap_err(),
                    "failed to connect to endpoint"
                );
                return Ok(error_resp(503));
            };

            let resp = channel.oneshot(req).await;
            let Ok(resp) = resp else {
                warn!(
                    endpoint = ?endpoint.uri(),
                    error = ?resp.unwrap_err(),
                    "failed to forward request"
                );
                return Ok(error_resp(503));
            };

            Ok(resp)
        })
    }
}

impl NamedService for ForwardingService {
    const NAME: &'static str = "ForwardingService";
}
