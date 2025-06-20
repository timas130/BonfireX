use crate::TranslationService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::translation::{RenderStringSetRequest, RenderTemplateReply};
use tonic::{Code, Request, Response, Status};

impl TranslationService {
    /// Render a set of conditional strings
    ///
    /// # Errors
    ///
    /// - If the template or conditional parsing fails
    /// - If the template rendering fails
    /// - If no template matches
    pub fn render_string_set(
        &self,
        request: Request<RenderStringSetRequest>,
    ) -> Result<Response<RenderTemplateReply>, Status> {
        let request = request.into_inner();

        let env = self.create_jinja_env(request.lang_id);
        let context = Self::convert_context(request.context);

        for conditional in request.conditionals {
            let matches = if let Some(if_) = &conditional.r#if {
                let expr = env.compile_expression(if_).map_err(|err| {
                    Status::coded(Code::InvalidArgument, ErrorCode::InvalidTemplate)
                        .with_source(err)
                })?;
                expr.eval(&context)
                    .map_err(|err| {
                        Status::coded(Code::InvalidArgument, ErrorCode::TemplateRenderingFailed)
                            .with_source(err)
                    })?
                    .is_true()
            } else {
                true
            };

            if matches {
                let template = env.template_from_str(&conditional.value).map_err(|err| {
                    Status::coded(Code::InvalidArgument, ErrorCode::InvalidTemplate)
                        .with_source(err)
                })?;
                let result = template.render(context).map_err(|err| {
                    Status::coded(Code::InvalidArgument, ErrorCode::TemplateRenderingFailed)
                        .with_source(err)
                })?;
                return Ok(RenderTemplateReply { output: result }.into());
            }
        }

        Err(Status::coded(
            Code::InvalidArgument,
            ErrorCode::NoTemplateMatched,
        ))
    }
}
