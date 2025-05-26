#[cfg(test)]
mod tests {
    use crate::repository::transaction::transaction_repo::{
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

    #[tokio::test]
    async fn test_save_transaction() {
        let repo = create_repo();
        let transaction = create_test_transaction();
        let transaction_id = transaction.id;
        let user_id = transaction.user_id;
        
        let result = repo.save(&transaction).await.unwrap();
        
        assert_eq!(result.id, transaction_id);
        assert_eq!(result.user_id, user_id);
        assert_eq!(result.amount, 100);
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let repo = create_repo();
        let transaction = create_test_transaction();
        let transaction_id = transaction.id;
        repo.save(&transaction).await.unwrap();
        
        let found = repo.find_by_id(transaction_id).await.unwrap();
        
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, transaction_id);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        let repo = create_repo();
        
        let found = repo.find_by_id(Uuid::new_v4()).await.unwrap();
        
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_by_user() {
        let repo = create_repo();
        let user_id = Uuid::new_v4();
        
        let mut transaction1 = create_test_transaction();
        let mut transaction2 = create_test_transaction();
        let transaction3 = create_test_transaction();
        
        transaction1.user_id = user_id;
        transaction2.user_id = user_id;
        
        repo.save(&transaction1).await.unwrap();
        repo.save(&transaction2).await.unwrap();
        repo.save(&transaction3).await.unwrap();
        
        let user_transactions = repo.find_by_user(user_id).await.unwrap();
        
        assert_eq!(user_transactions.len(), 2);
        assert!(user_transactions.iter().all(|t| t.user_id == user_id));
    }

    #[tokio::test]
    async fn test_update_status() {
        let repo = create_repo();
        let transaction = create_test_transaction();
        let transaction_id = transaction.id;
        repo.save(&transaction).await.unwrap();
        
        let updated = repo.update_status(transaction_id, TransactionStatus::Success).await.unwrap();
        
        assert_eq!(updated.status, TransactionStatus::Success);
        
        let found = repo.find_by_id(transaction_id).await.unwrap().unwrap();
        assert_eq!(found.status, TransactionStatus::Success);
    }

    #[tokio::test]
    async fn test_update_status_not_found() {
        let repo = create_repo();
        
        let result = repo.update_status(Uuid::new_v4(), TransactionStatus::Success).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_transaction() {
        let repo = create_repo();
        let transaction = create_test_transaction();
        let transaction_id = transaction.id;
        repo.save(&transaction).await.unwrap();
        
        let result = repo.delete(transaction_id).await;
        
        assert!(result.is_ok());
        let found = repo.find_by_id(transaction_id).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_transaction_not_found() {
        let repo = create_repo();
        
        let result = repo.delete(Uuid::new_v4()).await;
        
        assert!(result.is_err());
    }
}
