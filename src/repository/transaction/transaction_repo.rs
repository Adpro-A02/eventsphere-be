use chrono::Utc;
use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
use uuid::Uuid;
use crate::model::transaction::{Transaction, TransactionStatus};

pub trait TransactionRepository {
    fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>>;
    fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>>;
    fn save(&self, transaction: &Transaction) -> Result<Transaction, Box<dyn Error>>;
    fn update_status(&self, id: Uuid, status: TransactionStatus) -> Result<Transaction, Box<dyn Error>>;
    fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>>;
}

pub struct DbTransactionRepository {
    transactions: RwLock<HashMap<Uuid, Transaction>>,
}

impl DbTransactionRepository {
    pub fn new() -> Self {
        Self {
            transactions: RwLock::new(HashMap::new()),
        }
    }
}

impl TransactionRepository for DbTransactionRepository {
    fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>> {
        let transactions = self.transactions.read().unwrap();
        Ok(transactions.get(&id).cloned())
    }
    
    fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>> {
        let transactions = self.transactions.read().unwrap();
        let user_transactions = transactions
            .values()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect();
        Ok(user_transactions)
    }
    
    fn save(&self, transaction: &Transaction) -> Result<Transaction, Box<dyn Error>> {
        let mut transactions = self.transactions.write().unwrap();
        let transaction_clone = transaction.clone();
        transactions.insert(transaction.id, transaction_clone.clone());
        Ok(transaction_clone)
    }
    
    fn update_status(&self, id: Uuid, status: TransactionStatus) -> Result<Transaction, Box<dyn Error>> {
        let mut transactions = self.transactions.write().unwrap();
        
        if let Some(transaction) = transactions.get_mut(&id) {
            transaction.status = status;
            transaction.updated_at = Utc::now();
            Ok(transaction.clone())
        } else {
            Err("Transaction not found".into())
        }
    }
    
    fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let mut transactions = self.transactions.write().unwrap();
        
        if transactions.remove(&id).is_some() {
            Ok(())
        } else {
            Err("Transaction not found".into())
        }
    }
}