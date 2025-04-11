use std::error::Error;
use uuid::Uuid;

use crate::model::transaction::Transaction;

pub trait PaymentService {
    fn process_payment(&self, transaction: &Transaction) -> Result<(bool, Option<String>), Box<dyn Error>>;
}

pub struct MockPaymentService;

impl MockPaymentService {
    pub fn new() -> Self {
        Self {}
    }
}

impl PaymentService for MockPaymentService {
    fn process_payment(&self, transaction: &Transaction) -> Result<(bool, Option<String>), Box<dyn Error>> {
        let success = transaction.amount >= 0;
        let reference = if success {
            Some(format!("PG-REF-{}", Uuid::new_v4()))
        } else {
            None
        };
        
        Ok((success, reference))
    }
}
