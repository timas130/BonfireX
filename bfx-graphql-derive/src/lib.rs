extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::{ImplItem, ImplItemMacro, ItemImpl, PathArguments, parse_macro_input, parse_quote};
use unsynn::{Comma, DotDelimitedVec, FatArrow, IParse, ToTokens, TokenIter, unsynn};

#[proc_macro_attribute]
pub fn complex_object_ext(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_impl = parse_macro_input!(input as ItemImpl);

    let args = proc_macro2::TokenStream::from(args);
    item_impl.attrs.push(parse_quote! {
        #[::async_graphql::ComplexObject(#args)]
    });

    object_impl(&mut item_impl)
}

#[proc_macro_attribute]
pub fn object_ext(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_impl = parse_macro_input!(input as ItemImpl);

    let args = proc_macro2::TokenStream::from(args);
    item_impl.attrs.push(parse_quote! {
        #[::async_graphql::Object(#args)]
    });

    object_impl(&mut item_impl)
}

macro_rules! parse_params {
    ($ty:ty, $mac:ident, $macro_segment:ident) => {
        match $mac.mac.tokens.to_token_iter().parse::<$ty>() {
            Ok(params) => params,
            Err(err) => {
                return syn::Error::new($macro_segment.span(), format!("invalid argument: {err}"))
                    .to_compile_error()
                    .into();
            }
        }
    };
}

fn object_impl(item_impl: &mut ItemImpl) -> TokenStream {
    for item in &mut item_impl.items {
        let ImplItem::Macro(mac) = item else {
            continue;
        };

        let path = &mac.mac.path;
        if path.leading_colon.is_some() || path.segments.len() != 1 {
            continue;
        }

        let macro_segment = path.segments.last().unwrap();
        if !matches!(macro_segment.arguments, PathArguments::None) {
            continue;
        }
        let macro_name = macro_segment.ident.to_string();

        match macro_name.as_str() {
            "id" => {
                let params = parse_params!(IdParams, mac, macro_segment);

                *item = id_macro(mac, params);
            }
            "optional_id" => {
                let params = parse_params!(IdParams, mac, macro_segment);

                *item = optional_id_macro(mac, params);
            }
            "image" => {
                let field = mac.mac.tokens.clone().into();
                let field = parse_macro_input!(field as Ident);

                *item = image_macro(mac, &field);
            }
            "optional_image" => {
                let field = mac.mac.tokens.clone().into();
                let field = parse_macro_input!(field as Ident);

                *item = optional_image_macro(mac, &field);
            }
            "user" => {
                let params = parse_params!(ModelParams, mac, macro_segment);

                *item = user_macro(mac, params);
            }
            other => {
                return syn::Error::new(
                    macro_segment.span(),
                    format!("`{other}` is not a valid bfx-graphql-derive macro"),
                )
                .to_compile_error()
                .into();
            }
        }
    }

    quote::ToTokens::into_token_stream(item_impl).into()
}

fn id_macro(mac: &ImplItemMacro, params: IdParams) -> ImplItem {
    let attrs = &mac.attrs;
    let method_name = params.method_name;
    let field_name = params.struct_field_name.into_token_stream();
    let id_type = params.id_type;

    parse_quote! {
        #(#attrs)*
        async fn #method_name(
            &self,
            ctx: &::async_graphql::Context<'_>
        ) -> ::async_graphql::ID {
            use crate::id_encryption::IdEncryptor;
            ctx.encrypt_id(
                ::bfx_core::service::id_encryption::IdType::#id_type,
                self.#field_name,
            )
        }
    }
}

fn optional_id_macro(mac: &ImplItemMacro, params: IdParams) -> ImplItem {
    let attrs = &mac.attrs;
    let method_name = params.method_name;
    let field_name = params.struct_field_name.into_token_stream();
    let id_type = params.id_type;

    parse_quote! {
        #(#attrs)*
        async fn #method_name(
            &self,
            ctx: &::async_graphql::Context<'_>
        ) -> Option<::async_graphql::ID> {
            use crate::id_encryption::IdEncryptor;
            self.#field_name.map(|id| ctx.encrypt_id(
                ::bfx_core::service::id_encryption::IdType::#id_type,
                id,
            ))
        }
    }
}

fn image_macro(mac: &ImplItemMacro, field: &Ident) -> ImplItem {
    let attrs = &mac.attrs;

    parse_quote! {
        #(#attrs)*
        async fn #field(
            &self,
            ctx: &::async_graphql::Context<'_>,
        ) -> Result<crate::services::image::image::GImage, crate::error::RespError> {
            let loader = ctx.data_unchecked::<::async_graphql::dataloader::DataLoader<
                crate::services::image::data_loaders::ImageLoader,
            >>();
            loader.load_one(#field).await
        }
    }
}

fn optional_image_macro(mac: &ImplItemMacro, field: &Ident) -> ImplItem {
    let attrs = &mac.attrs;

    parse_quote! {
        #(#attrs)*
        async fn #field(
            &self,
            ctx: &::async_graphql::Context<'_>,
        ) -> Result<Option<crate::services::image::image::GImage>, crate::error::RespError> {
            if let Some(#field) = self.#field {
                let loader = ctx.data_unchecked::<::async_graphql::dataloader::DataLoader<
                    crate::services::image::data_loaders::ImageLoader,
                >>();
                loader.load_one(#field).await
            } else {
                Ok(None)
            }
        }
    }
}

fn user_macro(mac: &ImplItemMacro, params: ModelParams) -> ImplItem {
    let attrs = &mac.attrs;
    let method_name = params.method_name;
    let field_name = params.struct_field_name.into_token_stream();

    parse_quote! {
        #(#attrs)*
        async fn #method_name(
            &self,
            ctx: &::async_graphql::Context<'_>,
        ) -> Result<crate::models::user::GUser, crate::error::RespError> {
            crate::models::user::GUser::from_id(ctx, self.#field_name)
                .await?
                .ok_or_else(crate::error::RespError::out_of_sync)
        }
    }
}

unsynn! {
    struct IdParams {
        pub struct_field_name: DotDelimitedVec<Ident>,
        pub fat_arrow: FatArrow,
        pub method_name: Ident,
        pub comma: Comma,
        pub id_type: Ident,
    }

    struct ModelParams {
        pub struct_field_name: DotDelimitedVec<Ident>,
        pub fat_arrow: FatArrow,
        pub method_name: Ident,
    }
}
