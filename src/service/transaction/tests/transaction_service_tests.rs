use crate::service::transaction::tests::common::*;
use uuid::Uuid;
use crate::model::transaction::TransactionStatus;
use crate::service::transaction::transaction_service::TransactionService;
use tokio::runtime::Runtime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transaction_success() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        let ticket_id = Some(Uuid::new_v4());
        let amount = 1000;
        let description = "Test transaction".to_string();
        let payment_method = "Credit Card".to_string();

        let result = rt.block_on(service.create_transaction(
            user_id,
            ticket_id,
            amount,
            description.clone(),
            payment_method.clone(),
        ));

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
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        
        let result = rt.block_on(service.create_transaction(
            user_id,
            None,
            0,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        ));

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction amount must be positive");
    }    
    
    #[test]
    fn test_process_payment_success() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        
        let transaction = rt.block_on(service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        )).unwrap();

        let result = rt.block_on(service.process_payment(transaction.id, None));
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.id, transaction.id);
        assert_eq!(processed.status, TransactionStatus::Success);
        assert!(processed.external_reference.is_some());
    }    
    
    #[test]
    fn test_process_payment_with_external_reference() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        
        let transaction = rt.block_on(service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        )).unwrap();

        let external_ref = "EXTERNAL-REF-123".to_string();
        let result = rt.block_on(service.process_payment(transaction.id, Some(external_ref.clone())));
        
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.status, TransactionStatus::Success);
        assert_eq!(processed.external_reference, Some(external_ref));
    }    
    
    #[test]
    fn test_process_payment_not_found() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let non_existent_id = Uuid::new_v4();

        let result = rt.block_on(service.process_payment(non_existent_id, None));
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction not found");
    }    
    
    #[test]
    fn test_process_payment_already_finalized() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        
        let transaction = rt.block_on(service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        )).unwrap();
        rt.block_on(service.process_payment(transaction.id, None)).unwrap();

        let result = rt.block_on(service.process_payment(transaction.id, None));
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction is already finalized");
    }    
    
    #[test]
    fn test_validate_payment_success() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        
        let transaction = rt.block_on(service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        )).unwrap();
        rt.block_on(service.process_payment(transaction.id, None)).unwrap();

        let result = rt.block_on(service.validate_payment(transaction.id));
        
        assert!(result.is_ok());
        assert!(result.unwrap());
    }    
    
    #[test]
    fn test_validate_payment_not_found() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let non_existent_id = Uuid::new_v4();

        let result = rt.block_on(service.validate_payment(non_existent_id));
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction not found");
    }    
    
    #[test]
    fn test_refund_transaction_success() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        
        let transaction = rt.block_on(service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        )).unwrap();
        rt.block_on(service.process_payment(transaction.id, None)).unwrap();

        let result = rt.block_on(service.refund_transaction(transaction.id));
        
        assert!(result.is_ok());
        let refunded = result.unwrap();
        assert_eq!(refunded.status, TransactionStatus::Refunded);
    }    
    
    #[test]
    fn test_get_transaction_found() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        
        let transaction = rt.block_on(service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        )).unwrap();

        let result = rt.block_on(service.get_transaction(transaction.id));
        
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, transaction.id);
    }    
    
    #[test]
    fn test_get_transaction_not_found() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let non_existent_id = Uuid::new_v4();

        let result = rt.block_on(service.get_transaction(non_existent_id));
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }    
    
    #[test]
    fn test_get_user_transactions() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        
        let transaction1 = rt.block_on(service.create_transaction(
            user_id,
            None,
            1000,
            "Transaction 1".to_string(),
            "Credit Card".to_string(),
        )).unwrap();
        
        let transaction2 = rt.block_on(service.create_transaction(
            user_id,
            None,
            2000,
            "Transaction 2".to_string(),
            "Credit Card".to_string(),
        )).unwrap();

        let result = rt.block_on(service.get_user_transactions(user_id));
        
        assert!(result.is_ok());
        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 2);
        assert!(transactions.iter().any(|t| t.id == transaction1.id));
        assert!(transactions.iter().any(|t| t.id == transaction2.id));
    }    
    
    #[test]
    fn test_delete_transaction_success() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let user_id = Uuid::new_v4();
        
        let transaction = rt.block_on(service.create_transaction(
            user_id,
            None,
            1000,
            "Test transaction".to_string(),
            "Credit Card".to_string(),
        )).unwrap();
        
        let result = rt.block_on(service.delete_transaction(transaction.id));
        
        assert!(result.is_ok());
        
        let get_result = rt.block_on(service.get_transaction(transaction.id));
        assert!(get_result.is_ok());
        assert!(get_result.unwrap().is_none());
    }    
    
    #[test]
    fn test_delete_transaction_not_found() {
        let rt = Runtime::new().unwrap();
        let service = create_transaction_service();
        let non_existent_id = Uuid::new_v4();
        
        let result = rt.block_on(service.delete_transaction(non_existent_id));
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction not found");
    }
}
