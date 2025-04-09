use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::model::transaction::{Transaction, TransactionStatus};
use crate::model::transaction::Balance;
use crate::repository::transaction::transaction_repo::TransactionRepository;
use crate::repository::transaction::balance_repo::BalanceRepository;

pub trait TransactionService {
    fn create_transaction(
        &self,
        user_id: Uuid,
        ticket_id: Option<Uuid>,
        amount: i64,
        description: String,
        payment_method: String,
    ) -> Result<Transaction, Box<dyn Error>>;

    // Process a payment through payment gateway
    fn process_payment(
        &self,
        transaction_id: Uuid,
        external_reference: Option<String>,
    ) -> Result<Transaction, Box<dyn Error>>;
    
    fn validate_payment(&self, transaction_id: Uuid) -> Result<bool, Box<dyn Error>>;
    fn refund_transaction(&self, transaction_id: Uuid) -> Result<Transaction, Box<dyn Error>>;
    fn get_transaction(&self, transaction_id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>>;
    fn get_user_transactions(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>>;
    
    fn add_funds_to_balance(
        &self,
        user_id: Uuid,
        amount: i64,
        payment_method: String,
    ) -> Result<(Transaction, i64), Box<dyn Error>>;
    
    fn withdraw_funds(
        &self,
        user_id: Uuid,
        amount: i64,
        description: String,
    ) -> Result<(Transaction, i64), Box<dyn Error>>;

    fn delete_transaction(&self, transaction_id: Uuid) -> Result<(), Box<dyn Error>>;
}

pub struct DefaultTransactionService {
    transaction_repository: Arc<dyn TransactionRepository>,
    balance_repository: Arc<dyn BalanceRepository>,
}

impl DefaultTransactionService {
    pub fn new(
        transaction_repository: Arc<dyn TransactionRepository>,
        balance_repository: Arc<dyn BalanceRepository>
    ) -> Self {
        Self {
            transaction_repository,
            balance_repository,
        }
    }
    
    // Helper method to simulate payment gateway processing
    fn process_with_payment_gateway(&self, transaction: &Transaction) -> Result<(bool, Option<String>), Box<dyn Error>> {
        let success = transaction.amount >= 0;
        let reference = if success {
            Some(format!("PG-REF-{}", Uuid::new_v4()))
        } else {
            None
        };
        
        Ok((success, reference))
    }
    
    fn get_or_create_balance(&self, user_id: Uuid) -> Result<Balance, Box<dyn Error>> {
        match self.balance_repository.find_by_user_id(user_id)? {
            Some(balance) => Ok(balance),
            None => {
                let balance = Balance::new(user_id);
                self.balance_repository.save(&balance)?;
                Ok(balance)
            }
        }
    }
    
    fn save_balance(&self, balance: &Balance) -> Result<(), Box<dyn Error>> {
        self.balance_repository.save(balance)
    }
}

impl TransactionService for DefaultTransactionService {
    fn create_transaction(
        &self,
        user_id: Uuid,
        ticket_id: Option<Uuid>,
        amount: i64,
        description: String,
        payment_method: String,
    ) -> Result<Transaction, Box<dyn Error>> {
        if amount <= 0 {
            return Err("Transaction amount must be positive".into());
        }
        
        let transaction = Transaction::new(
            user_id,
            ticket_id,
            amount,
            description,
            payment_method,
        );
        
        self.transaction_repository.save(&transaction)
    }
    
    fn process_payment(
        &self,
        transaction_id: Uuid,
        external_reference: Option<String>,
    ) -> Result<Transaction, Box<dyn Error>> {
        let transaction = match self.transaction_repository.find_by_id(transaction_id)? {
            Some(t) => t,
            None => return Err("Transaction not found".into()),
        };
        
        if transaction.is_finalized() {
            return Err("Transaction is already finalized".into());
        }
        
        // If external reference is provided, use it directly
        if let Some(ref_id) = external_reference {
            let mut updated = self.transaction_repository.update_status(
                transaction_id, 
                TransactionStatus::Success
            )?;
            updated.external_reference = Some(ref_id);
            return self.transaction_repository.save(&updated);
        }
        
        let (success, reference) = self.process_with_payment_gateway(&transaction)?;
        
        let status = if success { 
            TransactionStatus::Success 
        } else { 
            TransactionStatus::Failed 
        };
        
        let mut updated_transaction = self.transaction_repository.update_status(transaction_id, status)?;
        updated_transaction.external_reference = reference;
        updated_transaction.updated_at = Utc::now();
        
        self.transaction_repository.save(&updated_transaction)
    }
    
    fn validate_payment(&self, transaction_id: Uuid) -> Result<bool, Box<dyn Error>> {
        let transaction = match self.transaction_repository.find_by_id(transaction_id)? {
            Some(t) => t,
            None => return Err("Transaction not found".into()),
        };
        
        Ok(transaction.status == TransactionStatus::Success)
    }
    
    fn refund_transaction(&self, transaction_id: Uuid) -> Result<Transaction, Box<dyn Error>> {
        let mut transaction = match self.transaction_repository.find_by_id(transaction_id)? {
            Some(t) => t,
            None => return Err("Transaction not found".into()),
        };
        
        transaction.refund().map_err(|e| -> Box<dyn Error> { e.into() })?;
        
        self.transaction_repository.update_status(transaction_id, TransactionStatus::Refunded)
    }
    
    fn get_transaction(&self, transaction_id: Uuid) -> Result<Option<Transaction>, Box<dyn Error>> {
        self.transaction_repository.find_by_id(transaction_id)
    }
    
    fn get_user_transactions(&self, user_id: Uuid) -> Result<Vec<Transaction>, Box<dyn Error>> {
        self.transaction_repository.find_by_user(user_id)
    }
    
    fn add_funds_to_balance(
        &self,
        user_id: Uuid,
        amount: i64,
        payment_method: String,
    ) -> Result<(Transaction, i64), Box<dyn Error>> {
        if amount <= 0 {
            return Err("Amount must be positive".into());
        }
        
        let transaction = self.create_transaction(
            user_id,
            None,
            amount,
            "Add funds to balance".to_string(),
            payment_method,
        )?;
        
        let processed_transaction = self.process_payment(transaction.id, None)?;
        
        if processed_transaction.status != TransactionStatus::Success {
            return Err("Payment processing failed".into());
        }
        
        let mut balance = self.get_or_create_balance(user_id)?;
        let new_balance = balance.add_funds(amount).map_err(|e| e.to_string())?;
        self.save_balance(&balance)?;
        
        Ok((processed_transaction, new_balance))
    }
    
    fn withdraw_funds(
        &self,
        user_id: Uuid,
        amount: i64,
        description: String,
    ) -> Result<(Transaction, i64), Box<dyn Error>> {
        if amount <= 0 {
            return Err("Amount must be positive".into());
        }
        
        let mut balance = self.get_or_create_balance(user_id)?;
        if balance.amount < amount {
            return Err("Insufficient funds".into());
        }
        
        let transaction = self.create_transaction(
            user_id,
            None,
            amount,
            description,
            "Balance".to_string(),
        )?;
        
        let mut processed_transaction = self.transaction_repository.update_status(
            transaction.id,
            TransactionStatus::Success,
        )?;
        
        processed_transaction.amount = -amount;
        let processed_transaction = self.transaction_repository.save(&processed_transaction)?;
        
        let new_balance = balance.withdraw(amount).map_err(|e| e.to_string())?;
        self.save_balance(&balance)?;
        
        Ok((processed_transaction, new_balance))
    }

    fn delete_transaction(&self, transaction_id: Uuid) -> Result<(), Box<dyn Error>> {
        let transaction = match self.transaction_repository.find_by_id(transaction_id)? {
            Some(t) => t,
            None => return Err("Transaction not found".into()),
        };
        
        if transaction.status != TransactionStatus::Pending {
            return Err("Cannot delete a processed transaction".into());
        }
        
        self.transaction_repository.delete(transaction_id)
    }
}
