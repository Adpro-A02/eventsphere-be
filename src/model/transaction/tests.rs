use uuid::Uuid;
use crate::model::transaction::{Transaction, Balance, TransactionStatus};

#[cfg(test)]
pub mod model_tests {
    use super::*;
    
    #[test]
    fn test_transaction_new() {
        let user_id = Uuid::new_v4();
        let ticket_id = Some(Uuid::new_v4());
        let amount = 2500;
        let desc = "Ticket Sigma".to_string();
        let payment_method = "credit_card".to_string();
        
        let transaction = Transaction::new(
            user_id,
            ticket_id,
            amount,
            desc.clone(),
            payment_method.clone()
        );
        
        assert_eq!(transaction.user_id, user_id);
        assert_eq!(transaction.ticket_id, ticket_id);
        assert_eq!(transaction.amount, amount);
        assert_eq!(transaction.description, desc);
        assert_eq!(transaction.payment_method, payment_method);
        assert_eq!(transaction.status, TransactionStatus::Pending);
        assert!(transaction.external_reference.is_none());
    }
    
    #[test]
    fn test_transaction_process() {
        let mut transaction = Transaction::new(
            Uuid::new_v4(),
            None,
            1000,
            "Balance top-up".to_string(),
            "bank_transfer".to_string()
        );
        
        let external_ref = Some("PAY-123456789".to_string());
        transaction.process(true, external_ref.clone());
        
        assert_eq!(transaction.status, TransactionStatus::Success);
        assert_eq!(transaction.external_reference, external_ref);
    }
    
    #[test]
    fn test_transaction_refund() {
        let mut transaction = Transaction::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            5000,
            "Event ticket".to_string(),
            "balance".to_string()
        );
        
        // Check if transaction is successful or not
        assert!(transaction.refund().is_err());
        
        transaction.process(true, None);
        
        assert!(transaction.refund().is_ok());
        assert_eq!(transaction.status, TransactionStatus::Refunded);
    }
    
    #[test]
    fn test_balance_new() {
        let user_id = Uuid::new_v4();
        let balance = Balance::new(user_id);
        
        assert_eq!(balance.user_id, user_id);
        assert_eq!(balance.amount, 0);
    }
    
    #[test]
    fn test_balance_add_funds() {
        let mut balance = Balance::new(Uuid::new_v4());
        
        assert!(balance.add_funds(-100).is_err());
        
        let result = balance.add_funds(1000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000);
        assert_eq!(balance.amount, 1000);
        
        let result = balance.add_funds(500);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1500);
        assert_eq!(balance.amount, 1500);
    }
    
    #[test]
    fn test_balance_withdraw() {
        let mut balance = Balance::new(Uuid::new_v4());
        
        balance.add_funds(1000).unwrap();
        
        assert!(balance.withdraw(-100).is_err());
        
        assert!(balance.withdraw(9999).is_err());
        
        let result = balance.withdraw(500);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 500);
        assert_eq!(balance.amount, 500);
    }
}
