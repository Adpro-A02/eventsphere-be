#[cfg(test)]
mod tests {
    use crate::repository::transaction::balance_repo::{
        BalanceRepository, 
        DbBalanceRepository,
        InMemoryBalancePersistence
    };
    use crate::model::transaction::Balance;
    use uuid::Uuid;
    use chrono;

    fn create_test_balance(amount: i64) -> Balance {
        Balance {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            amount,
            updated_at: chrono::Utc::now(),
        }
    }

    fn create_repo() -> impl BalanceRepository {
        DbBalanceRepository::new(InMemoryBalancePersistence::new())
    }

    #[tokio::test]
    async fn test_save_balance() {
        let repo = create_repo();
        let balance = create_test_balance(500);
        let user_id = balance.user_id;
        
        let result = repo.save(&balance).await;
        
        assert!(result.is_ok());
        
        let found = repo.find_by_user_id(user_id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.amount, 500);
    }

    #[tokio::test]
    async fn test_find_by_user_id() {
        let repo = create_repo();
        let balance = create_test_balance(1000);
        let user_id = balance.user_id;
        
        repo.save(&balance).await.unwrap();
        
        let found = repo.find_by_user_id(user_id).await.unwrap();
        
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.amount, 1000);
    }

    #[tokio::test]
    async fn test_find_by_user_id_not_found() {
        let repo = create_repo();
        let non_existent_id = Uuid::new_v4();
        
        let found = repo.find_by_user_id(non_existent_id).await.unwrap();
        
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_update_balance() {
        let repo = create_repo();
        let mut balance = create_test_balance(500);
        let user_id = balance.user_id;
        
        repo.save(&balance).await.unwrap();
        
        balance.amount = 750;
        repo.save(&balance).await.unwrap();
        
        let found = repo.find_by_user_id(user_id).await.unwrap().unwrap();
        assert_eq!(found.amount, 750);
    }
}
