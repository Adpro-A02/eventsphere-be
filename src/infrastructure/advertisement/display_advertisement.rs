use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

use crate::model::advertisement::advertisement::{Advertisement, AdvertisementStatus};

#[async_trait]
pub trait AdvertisementDisplay: Send + Sync {
    async fn prepare_for_display(&self, advertisement: Advertisement) -> Advertisement;
}

pub struct DefaultDisplay;

#[async_trait]
impl AdvertisementDisplay for DefaultDisplay {
    async fn prepare_for_display(&self, advertisement: Advertisement) -> Advertisement {
        advertisement
    }
}

pub struct ActiveOnlyDisplay;

#[async_trait]
impl AdvertisementDisplay for ActiveOnlyDisplay {
    async fn prepare_for_display(&self, mut advertisement: Advertisement) -> Advertisement {
        // Check if ad is expired (end_date < now)
        let now = Utc::now();
        if advertisement.end_date < now {
            advertisement.status = AdvertisementStatus::Expired;
        }
        
        advertisement
    }
}

pub struct DisplayFactory {
    strategies: HashMap<String, Arc<dyn AdvertisementDisplay>>,
}

impl DisplayFactory {
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        strategies.insert(
            "default".to_string(), 
            Arc::new(DefaultDisplay) as Arc<dyn AdvertisementDisplay>
        );
        strategies.insert(
            "active_only".to_string(), 
            Arc::new(ActiveOnlyDisplay) as Arc<dyn AdvertisementDisplay>
        );
        
        DisplayFactory { strategies }
    }
    
    pub fn get_(&self, _name: &str) -> Arc<dyn AdvertisementDisplay> {
        self.strategies
            .get(_name)
            .unwrap_or_else(|| self.strategies.get("default").unwrap())
            .clone()
    }
}