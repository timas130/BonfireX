use crate::ProfileService;
use crate::models::profile::RawProfile;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::image::image_client::ImageClient;
use bfx_proto::profile::{UpdateProfileReply, UpdateProfileRequest};
use tonic::{Code, Request, Response, Status};

const MAX_DISPLAY_NAME_LENGTH: usize = 40;
const MAX_BIO_LENGTH: usize = 1000;

impl ProfileService {
    pub async fn update_profile(
        &self,
        request: Request<UpdateProfileRequest>,
    ) -> Result<Response<UpdateProfileReply>, Status> {
        let request = request.into_inner();

        let UpdateProfileRequest {
            user_id,
            for_user_id,
            display_name,
            avatar_ticket,
            bio,
            cover_ticket,
        } = request;

        let mut tx = self.db.begin().await.map_err(Status::db)?;

        let mut profile = sqlx::query_as!(
            RawProfile,
            "select
                 p.*,
                 (select n.note
                  from profile.notes n
                  where n.profile_id = $1 and n.user_id = $2)
             from profile.profiles p
             where p.user_id = $1
             for update",
            user_id,
            for_user_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(Status::db)?
        .ok_or_else(|| Status::coded(Code::NotFound, ErrorCode::ProfileNotFound))?;

        let mut errors = vec![];

        // display name
        if let Some(name) = display_name {
            if let Err(err) = Self::check_display_name(&name) {
                errors.push(err.to_string());
            } else {
                profile.display_name = Some(name);
            }
        }

        // avatar
        if let Some(ticket) = avatar_ticket {
            self.replace_image(&mut profile.avatar_id, ticket, user_id)
                .await?;
        }

        // cover
        if let Some(ticket) = cover_ticket {
            self.replace_image(&mut profile.cover_id, ticket, user_id)
                .await?;
        }

        // bio
        if let Some(new_bio) = bio {
            if let Err(err) = Self::check_bio(&new_bio) {
                errors.push(err.to_string());
            } else {
                profile.bio = new_bio;
            }
        }

        sqlx::query!(
            "update profile.profiles
             set display_name = $1, avatar_id = $2, bio = $3, cover_id = $4
             where user_id = $5",
            profile.display_name,
            profile.avatar_id,
            profile.bio,
            profile.cover_id,
            user_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(Status::db)?;

        tx.commit().await.map_err(Status::db)?;

        Ok(Response::new(UpdateProfileReply {
            profile: Some(profile.into()),
            errors,
        }))
    }

    async fn replace_image(
        &self,
        image_id: &mut Option<i64>,
        ticket: String,
        user_id: i64,
    ) -> Result<(), Status> {
        let mut image = ImageClient::new(self.router.clone());

        let ref_id = format!("profile:{user_id}:avatar");
        let avatar = image
            .use_image_ticket_ext(ticket, user_id, ref_id.clone())
            .await?;
        image.set_image_ref_ext(*image_id, ref_id, false).await?;

        *image_id = Some(avatar.id);

        Ok(())
    }

    const fn check_display_name(display_name: &str) -> Result<(), ErrorCode> {
        if display_name.len() > MAX_DISPLAY_NAME_LENGTH {
            Err(ErrorCode::DisplayNameTooLong)
        } else {
            Ok(())
        }
    }

    const fn check_bio(bio: &str) -> Result<(), ErrorCode> {
        if bio.len() > MAX_BIO_LENGTH {
            Err(ErrorCode::BioTooLong)
        } else {
            Ok(())
        }
    }
}
