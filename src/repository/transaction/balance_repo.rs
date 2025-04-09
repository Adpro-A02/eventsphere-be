use std::error::Error;
use uuid::Uuid;
use crate::model::transaction::Balance;
use std::collections::HashMap;
use std::sync::RwLock;

pub trait BalanceRepository {
    fn save(&self, balance: &Balance) -> Result<(), Box<dyn Error>>;
    fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Balance>, Box<dyn Error>>;
}

pub struct DbBalanceRepository {
    balances: RwLock<HashMap<Uuid, Balance>>,
}

impl DbBalanceRepository {
    pub fn new() -> Self {
        Self {
            balances: RwLock::new(HashMap::new()),
        }
    }
}

impl BalanceRepository for DbBalanceRepository {
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
