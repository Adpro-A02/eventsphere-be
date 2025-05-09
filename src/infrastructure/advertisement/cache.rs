use redis::{AsyncCommands, Client};
use std::env;
use std::sync::Arc;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

use crate::model::advertisement::Advertisement;

// Global Redis client
static REDIS_CLIENT: Lazy<Arc<RedisAdvertisementCache>> = Lazy::new(|| {
    Arc::new(RedisAdvertisementCache::new())
});

pub struct RedisAdvertisementCache {
    client: Client,
    enabled: bool,
}

impl RedisAdvertisementCache {
    fn new() -> Self {
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let enabled = env::var("ENABLE_AD_CACHE").unwrap_or_else(|_| "true".to_string()) == "true";
        
        let client = match Client::open(redis_url) {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Failed to connect to Redis: {}. Cache disabled.", e);
                return Self { client: Client::open("redis://nowhere").unwrap(), enabled: false };
            }
        };
        
        Self { client, enabled }
    }
    
    pub async fn get_advertisement(&self, id: &str) -> Option<Advertisement> {
        if !self.enabled {
            return None;
        }
        
        let mut conn = match self.client.get_async_connection().await {
            Ok(conn) => conn,
            Err(_) => return None,
        };
        
        let key = format!("ad:{}", id);
        let ad_json: Option<String> = conn.get(&key).await.ok();
        
        ad_json.and_then(|json| serde_json::from_str(&json).ok())
    }
    
    pub async fn cache_advertisement(&self, ad: &Advertisement, ttl_seconds: u64) -> Result<(), redis::RedisError> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("ad:{}", ad.id);
        
        let json = serde_json::to_string(ad).unwrap_or_default();
        conn.set_ex(key, json, ttl_seconds).await
    }
    
    pub async fn invalidate(&self, id: &str) -> Result<(), redis::RedisError> {
        if !self.enabled {
            return Ok(());
        }
        
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("ad:{}", id);
        conn.del(key).await
    }
}

pub fn get_cache() -> Arc<RedisAdvertisementCache> {
    REDIS_CLIENT.clone()
}