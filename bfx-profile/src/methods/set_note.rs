use crate::ProfileService;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::profile::{GetProfileRequest, SetNoteReply, SetNoteRequest, get_profile_request};
use tonic::{Code, IntoRequest, Request, Response, Status};

const MAX_NOTE_LENGTH: usize = 250;

impl ProfileService {
    pub async fn set_note(
        &self,
        request: Request<SetNoteRequest>,
    ) -> Result<Response<SetNoteReply>, Status> {
        let request = request.into_inner();

        if let Err(err) = Self::check_note(&request.note) {
            return Err(Status::coded(Code::InvalidArgument, err));
        }

        let mut profile = self
            .get_profile(
                GetProfileRequest {
                    request: Some(get_profile_request::Request::UserId(request.profile_id)),
                    for_user_id: Some(request.user_id),
                }
                .into_request(),
            )
            .await?
            .into_inner()
            .profile
            .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::ProfileNotFound))?;

        sqlx::query!(
            "insert into profile.notes (user_id, profile_id, note)
             values ($1, $2, $3)
             on conflict (user_id, profile_id) do update set note = $3",
            request.user_id,
            request.profile_id,
            request.note,
        )
        .execute(&self.db)
        .await
        .map_err(Status::db)?;

        profile.note = Some(request.note);

        Ok(Response::new(SetNoteReply {
            profile: Some(profile),
        }))
    }

    const fn check_note(note: &str) -> Result<(), ErrorCode> {
        if note.len() > MAX_NOTE_LENGTH {
            Err(ErrorCode::NoteTooLong)
        } else {
            Ok(())
        }
    }
}
