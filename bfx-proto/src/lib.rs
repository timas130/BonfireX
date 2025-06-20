#![allow(clippy::all, clippy::pedantic, clippy::nursery)]

mod date_time;
pub mod factory;
mod param_value_util;
pub mod services;

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("file_descriptor_set");

tonic::include_proto!("bfx");

pub mod auth {
    tonic::include_proto!("bfx.auth");
}
pub mod router_registry {
    tonic::include_proto!("bfx.router_registry");
}
pub mod translation {
    tonic::include_proto!("bfx.translation");
}
pub mod notification {
    tonic::include_proto!("bfx.notification");
    pub mod email {
        tonic::include_proto!("bfx.notification.email");
    }
}
pub mod image {
    tonic::include_proto!("bfx.image");
}
pub mod profile {
    tonic::include_proto!("bfx.profile");
}
pub mod markdown {
    tonic::include_proto!("bfx.markdown");
}
