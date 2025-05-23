use crate::service::transaction::tests::common::*;
use uuid::Uuid;
use crate::model::transaction::TransactionStatus;
use crate::service::transaction::TransactionService;
use tokio::runtime::Runtime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_or_create_balance_new() {
        let rt = Runtime::new().unwrap();
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        
        let result = rt.block_on(balance_service.get_or_create_balance(user_id));
        
        assert!(result.is_ok());
        let balance = result.unwrap();
        assert_eq!(balance.user_id, user_id);
        assert_eq!(balance.amount, 0);
    }    #[test]

    fn test_get_or_create_balance_existing() {
        let rt = Runtime::new().unwrap();
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        
        let balance1 = rt.block_on(balance_service.get_or_create_balance(user_id)).unwrap();
        
        let balance2 = rt.block_on(balance_service.get_or_create_balance(user_id)).unwrap();
        
        assert_eq!(balance1.user_id, balance2.user_id);
        assert_eq!(balance1.amount, balance2.amount);
    }
    
    #[test]
    fn test_add_funds_direct() {
        let rt = Runtime::new().unwrap();
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        let amount = 1000;
        
        rt.block_on(balance_service.get_or_create_balance(user_id)).unwrap();
        
        let result = rt.block_on(balance_service.add_funds(user_id, amount));
        
        assert!(result.is_ok());
        let new_balance = result.unwrap();
        assert_eq!(new_balance, amount);
        
        let balance = rt.block_on(balance_service.get_or_create_balance(user_id)).unwrap();
        assert_eq!(balance.amount, amount);
    }
  
    #[test]
    fn test_add_funds_invalid_amount() {
        let rt = Runtime::new().unwrap();
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        
        rt.block_on(balance_service.get_or_create_balance(user_id)).unwrap();
        
        let result = rt.block_on(balance_service.add_funds(user_id, 0));
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Amount must be positive");
    }
      
    #[test]
    fn test_withdraw_funds_direct() {
        let rt = Runtime::new().unwrap();
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        let initial_amount = 2000;
        let withdraw_amount = 1000;
        
        rt.block_on(balance_service.get_or_create_balance(user_id)).unwrap();
        rt.block_on(balance_service.add_funds(user_id, initial_amount)).unwrap();
        
        let result = rt.block_on(balance_service.withdraw_funds(user_id, withdraw_amount));
        
        assert!(result.is_ok());
        let new_balance = result.unwrap();
        assert_eq!(new_balance, initial_amount - withdraw_amount);
        
        let balance = rt.block_on(balance_service.get_or_create_balance(user_id)).unwrap();
        assert_eq!(balance.amount, initial_amount - withdraw_amount);
    }
      
    #[test]
    fn test_withdraw_funds_insufficient() {
        let rt = Runtime::new().unwrap();
        let balance_service = create_balance_service();
        let user_id = Uuid::new_v4();
        let initial_amount = 500;
        
        rt.block_on(balance_service.get_or_create_balance(user_id)).unwrap();
        rt.block_on(balance_service.add_funds(user_id, initial_amount)).unwrap();
        
        let result = rt.block_on(balance_service.withdraw_funds(user_id, 1000));
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Insufficient funds");
        
        let balance = rt.block_on(balance_service.get_or_create_balance(user_id)).unwrap();
        assert_eq!(balance.amount, initial_amount);
    }
    
    #[test]
    fn test_add_funds_to_balance_through_transaction() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        let amount = 1000;
        
        let result = rt.block_on(service.add_funds_to_balance(
            user_id, 
            amount, 
            "Credit Card".to_string()
        ));
        
        assert!(result.is_ok());
        let (transaction, balance) = result.unwrap();
        assert_eq!(transaction.status, TransactionStatus::Success);
        assert_eq!(balance, amount);
    }
      
    #[test]
    fn test_withdraw_funds_through_transaction() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        let initial_amount = 2000;
        let withdraw_amount = 1000;
        
        rt.block_on(service.add_funds_to_balance(
            user_id, 
            initial_amount, 
            "Credit Card".to_string()
        )).unwrap();
        
        let result = rt.block_on(service.withdraw_funds(
            user_id, 
            withdraw_amount, 
            "Withdrawal test".to_string()
        ));
        
        assert!(result.is_ok());
        let (transaction, balance) = result.unwrap();
        assert_eq!(transaction.amount, -withdraw_amount);
        assert_eq!(transaction.status, TransactionStatus::Success);
        assert_eq!(balance, initial_amount - withdraw_amount);
    }
}
