use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::error::Error;
use uuid::Uuid;
use chrono::Utc;
use crate::model::transaction::{Transaction, TransactionStatus, Balance};
use crate::repository::transaction::transaction_repo::TransactionRepository;
use crate::repository::transaction::balance_repo::BalanceRepository;
use crate::service::transaction::balance_service::{BalanceService, DefaultBalanceService};
use crate::service::transaction::payment_service::{PaymentService, MockPaymentService};
use crate::service::transaction::transaction_service::DefaultTransactionService;

pub struct MockTransactionRepository {
    transactions: Mutex<HashMap<Uuid, Transaction>>,
}

impl MockTransactionRepository {
    pub fn new() -> Self {
        Self {
            transactions: Mutex::new(HashMap::new()),
        }
    }
}

impl TransactionRepository for MockTransactionRepository {
    fn save(&self, transaction: &Transaction) -> Result<Transaction, Box<dyn Error>> {
        let mut transactions = self.transactions.lock().unwrap();
        let transaction_clone = transaction.clone();
        transactions.insert(transaction.id, transaction_clone.clone());
        Ok(transaction_clone)
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>> {
        let transactions = self.transactions.lock().unwrap();
        Ok(transactions.get(&id).cloned())
    }

    fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>> {
        let transactions = self.transactions.lock().unwrap();
        let user_transactions: Vec<Transaction> = transactions
            .values()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect();
        Ok(user_transactions)
    }

    fn update_status(&self, id: Uuid, status: TransactionStatus) -> Result<Transaction, Box<dyn Error>> {
        let mut transactions = self.transactions.lock().unwrap();
        
        match transactions.get_mut(&id) {
            Some(transaction) => {
                transaction.status = status;
                transaction.updated_at = Utc::now();
                Ok(transaction.clone())
            },
            None => Err("Transaction not found".into()),
        }
    }

    fn delete(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let mut transactions = self.transactions.lock().unwrap();
        if transactions.remove(&id).is_some() {
            Ok(())
        } else {
            Err("Transaction not found".into())
        }
    }
}

pub struct MockBalanceRepository {
    balances: Mutex<HashMap<Uuid, Balance>>,
}

impl MockBalanceRepository {
    pub fn new() -> Self {
        Self {
            balances: Mutex::new(HashMap::new()),
        }
    }
}

impl BalanceRepository for MockBalanceRepository {
    fn save(&self, balance: &Balance) -> Result<(), Box<dyn Error>> {
        let mut balances = self.balances.lock().unwrap();
        balances.insert(balance.user_id, balance.clone());
        Ok(())
    }

    fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error>> {
        let balances = self.balances.lock().unwrap();
        Ok(balances.get(&user_id).cloned())
    }
}

pub fn create_transaction_service() -> DefaultTransactionService {
    let transaction_repository = Arc::new(MockTransactionRepository::new());
    let balance_repository = Arc::new(MockBalanceRepository::new());
    let balance_service = Arc::new(DefaultBalanceService::new(balance_repository));
    let payment_service = Arc::new(MockPaymentService::new());
    
    DefaultTransactionService::new(
        transaction_repository, 
        balance_service,
        payment_service
    )
}

pub fn create_balance_service() -> Arc<dyn BalanceService> {
    let balance_repository = Arc::new(MockBalanceRepository::new());
    Arc::new(DefaultBalanceService::new(balance_repository))
}

pub fn create_payment_service() -> Arc<dyn PaymentService> {
    Arc::new(MockPaymentService::new())
}
