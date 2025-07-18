use std::error::Error;
use uuid::Uuid;
use async_trait::async_trait;

use crate::model::transaction::Transaction;

#[async_trait]
pub trait PaymentService {
    async fn process_payment(&self, transaction: &Transaction) -> Result<(bool, Option<String>), Box<dyn Error + Send + Sync>>;
}

pub struct MockPaymentService;

impl MockPaymentService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PaymentService for MockPaymentService {
    async fn process_payment(&self, transaction: &Transaction) -> Result<(bool, Option<String>), Box<dyn Error + Send + Sync>> {
        let success = transaction.amount >= 0;
        let reference = if success {
            Some(format!("PG-REF-{}", Uuid::new_v4()))
        } else {
            None
        };
        
        Ok((success, reference))
    }
}
