use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub created_at: DateTime<Utc>,
}

impl RefreshToken {
    pub fn new(user_id: Uuid, token: String, expires_in_days: i64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            token,
            expires_at: now + chrono::Duration::days(expires_in_days),
            is_revoked: false,
            created_at: now,
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.is_revoked && self.expires_at > Utc::now()
    }
}