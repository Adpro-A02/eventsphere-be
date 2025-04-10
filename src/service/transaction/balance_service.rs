use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

use crate::model::transaction::Balance;
use crate::repository::transaction::balance_repo::BalanceRepository;

pub trait BalanceService {
    fn get_user_balance(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error>>;
    fn get_or_create_balance(&self, user_id: Uuid) -> Result<Balance, Box<dyn Error>>;
    fn add_funds(&self, user_id: Uuid, amount: i64) -> Result<i64, Box<dyn Error>>;
    fn withdraw_funds(&self, user_id: Uuid, amount: i64) -> Result<i64, Box<dyn Error>>;
    fn save_balance(&self, balance: &Balance) -> Result<(), Box<dyn Error>>;
}

pub struct DefaultBalanceService {
    balance_repository: Arc<dyn BalanceRepository>,
}

impl DefaultBalanceService {
    pub fn new(balance_repository: Arc<dyn BalanceRepository>) -> Self {
        Self {
            balance_repository,
        }
    }
}

impl BalanceService for DefaultBalanceService {
    fn get_user_balance(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error>> {
        self.balance_repository.find_by_user_id(user_id)
    }

    fn get_or_create_balance(&self, user_id: Uuid) -> Result<Balance, Box<dyn Error>> {
        match self.balance_repository.find_by_user_id(user_id)? {
            Some(balance) => Ok(balance),
            None => {
                let balance = Balance::new(user_id);
                self.balance_repository.save(&balance)?;
                Ok(balance)
            }
        }
    }

    fn add_funds(&self, user_id: Uuid, amount: i64) -> Result<i64, Box<dyn Error>> {
        if amount <= 0 {
            return Err("Amount must be positive".into());
        }
        
        let mut balance = self.get_or_create_balance(user_id)?;
        let new_balance = balance.add_funds(amount).map_err(|e| e.to_string())?;
        self.save_balance(&balance)?;
        
        Ok(new_balance)
    }

    fn withdraw_funds(&self, user_id: Uuid, amount: i64) -> Result<i64, Box<dyn Error>> {
        if amount <= 0 {
            return Err("Amount must be positive".into());
        }
        
        let mut balance = self.get_or_create_balance(user_id)?;
        if balance.amount < amount {
            return Err("Insufficient funds".into());
        }
        
        let new_balance = balance.withdraw(amount).map_err(|e| e.to_string())?;
        self.save_balance(&balance)?;
        
        Ok(new_balance)
    }

    fn save_balance(&self, balance: &Balance) -> Result<(), Box<dyn Error>> {
        self.balance_repository.save(balance)
    }
}
