use crate::service::transaction::tests::common::*;
use uuid::Uuid;
use crate::model::transaction::Transaction;
use tokio::runtime::Runtime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_payment_positive_amount() {
        let rt = Runtime::new().unwrap();
        let payment_service = create_payment_service();
        let user_id = Uuid::new_v4();
        
        let transaction = Transaction::new(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        );
        
        let result = rt.block_on(payment_service.process_payment(&transaction));
        
        assert!(result.is_ok());
        let (success, reference) = result.unwrap();
        assert!(success);
        assert!(reference.is_some());
    }    
    
    #[test]
    fn test_process_payment_negative_amount() {
        let rt = Runtime::new().unwrap();
        let payment_service = create_payment_service();
        let user_id = Uuid::new_v4();
        
        let mut transaction = Transaction::new(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        );
        transaction.amount = -1000;
        
        let result = rt.block_on(payment_service.process_payment(&transaction));
        
        assert!(result.is_ok());
        let (success, reference) = result.unwrap();
        assert!(!success);
        assert!(reference.is_none());
    }
}
