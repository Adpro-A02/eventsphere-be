#[cfg(feature = "redis")]
use redis::{Client, Connection, RedisError};
use tracing::{info, warn};

use crate::config::Config;

/// Initialize Redis client
#[cfg(feature = "redis")]
pub async fn init_redis(config: &Config) -> Option<Client> {
    if let Some(redis_url) = &config.redis_url {
        info!("Initializing Redis connection");
        
        match Client::open(redis_url.as_str()) {
            Ok(client) => {
                // Test the connection
                match client.get_connection() {
                    Ok(_) => {
                        info!("Redis connection established");
                        Some(client)
                    },
                    Err(e) => {
                        warn!("Failed to connect to Redis: {}", e);
                        None
                    }
                }
            },
            Err(e) => {
                warn!("Failed to initialize Redis client: {}", e);
                None
            }
        }
    } else {
        info!("Redis URL not provided, skipping Redis initialization");
        None
    }
}

#[cfg(not(feature = "redis"))]
pub async fn init_redis(_config: &Config) -> Option<()> {
    info!("Redis feature not enabled, skipping Redis initialization");
    None
}

#[cfg(feature = "redis")]
pub fn get_connection(client: &Client) -> Result<Connection, RedisError> {
    client.get_connection()
}

#[cfg(feature = "redis")]
pub use redis::{Commands, RedisResult};

// Provide stub types when Redis is not enabled
#[cfg(not(feature = "redis"))]
pub mod stubs {
    pub trait Commands {}
    pub type RedisResult<T> = Result<T, String>;
}

#[cfg(not(feature = "redis"))]
pub use stubs::*;