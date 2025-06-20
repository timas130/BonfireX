use crate::definition::NotificationDefinition;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NotificationData {
    pub definition: NotificationDefinition,
}
