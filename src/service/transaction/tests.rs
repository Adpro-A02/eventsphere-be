use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::error::Error;
use uuid::Uuid;
use chrono::Utc;
use crate::model::transaction::{Transaction, TransactionStatus, Balance};
use crate::repository::transaction::transaction_repo::TransactionRepository;
use crate::repository::transaction::balance_repo::BalanceRepository;

struct MockTransactionRepository {
    transactions: Mutex<HashMap<Uuid, Transaction>>,
}

impl MockTransactionRepository {
    fn new() -> Self {
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

struct MockBalanceRepository {
    balances: Mutex<HashMap<Uuid, Balance>>,
}

impl MockBalanceRepository {
    fn new() -> Self {
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

fn create_service() -> DefaultTransactionService {
    let transaction_repository = Arc::new(MockTransactionRepository::new());
    let balance_repository = Arc::new(MockBalanceRepository::new());
    DefaultTransactionService::new(transaction_repository, balance_repository)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transaction_success() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        let ticket_id = Some(Uuid::new_v4());
        let amount = 1000;
        let description = "Test transaction".to_string();
        let payment_method = "Credit Card".to_string();

        let result = service.create_transaction(
            user_id,
            ticket_id,
            amount,
            description.clone(),
            payment_method.clone(),
        );

        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.user_id, user_id);
        assert_eq!(transaction.ticket_id, ticket_id);
        assert_eq!(transaction.amount, amount);
        assert_eq!(transaction.description, description);
        assert_eq!(transaction.payment_method, payment_method);
        assert_eq!(transaction.status, TransactionStatus::Pending);
    }

    #[test]
    fn test_create_transaction_invalid_amount() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        let result = service.create_transaction(
            user_id,
            None,
            0,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction amount must be positive");
    }

    #[test]
    fn test_process_payment_success() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        let transaction = service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        ).unwrap();

        let result = service.process_payment(transaction.id, None);
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.id, transaction.id);
        assert_eq!(processed.status, TransactionStatus::Success);
        assert!(processed.external_reference.is_some());
    }

    #[test]
    fn test_process_payment_with_external_reference() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        let transaction = service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        ).unwrap();

        let external_ref = "EXTERNAL-REF-123".to_string();
        let result = service.process_payment(transaction.id, Some(external_ref.clone()));
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.status, TransactionStatus::Success);
        assert_eq!(processed.external_reference, Some(external_ref));
    }

    #[test]
    fn test_process_payment_not_found() {
        let service = create_service();
        let non_existent_id = Uuid::new_v4();

        let result = service.process_payment(non_existent_id, None);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction not found");
    }

    #[test]
    fn test_process_payment_already_finalized() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        let transaction = service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        ).unwrap();
        service.process_payment(transaction.id, None).unwrap();

        let result = service.process_payment(transaction.id, None);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction is already finalized");
    }

    #[test]
    fn test_validate_payment_success() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        let transaction = service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        ).unwrap();
        service.process_payment(transaction.id, None).unwrap();

        let result = service.validate_payment(transaction.id);
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_validate_payment_not_found() {
        let service = create_service();
        let non_existent_id = Uuid::new_v4();

        let result = service.validate_payment(non_existent_id);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction not found");
    }

    #[test]
    fn test_refund_transaction_success() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        let transaction = service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        ).unwrap();
        service.process_payment(transaction.id, None).unwrap();

        let result = service.refund_transaction(transaction.id);
        
        assert!(result.is_ok());
        let refunded = result.unwrap();
        assert_eq!(refunded.status, TransactionStatus::Refunded);
    }

    #[test]
    fn test_get_transaction_found() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        let transaction = service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        ).unwrap();

        let result = service.get_transaction(transaction.id);
        
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, transaction.id);
    }

    #[test]
    fn test_get_transaction_not_found() {
        let service = create_service();
        let non_existent_id = Uuid::new_v4();

        let result = service.get_transaction(non_existent_id);
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_get_user_transactions() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        let transaction1 = service.create_transaction(
            user_id,
            None,
            1000,
            "Transaction 1".to_string(),
            "Credit Card".to_string(),
        ).unwrap();
        
        let transaction2 = service.create_transaction(
            user_id,
            None,
            2000,
            "Transaction 2".to_string(),
            "Credit Card".to_string(),
        ).unwrap();

        let result = service.get_user_transactions(user_id);
        
        assert!(result.is_ok());
        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 2);
        assert!(transactions.iter().any(|t| t.id == transaction1.id));
        assert!(transactions.iter().any(|t| t.id == transaction2.id));
    }

    #[test]
    fn test_add_funds_to_balance_success() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        let amount = 1000;
        
        let result = service.add_funds_to_balance(
            user_id, 
            amount, 
            "Credit Card".to_string()
        );
        
        assert!(result.is_ok());
        let (transaction, balance) = result.unwrap();
        assert_eq!(transaction.status, TransactionStatus::Success);
        assert_eq!(balance, amount);
    }

    #[test]
    fn test_add_funds_invalid_amount() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        let result = service.add_funds_to_balance(
            user_id, 
            0, 
            "Credit Card".to_string()
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Amount must be positive");
    }

    #[test]
    fn test_withdraw_funds_success() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        let initial_amount = 2000;
        let withdraw_amount = 1000;
        
        service.add_funds_to_balance(
            user_id, 
            initial_amount, 
            "Credit Card".to_string()
        ).unwrap();
        
        let result = service.withdraw_funds(
            user_id, 
            withdraw_amount, 
            "Withdrawal test".to_string()
        );
        
        if result.is_err() {
            println!("Withdraw funds error: {:?}", result.as_ref().unwrap_err());
        }
        
        assert!(result.is_ok());
        let (transaction, balance) = result.unwrap();
        assert_eq!(transaction.amount, -withdraw_amount);
        assert_eq!(transaction.status, TransactionStatus::Success);
        assert_eq!(balance, initial_amount - withdraw_amount);
    }

    #[test]
    fn test_withdraw_insufficient_funds() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        service.add_funds_to_balance(
            user_id, 
            500, 
            "Credit Card".to_string()
        ).unwrap();
        
        let result = service.withdraw_funds(
            user_id, 
            1000,
            "Withdrawal test".to_string()
        );
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Insufficient funds");
    }

    #[test]
    fn test_delete_transaction_success() {
        let service = create_service();
        let user_id = Uuid::new_v4();
        
        let transaction = service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        ).unwrap();
        
        let result = service.delete_transaction(transaction.id);
        
        assert!(result.is_ok());
        
        let get_result = service.get_transaction(transaction.id);
        assert!(get_result.is_ok());
        assert!(get_result.unwrap().is_none());
    }
    #[test]
    fn test_delete_transaction_not_found() {
        let service = create_service();
        let non_existent_id = Uuid::new_v4();
        
        let result = service.delete_transaction(non_existent_id);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction not found");
    }
}
