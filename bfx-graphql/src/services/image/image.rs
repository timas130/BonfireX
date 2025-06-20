use crate::models::blob::Blob;
use async_graphql::SimpleObject;
use bfx_graphql_derive::object_ext;
use bfx_proto::image::{ImageDesc, ImageVariant};
use o2o::o2o;

/// An image reference
#[derive(Clone, o2o)]
#[from_owned(ImageDesc)]
pub struct GImage {
    #[from(@)]
    pub inner: ImageDesc,
}

/// An image reference
#[object_ext(name = "Image")]
impl GImage {
    /// Unique ID for this image
    ///
    /// If an image is changed, this ID will change.
    id!(inner.id => id, Image);

    /// Maximum available size for the image
    async fn full(&self) -> Option<GImageVariant> {
        self.inner.full.clone().map(From::from)
    }

    /// A thumbnail of the image
    ///
    /// The smaller side of the thumbnail will be under 512 px, but that's not guaranteed.
    async fn thumbnail(&self) -> Option<GImageVariant> {
        self.inner.thumbnail.clone().map(From::from)
    }

    /// A very small image blob that can be used to display a blurred image while it's loading
    async fn blur_data(&self) -> Blob<Vec<u8>> {
        Blob(self.inner.blur_data.clone())
    }
}

/// An image format variant
#[derive(SimpleObject, o2o)]
#[from_owned(ImageVariant)]
#[graphql(name = "ImageVariant")]
pub struct GImageVariant {
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Image size in bytes
    pub size: u64,
    /// URL to the image
    pub url: String,
}
