use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ManagerMeta {
    pub request_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub origin: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
impl ManagerMeta {
    pub fn new<S: ToString>(origin: S) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            origin: origin.to_string(),
            created_at: Utc::now(),
            ..Self::default()
        }
    }

    pub fn reply<S: ToString>(self, origin: S) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            origin: origin.to_string(),
            parent_id: Some(self.request_id),
            created_at: Utc::now(),
        }
    }
}
