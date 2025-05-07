use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

use crate::model::advertisement::advertisement::{Advertisement, AdvertisementStatus};

#[async_trait]
pub trait AdvertisementDisplayStrategy: Send + Sync {
    async fn prepare_for_display(&self, advertisement: Advertisement) -> Advertisement;
}

pub struct DefaultDisplayStrategy;

#[async_trait]
impl AdvertisementDisplayStrategy for DefaultDisplayStrategy {
    async fn prepare_for_display(&self, advertisement: Advertisement) -> Advertisement {
        advertisement
    }
}

pub struct ActiveOnlyDisplayStrategy;

#[async_trait]
impl AdvertisementDisplayStrategy for ActiveOnlyDisplayStrategy {
    async fn prepare_for_display(&self, mut advertisement: Advertisement) -> Advertisement {
        // Check if ad is expired (end_date < now)
        let now = Utc::now();
        if advertisement.end_date < now {
            advertisement.status = AdvertisementStatus::Expired;
        }
        
        advertisement
    }
}

pub struct DisplayStrategyFactory {
    strategies: HashMap<String, Arc<dyn AdvertisementDisplayStrategy>>,
}

impl DisplayStrategyFactory {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        strategies.insert(
            "default".to_string(), 
            Arc::new(DefaultDisplayStrategy) as Arc<dyn AdvertisementDisplayStrategy>
        );
        strategies.insert(
            "active_only".to_string(), 
            Arc::new(ActiveOnlyDisplayStrategy) as Arc<dyn AdvertisementDisplayStrategy>
        );
        
        DisplayStrategyFactory { strategies }
    }
    
    pub fn get_strategy(&self, strategy_name: &str) -> Arc<dyn AdvertisementDisplayStrategy> {
        self.strategies
            .get(strategy_name)
            .unwrap_or_else(|| self.strategies.get("default").unwrap())
            .clone()
    }
}