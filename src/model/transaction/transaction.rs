use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
    Refunded,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "Pending"),
            TransactionStatus::Success => write!(f, "Success"),
            TransactionStatus::Failed => write!(f, "Failed"),
            TransactionStatus::Refunded => write!(f, "Refunded"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ticket_id: Option<Uuid>,
    pub amount: i64,
    pub status: TransactionStatus,
    pub description: String,
    pub payment_method: String,
    pub external_reference: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Transaction {
    pub fn new(
        user_id: Uuid,
        ticket_id: Option<Uuid>,
        amount: i64,
        description: String,
        payment_method: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            ticket_id,
            amount,
            status: TransactionStatus::Pending,
            description,
            payment_method,
            external_reference: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn process(&mut self, success: bool, external_reference: Option<String>) {
        self.status = if success { TransactionStatus::Success } else { TransactionStatus::Failed };
        self.external_reference = external_reference;
        self.updated_at = Utc::now();
    }

    pub fn refund(&mut self) -> Result<(), String> {
        if self.status != TransactionStatus::Success {
            return Err("Only successful transactions can be refunded".to_string());
        }
        
        self.status = TransactionStatus::Refunded;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn is_finalized(&self) -> bool {
        matches!(self.status, TransactionStatus::Success | TransactionStatus::Failed | TransactionStatus::Refunded)
    }
}