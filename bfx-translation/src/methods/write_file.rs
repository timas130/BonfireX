use crate::TranslationService;
use bfx_core::status::StatusExt;
use bfx_proto::translation::{WriteFileReply, WriteFileRequest};
use tonic::{Request, Response, Status};

impl TranslationService {
    /// Write a translation resource file to the database
    ///
    /// # Errors
    ///
    /// - If the database operation fails
    pub async fn write_file(
        &self,
        request: Request<WriteFileRequest>,
    ) -> Result<Response<WriteFileReply>, Status> {
        let request = request.into_inner();

        let resource_id = sqlx::query_scalar!(
            "insert into translation.resources (path, lang_id, source, modified_at)
             values ($1, $2, $3, now())
             on conflict (path)
             do update set
                 source = excluded.source,
                 modified_at = now()
             returning id",
            request.path,
            request.lang_id,
            request.source,
        )
        .fetch_one(&self.db)
        .await
        .map_err(Status::db)?;

        Ok(Response::new(WriteFileReply { id: resource_id }))
    }
}
