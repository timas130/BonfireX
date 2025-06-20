mod methods;

use bfx_core::logging::setup_logging;
use bfx_core::service::start_service;
use bfx_proto::markdown::markdown_server::{Markdown, MarkdownServer};
use bfx_proto::markdown::{ParseMarkdownReply, ParseMarkdownRequest};
use tonic::{Request, Response, Status};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();

    let service = MarkdownService {};

    start_service(MarkdownServer::new(service)).await?;

    Ok(())
}

struct MarkdownService {}

#[tonic::async_trait]
impl Markdown for MarkdownService {
    async fn parse_markdown(
        &self,
        request: Request<ParseMarkdownRequest>,
    ) -> Result<Response<ParseMarkdownReply>, Status> {
        self.parse_markdown(request)
    }
}
