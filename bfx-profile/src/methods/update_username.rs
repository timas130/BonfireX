use crate::ProfileService;
use crate::models::profile::RawProfile;
use bfx_core::status::{ErrorCode, StatusExt};
use bfx_proto::profile::{UpdateUsernameReply, UpdateUsernameRequest};
use tonic::{Code, Request, Response, Status};

const USERNAME_CHANGE_LIMIT_PER_MONTH: i64 = 2;
const MIN_USERNAME_LENGTH: usize = 3;
const MAX_USERNAME_LENGTH: usize = 25;

impl ProfileService {
    pub async fn update_username(
        &self,
        request: Request<UpdateUsernameRequest>,
    ) -> Result<Response<UpdateUsernameReply>, Status> {
        let request = request.into_inner();

        if let Err(err) = Self::check_username(&request.username) {
            return Err(Status::coded(Code::InvalidArgument, err));
        }

        let mut tx = self.db.begin().await.map_err(Status::db)?;

        // check if the username has been changed >=2 times in a month
        let username_changes_count = sqlx::query_scalar!(
            "select count(*) as \"count!\"
             from profile.usernames
             where user_id = $1 and created_at > now() - interval '1 month'",
            request.user_id,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Status::db)?;

        if username_changes_count >= USERNAME_CHANGE_LIMIT_PER_MONTH {
            tx.rollback().await.map_err(Status::db)?;
            return Err(Status::coded(Code::AlreadyExists, ErrorCode::UsernameTaken));
        }

        // check if the username already exists
        let username_exists = sqlx::query_scalar!(
            "select 1 as \"found!\"
             from profile.usernames
             where lower(username) = lower($1)",
            request.username,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(Status::db)?
        .is_some();

        if username_exists {
            tx.rollback().await.map_err(Status::db)?;
            return Err(Status::coded(Code::AlreadyExists, ErrorCode::UsernameTaken));
        }

        // change the username
        let new_profile = sqlx::query_as!(
            RawProfile,
            "insert into profile.profiles (user_id, username)
             values ($1, $2)
             on conflict (user_id)
             do update set username = $2
             returning
                 *,
                 (select n.note
                  from profile.notes n
                  where n.profile_id = profile.profiles.user_id and n.user_id = $3)",
            request.user_id,
            request.username,
            request.for_user_id,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Status::db)?;

        sqlx::query!(
            "insert into profile.usernames (user_id, username)
             values ($1, $2)",
            request.user_id,
            request.username,
        )
        .execute(&mut *tx)
        .await
        .map_err(Status::db)?;

        tx.commit().await.map_err(Status::db)?;

        Ok(Response::new(UpdateUsernameReply {
            profile: Some(new_profile.into()),
        }))
    }

    fn check_username(username: &str) -> Result<(), ErrorCode> {
        if username.len() < MIN_USERNAME_LENGTH {
            Err(ErrorCode::UsernameTooShort)
        } else if !username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
            || username.chars().next().unwrap().is_ascii_digit()
            || username.starts_with('_')
            || username.ends_with('_')
        {
            Err(ErrorCode::InvalidUsernameCharacters)
        } else if username.len() > MAX_USERNAME_LENGTH {
            Err(ErrorCode::UsernameTooLong)
        } else {
            Ok(())
        }
    }
}
