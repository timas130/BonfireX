mod auth_core;
mod image;
pub mod notification_email;
mod profile;
mod translation;

#[macro_export]
#[doc(hidden)]
macro_rules! ext_impl {
    ($ty:ident, { $($code:item)* }) => {
        impl<T> $ty<T>
        where
            T: tonic::client::GrpcService<tonic::body::Body>,
            T::Error: Into<tonic::codegen::StdError>,
            T::ResponseBody: tonic::codegen::Body<Data = tonic::codegen::Bytes> + std::marker::Send + 'static,
            <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + std::marker::Send,
        {
            $( $code )*
        }
    };
}
