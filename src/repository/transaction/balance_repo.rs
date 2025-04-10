use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
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
