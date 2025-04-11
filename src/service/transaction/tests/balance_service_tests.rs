use crate::service::transaction::tests::common::*;
use uuid::Uuid;
use crate::model::transaction::TransactionStatus;
use crate::service::transaction::TransactionService;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_or_create_balance_new() {
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        
        let result = balance_service.get_or_create_balance(user_id);
        
        assert!(result.is_ok());
        let balance = result.unwrap();
        assert_eq!(balance.user_id, user_id);
        assert_eq!(balance.amount, 0);
    }

    #[test]
    fn test_get_or_create_balance_existing() {
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        
        // Create balance first time
        let balance1 = balance_service.get_or_create_balance(user_id).unwrap();
        
        // Should retrieve the same balance
        let balance2 = balance_service.get_or_create_balance(user_id).unwrap();
        
        assert_eq!(balance1.user_id, balance2.user_id);
        assert_eq!(balance1.amount, balance2.amount);
    }
    
    #[test]
    fn test_add_funds_direct() {
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        let amount = 1000;
        
        // Create a balance first
        balance_service.get_or_create_balance(user_id).unwrap();
        
        // Then add funds to it
        let result = balance_service.add_funds(user_id, amount);
        
        assert!(result.is_ok());
        let new_balance = result.unwrap();
        assert_eq!(new_balance, amount);
        
        // Verify the balance was properly updated
        let balance = balance_service.get_or_create_balance(user_id).unwrap();
        assert_eq!(balance.amount, amount);
    }
    
    #[test]
    fn test_add_funds_invalid_amount() {
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        
        // Create a balance first
        balance_service.get_or_create_balance(user_id).unwrap();
        
        // Try to add an invalid amount
        let result = balance_service.add_funds(user_id, 0);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Amount must be positive");
    }
    
    #[test]
    fn test_withdraw_funds_direct() {
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        let initial_amount = 2000;
        let withdraw_amount = 1000;
        
        // Setup: Create a balance and add funds to it
        balance_service.get_or_create_balance(user_id).unwrap();
        balance_service.add_funds(user_id, initial_amount).unwrap();
        
        // Test withdrawing funds
        let result = balance_service.withdraw_funds(user_id, withdraw_amount);
        
        assert!(result.is_ok());
        let new_balance = result.unwrap();
        assert_eq!(new_balance, initial_amount - withdraw_amount);
        
        // Verify the balance was properly updated
        let balance = balance_service.get_or_create_balance(user_id).unwrap();
        assert_eq!(balance.amount, initial_amount - withdraw_amount);
    }
    
    #[test]
    fn test_withdraw_funds_insufficient() {
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        let initial_amount = 500;
        
        // Setup: Create a balance and add funds to it
        balance_service.get_or_create_balance(user_id).unwrap();
        balance_service.add_funds(user_id, initial_amount).unwrap();
        
        // Try to withdraw more than available
        let result = balance_service.withdraw_funds(user_id, 1000);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Insufficient funds");
        
        // Verify the balance was not changed
        let balance = balance_service.get_or_create_balance(user_id).unwrap();
        assert_eq!(balance.amount, initial_amount);
    }
    
    // These tests use transaction service but test balance-related operations
    // They are kept here for reference but would be better in transaction_service_tests.rs
    
    #[test]
    fn test_add_funds_to_balance_through_transaction() {
        let service = create_transaction_service();
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
    fn test_withdraw_funds_through_transaction() {
        let service = create_transaction_service();
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
        
        assert!(result.is_ok());
        let (transaction, balance) = result.unwrap();
        assert_eq!(transaction.amount, -withdraw_amount);
        assert_eq!(transaction.status, TransactionStatus::Success);
        assert_eq!(balance, initial_amount - withdraw_amount);
    }
}
