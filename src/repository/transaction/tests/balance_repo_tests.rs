#[cfg(test)]
mod tests {
    use crate::repository::transaction::{
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

    #[test]
    fn test_save_balance() {
        let repo = create_repo();
        let balance = create_test_balance(500);
        let user_id = balance.user_id;
        
        let result = repo.save(&balance);
        
        assert!(result.is_ok());
        
        let found = repo.find_by_user_id(user_id).unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.amount, 500);
    }

    #[test]
    fn test_find_by_user_id() {
        let repo = create_repo();
        let balance = create_test_balance(1000);
        let user_id = balance.user_id;
        
        repo.save(&balance).unwrap();
        
        let found = repo.find_by_user_id(user_id).unwrap();
        
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.amount, 1000);
    }

    #[test]
    fn test_find_by_user_id_not_found() {
        let repo = create_repo();
        let non_existent_id = Uuid::new_v4();
        
        let found = repo.find_by_user_id(non_existent_id).unwrap();
        
        assert!(found.is_none());
    }

    #[test]
    fn test_update_balance() {
        let repo = create_repo();
        let mut balance = create_test_balance(500);
        let user_id = balance.user_id;
        
        // Save initial balance
        repo.save(&balance).unwrap();
        
        // Update the balance and save again
        balance.amount = 750;
        repo.save(&balance).unwrap();
        
        // Verify the balance was updated
        let found = repo.find_by_user_id(user_id).unwrap().unwrap();
        assert_eq!(found.amount, 750);
    }
}
