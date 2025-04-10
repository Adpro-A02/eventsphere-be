use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: i64,
    pub updated_at: DateTime<Utc>,
}

impl Balance {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            amount: 0,
            updated_at: Utc::now(),
        }
    }

    pub fn add_funds(&mut self, amount: i64) -> Result<i64, String> {
        if amount <= 0 {
            return Err("Amount must be positive".to_string());
        }
        
        self.amount += amount;
        self.updated_at = Utc::now();
        Ok(self.amount)
    }

    pub fn withdraw(&mut self, amount: i64) -> Result<i64, String> {
        if amount <= 0 {
            return Err("Amount must be positive".to_string());
        }
        
        if amount > self.amount {
            return Err("Insufficient funds".to_string());
        }
        
        self.amount -= amount;
        self.updated_at = Utc::now();
        Ok(self.amount)
    }
}
