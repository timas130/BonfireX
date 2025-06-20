mod date_time;

tonic::include_proto!("bfx");

pub mod auth {
    tonic::include_proto!("bfx.auth");
}
pub mod router_registry {
    tonic::include_proto!("bfx.router_registry");
}
