use crate::services::image::request_upload::RequestUploadMutation;
use async_graphql::MergedObject;

pub mod data_loaders;
#[allow(clippy::module_inception)]
pub mod image;
mod request_upload;

#[derive(Default, MergedObject)]
pub struct ImageQuery;

#[derive(Default, MergedObject)]
pub struct ImageMutation(RequestUploadMutation);
