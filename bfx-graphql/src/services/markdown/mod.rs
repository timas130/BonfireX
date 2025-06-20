use crate::context::ServiceFactory;
use crate::error::RespError;
use crate::models::blob::Blob;
use async_graphql::{ComplexObject, Context, SimpleObject};
use bfx_proto::markdown::ParseMarkdownRequest;
use bfx_proto::markdown::markdown_client::MarkdownClient;
use prost::Message;

/// Some text that can be formatted with Markdown
#[derive(SimpleObject, Clone)]
#[graphql(complex)]
pub struct Markdown {
    /// Raw Markdown source of the text
    pub raw_text: String,
    /// Whether the Markdown is parsed as inline or not
    pub inline: bool,
}

impl Markdown {
    #[must_use]
    pub const fn new(raw_text: String) -> Self {
        Self {
            raw_text,
            inline: false,
        }
    }

    #[must_use]
    pub const fn new_inline(raw_text: String) -> Self {
        Self {
            raw_text,
            inline: true,
        }
    }
}

#[ComplexObject]
impl Markdown {
    /// Parsed AST of the Markdown text in protobuf format
    async fn parsed(&self, ctx: &Context<'_>) -> Result<Blob<Vec<u8>>, RespError> {
        let mut markdown: MarkdownClient<_> = ctx.service();

        // fixme: double encoding. oh well!
        let parsed = markdown
            .parse_markdown(ParseMarkdownRequest {
                text: self.raw_text.clone(),
            })
            .await?
            .into_inner()
            .parsed
            .ok_or_else(RespError::missing_field)?
            .encode_to_vec();

        Ok(Blob(parsed))
    }
}
