use crate::TranslationService;
use crate::lang_id_ext::LanguageIdentifierExt;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::param_value::ParamValue;
use bfx_proto::translation::{TranslateReply, TranslateRequest};
use fluent::{FluentArgs, FluentValue};
use std::collections::HashMap;
use tonic::{Code, Request, Response, Status};

impl TranslationService {
    /// Translate a message key with the given parameters
    ///
    /// # Errors
    ///
    /// - If the language identifier is invalid
    /// - If the language is not found
    /// - If the translation key is not found
    /// - If there are translation formatting errors
    pub fn translate(
        &self,
        request: Request<TranslateRequest>,
    ) -> Result<Response<TranslateReply>, Status> {
        let request = request.into_inner();

        let lang_id = request
            .lang
            .parse()
            .map_err(|_| Status::coded(Code::InvalidArgument, ErrorCode::InvalidLanguage))?;

        let bundle = self
            .bundles
            .get(&lang_id)
            .or_else(|| self.bundles.get(&lang_id.only_region()))
            .or_else(|| self.bundles.get(&lang_id.only_script()))
            .or_else(|| self.bundles.get(&lang_id.only_language()))
            .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::LanguageNotFound))?;

        let message = bundle.bundle.get_message(&request.key).ok_or_else(|| {
            Status::coded(Code::InvalidArgument, ErrorCode::TranslationKeyNotFound)
        })?;
        let value = message.value().ok_or_else(|| {
            Status::coded(Code::InvalidArgument, ErrorCode::TranslationKeyNotFound)
        })?;

        let mut args = FluentArgs::new();
        for (key, value) in request.params {
            let param_value = value
                .param_value
                .ok_or_else(|| Status::coded(Code::InvalidArgument, ErrorCode::Internal))?;

            args.set(
                key,
                match param_value {
                    ParamValue::String(string) => FluentValue::from(string),
                    ParamValue::Number(number) => FluentValue::from(number),
                },
            );
        }

        let mut errors = vec![];
        let text = bundle
            .bundle
            .format_pattern(value, Some(&args), &mut errors)
            .into_owned();

        let mut attributes = HashMap::new();
        for attr in message.attributes() {
            attributes.insert(
                attr.id().to_string(),
                bundle
                    .bundle
                    .format_pattern(attr.value(), Some(&args), &mut errors)
                    .into_owned(),
            );
        }

        drop(bundle);

        if !errors.is_empty() {
            return Err(Status::coded(
                Code::InvalidArgument,
                ErrorCode::TranslationErrors,
            ));
        }

        Ok(Response::new(TranslateReply { text, attributes }))
    }
}
