use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
use sqlx::{PgPool, Row};
use async_trait::async_trait;
use uuid::Uuid;

use crate::model::transaction::Balance;

pub trait BalancePersistenceStrategy {
    fn save(&self, balance: &Balance) -> Result<(), Box<dyn Error>>;
    fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error>>;
}

pub struct InMemoryBalancePersistence {
    balances: RwLock<HashMap<Uuid, Balance>>,
}

impl InMemoryBalancePersistence {
    pub fn new() -> Self {
        Self {
            balances: RwLock::new(HashMap::new()),
        }
    }
}

impl BalancePersistenceStrategy for InMemoryBalancePersistence {
    fn save(&self, balance: &Balance) -> Result<(), Box<dyn Error>> {
        let mut balances = self.balances.write().unwrap();
        balances.insert(balance.user_id, balance.clone());
        Ok(())
    }

    fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error>> {
        let balances = self.balances.read().unwrap();
        Ok(balances.get(&user_id).cloned())
    }
}

pub trait BalanceRepository {
    fn save(&self, balance: &Balance) -> Result<(), Box<dyn Error>>;
    fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error>>;
}

pub struct DbBalanceRepository<S: BalancePersistenceStrategy> {
    strategy: S,
}

impl<S: BalancePersistenceStrategy> DbBalanceRepository<S> {
    pub fn new(strategy: S) -> Self {
        DbBalanceRepository { strategy }
    }
}

impl<S: BalancePersistenceStrategy> BalanceRepository for DbBalanceRepository<S> {
    fn save(&self, balance: &Balance) -> Result<(), Box<dyn Error>> {
        self.strategy.save(balance)
    }

    fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error>> {
        self.strategy.find_by_user_id(user_id)
    }
}

pub struct PostgresBalancePersistence {
    pool: PgPool,
}

impl PostgresBalancePersistence {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
pub trait AsyncBalancePersistenceStrategy {
    async fn save(&self, balance: &Balance) -> Result<(), Box<dyn Error>>;
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error>>;
}

#[async_trait]
impl AsyncBalancePersistenceStrategy for PostgresBalancePersistence {
    async fn save(&self, balance: &Balance) -> Result<(), Box<dyn Error>> {
        let query = "INSERT INTO balances (id, user_id, amount, updated_at) 
                    VALUES ($1, $2, $3, $4) 
                    ON CONFLICT (user_id) 
                    DO UPDATE SET amount = EXCLUDED.amount, updated_at = EXCLUDED.updated_at";
        
        let result = sqlx::query(query)
            .bind(balance.id)
            .bind(balance.user_id)
            .bind(balance.amount)
            .bind(balance.updated_at)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err("Failed to save balance".into());
        }

        Ok(())
    }

    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error>> {
        let query = "SELECT * FROM balances WHERE user_id = $1";
        
        let row = sqlx::query(query)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
            
        if let Some(row) = row {
            let balance = Balance {
                id: row.get("id"),
                user_id: row.get("user_id"),
                amount: row.get("amount"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(balance))
        } else {
            Ok(None)
        }
    }
}