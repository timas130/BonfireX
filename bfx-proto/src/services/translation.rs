use crate::translation::translation_client::TranslationClient;
use crate::translation::{ConditionalString, RenderStringSetRequest, RenderTemplateRequest};
use crate::{ParamValue, ext_impl, param_value};
use std::collections::HashMap;
use tonic::Status;

ext_impl!(TranslationClient, {
    pub async fn render_template_ext(
        &mut self,
        template_source: &str,
        lang_id: String,
        params: HashMap<String, ParamValue>,
    ) -> Result<String, Status> {
        let resp = self
            .render_template(RenderTemplateRequest {
                source: template_source.to_string(),
                lang_id,
                context: params,
            })
            .await?
            .into_inner();

        Ok(resp.output)
    }

    pub async fn render_string_set_ext(
        &mut self,
        lang_id: String,
        conditionals: Vec<ConditionalString>,
        context: HashMap<String, ParamValue>,
    ) -> Result<String, Status> {
        let resp = self
            .render_string_set(RenderStringSetRequest {
                lang_id,
                conditionals: conditionals.into(),
                context,
            })
            .await?
            .into_inner();

        Ok(resp.output)
    }
});

// === convenience macro for creating map<string, ParamValue> ===

#[macro_export]
macro_rules! param_map {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key.into(), $value.into());
            )*
            map
        }
    };
}

// === conversions to ParamValue ===

macro_rules! impl_from_string {
    ($ty:ty) => {
        impl From<$ty> for ParamValue {
            fn from(s: $ty) -> Self {
                ParamValue {
                    param_value: Some(param_value::ParamValue::String(s.into())),
                }
            }
        }
    };
}

impl_from_string!(String);
impl_from_string!(&str);

macro_rules! impl_from_int {
    ($ty:ty) => {
        impl From<$ty> for ParamValue {
            fn from(i: $ty) -> Self {
                ParamValue {
                    param_value: Some(param_value::ParamValue::Number(i.into())),
                }
            }
        }
    };
}

impl_from_int!(i8);
impl_from_int!(i16);
impl_from_int!(i32);
impl_from_int!(i64);
impl_from_int!(u8);
impl_from_int!(u16);
impl_from_int!(u32);
