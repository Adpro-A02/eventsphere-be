use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::model::transaction::{Transaction, TransactionStatus};
use crate::repository::transaction::transaction_repo::TransactionRepository;
use crate::service::transaction::balance_service::BalanceService;
use crate::service::transaction::payment_service::PaymentService;

pub trait TransactionService {
    fn create_transaction(
        &self,
        user_id: Uuid,
        ticket_id: Option<Uuid>,
        amount: i64,
        description: String,
        payment_method: String,
    ) -> Result<Transaction, Box<dyn Error>>;

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
    balance_service: Arc<dyn BalanceService>,
    payment_service: Arc<dyn PaymentService>,
}

impl DefaultTransactionService {
    pub fn new(
        transaction_repository: Arc<dyn TransactionRepository>,
        balance_service: Arc<dyn BalanceService>,
        payment_service: Arc<dyn PaymentService>
    ) -> Self {
        Self {
            transaction_repository,
            balance_service,
            payment_service,
        }
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
        
        let (success, reference) = self.payment_service.process_payment(&transaction)?;
        
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
        
        let new_balance = self.balance_service.add_funds(user_id, amount)?;
        
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
        
        // Check if user has sufficient funds first
        let balance = self.balance_service.get_or_create_balance(user_id)?;
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
        
        let new_balance = self.balance_service.withdraw_funds(user_id, amount)?;
        
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
