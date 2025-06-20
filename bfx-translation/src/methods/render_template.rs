use crate::TranslationService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::param_value::ParamValue;
use bfx_proto::translation::{RenderTemplateReply, RenderTemplateRequest, TranslateRequest};
use minijinja::value::{Kwargs, ValueKind};
use minijinja::{Environment, ErrorKind, Value};
use std::collections::HashMap;
use tonic::{Code, Request, Response, Status};
use tracing::warn;

impl TranslationService {
    /// Render a template with the given context and language
    ///
    /// # Errors
    ///
    /// - If the template parsing fails
    /// - If the template rendering fails
    /// - If translation lookups fail
    ///
    /// # Panics
    ///
    /// - If the template calls `t()` weirdly (should not happen)
    pub fn render_template(
        &self,
        request: Request<RenderTemplateRequest>,
    ) -> Result<Response<RenderTemplateReply>, Status> {
        let request = request.into_inner();

        let env = self.create_jinja_env(request.lang_id);
        let template = env.template_from_str(&request.source).map_err(|err| {
            warn!(%err, "template parsing failed");
            Status::coded(Code::InvalidArgument, ErrorCode::InvalidTemplate).with_source(err)
        })?;

        let context = Self::convert_context(request.context);

        let output = template.render(context).map_err(|err| {
            warn!(%err, "failed to render template");
            Status::coded(Code::Internal, ErrorCode::TemplateRenderingFailed).with_source(err)
        })?;

        Ok(Response::new(RenderTemplateReply { output }))
    }

    pub(crate) fn create_jinja_env(&self, lang_id: String) -> Environment<'_> {
        let mut env = Environment::new();

        let self0 = self.clone();
        env.add_function("t", move |key: String, kwargs: Kwargs| {
            let kwargs = kwargs.args().map(|key| (key, kwargs.get(key).unwrap()));

            let params = kwargs
                .map(|(key, value): (_, Value)| {
                    (
                        key.to_string(),
                        match value.kind() {
                            ValueKind::Number => ParamValue::Number(value.as_i64().unwrap()).into(),
                            ValueKind::String => ParamValue::String(value.to_string()).into(),
                            _ => ParamValue::String("[???]".to_string()).into(),
                        },
                    )
                })
                .collect();

            Ok(self0
                .translate(Request::new(TranslateRequest {
                    key,
                    lang: lang_id.to_string(),
                    params,
                }))
                .map_err(|err| minijinja::Error::new(ErrorKind::InvalidOperation, err.to_string()))?
                .into_inner()
                .text)
        });

        env
    }

    pub(crate) fn convert_context(
        context: HashMap<String, bfx_proto::ParamValue>,
    ) -> HashMap<String, Value> {
        context
            .into_iter()
            .map(|(key, value)| {
                (
                    key,
                    value.param_value.map_or_else(
                        || Value::from(""),
                        |val| match val {
                            ParamValue::String(string) => Value::from(string),
                            ParamValue::Number(number) => Value::from(number),
                        },
                    ),
                )
            })
            .collect::<HashMap<_, _>>()
    }
}
