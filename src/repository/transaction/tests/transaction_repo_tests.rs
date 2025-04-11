#[cfg(test)]
mod tests {
    use crate::repository::transaction::{
        TransactionRepository, 
        DbTransactionRepository,
        InMemoryTransactionPersistence
    };
    use crate::model::transaction::{Transaction, TransactionStatus};
    use uuid::Uuid;

    fn create_test_transaction() -> Transaction {
        Transaction::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            100,
            "Test transaction".to_string(),
            "credit_card".to_string()
        )
    }

    fn create_repo() -> impl TransactionRepository {
        DbTransactionRepository::new(InMemoryTransactionPersistence::new())
    }

    #[test]
    fn test_save_transaction() {
        let repo = create_repo();
        let transaction = create_test_transaction();
        let transaction_id = transaction.id;
        let user_id = transaction.user_id;
        
        let result = repo.save(&transaction).unwrap();
        
        assert_eq!(result.id, transaction_id);
        assert_eq!(result.user_id, user_id);
        assert_eq!(result.amount, 100);
    }

    #[test]
    fn test_find_by_id() {
        let repo = create_repo();
        let transaction = create_test_transaction();
        let transaction_id = transaction.id;
        repo.save(&transaction).unwrap();
        
        let found = repo.find_by_id(transaction_id).unwrap();
        
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, transaction_id);
    }

    #[test]
    fn test_find_by_id_not_found() {
        let repo = create_repo();
        
        let found = repo.find_by_id(Uuid::new_v4()).unwrap();
        
        assert!(found.is_none());
    }

    #[test]
    fn test_find_by_user() {
        let repo = create_repo();
        let user_id = Uuid::new_v4();
        
        let mut transaction1 = create_test_transaction();
        let mut transaction2 = create_test_transaction();
        let transaction3 = create_test_transaction();
        
        transaction1.user_id = user_id;
        transaction2.user_id = user_id;
        
        repo.save(&transaction1).unwrap();
        repo.save(&transaction2).unwrap();
        repo.save(&transaction3).unwrap();
        
        let user_transactions = repo.find_by_user(user_id).unwrap();
        
        assert_eq!(user_transactions.len(), 2);
        assert!(user_transactions.iter().all(|t| t.user_id == user_id));
    }

    #[test]
    fn test_update_status() {
        let repo = create_repo();
        let transaction = create_test_transaction();
        let transaction_id = transaction.id;
        repo.save(&transaction).unwrap();
        
        let updated = repo.update_status(transaction_id, TransactionStatus::Success).unwrap();
        
        assert_eq!(updated.status, TransactionStatus::Success);
        
        let found = repo.find_by_id(transaction_id).unwrap().unwrap();
        assert_eq!(found.status, TransactionStatus::Success);
    }

    #[test]
    fn test_update_status_not_found() {
        let repo = create_repo();
        
        let result = repo.update_status(Uuid::new_v4(), TransactionStatus::Success);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_transaction() {
        let repo = create_repo();
        let transaction = create_test_transaction();
        let transaction_id = transaction.id;
        repo.save(&transaction).unwrap();
        
        let result = repo.delete(transaction_id);
        
        assert!(result.is_ok());
        let found = repo.find_by_id(transaction_id).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_delete_transaction_not_found() {
        let repo = create_repo();
        
        let result = repo.delete(Uuid::new_v4());
        
        assert!(result.is_err());
    }
}
