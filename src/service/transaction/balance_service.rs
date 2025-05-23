use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;
use async_trait::async_trait;

use crate::model::transaction::Balance;
use crate::repository::transaction::balance_repo::BalanceRepository;

#[async_trait]
pub trait BalanceService {
    async fn get_user_balance(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error + Send + Sync>>;
    async fn get_or_create_balance(&self, user_id: Uuid) -> Result<Balance, Box<dyn Error + Send + Sync>>;
    async fn add_funds(&self, user_id: Uuid, amount: i64) -> Result<i64, Box<dyn Error + Send + Sync>>;
    async fn withdraw_funds(&self, user_id: Uuid, amount: i64) -> Result<i64, Box<dyn Error + Send + Sync>>;
    async fn save_balance(&self, balance: &Balance) -> Result<(), Box<dyn Error + Send + Sync>>;
}

pub struct DefaultBalanceService {
    balance_repository: Arc<dyn BalanceRepository + Send + Sync>,
}

impl DefaultBalanceService {
    pub fn new(balance_repository: Arc<dyn BalanceRepository + Send + Sync>) -> Self {
        Self {
            balance_repository,
        }
    }
}

#[async_trait]
impl BalanceService for DefaultBalanceService {
    async fn get_user_balance(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error + Send + Sync>> {
        self.balance_repository.find_by_user_id(user_id).await
    }

    async fn get_or_create_balance(&self, user_id: Uuid) -> Result<Balance, Box<dyn Error + Send + Sync>> {
        match self.balance_repository.find_by_user_id(user_id).await? {
            Some(balance) => Ok(balance),
            None => {
                let balance = Balance::new(user_id);
                self.balance_repository.save(&balance).await?;
                Ok(balance)
            }
        }
    }

    async fn add_funds(&self, user_id: Uuid, amount: i64) -> Result<i64, Box<dyn Error + Send + Sync>> {
        if amount <= 0 {
            return Err("Amount must be positive".into());
        }
        
        let mut balance = self.get_or_create_balance(user_id).await?;
        let new_balance = balance.add_funds(amount).map_err(|e| e.to_string())?;
        self.save_balance(&balance).await?;
        
        Ok(new_balance)
    }

    async fn withdraw_funds(&self, user_id: Uuid, amount: i64) -> Result<i64, Box<dyn Error + Send + Sync>> {
        if amount <= 0 {
            return Err("Amount must be positive".into());
        }
        
        let mut balance = self.get_or_create_balance(user_id).await?;
        if balance.amount < amount {
            return Err("Insufficient funds".into());
        }
        
        let new_balance = balance.withdraw(amount).map_err(|e| e.to_string())?;
        self.save_balance(&balance).await?;
        
        Ok(new_balance)
    }

    async fn save_balance(&self, balance: &Balance) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.balance_repository.save(balance).await
    }
}
