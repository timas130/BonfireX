pub mod data_loaders;
mod set_profile_note;
mod update_profile;
mod update_username;
pub mod user_profile;

use crate::services::profile::set_profile_note::SetProfileNoteMutation;
use crate::services::profile::update_profile::UpdateProfileMutation;
use crate::services::profile::update_username::UpdateUsernameMutation;
use async_graphql::MergedObject;

#[derive(MergedObject)]
pub struct ProfileQuery;

#[derive(MergedObject)]
pub struct ProfileMutation(
    UpdateUsernameMutation,
    UpdateProfileMutation,
    SetProfileNoteMutation,
);
