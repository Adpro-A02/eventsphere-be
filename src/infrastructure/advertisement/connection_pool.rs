use once_cell::sync::Lazy;
use sqlx::{Pool, Postgres};
use std::env;
use std::sync::Arc;
use tokio::sync::Semaphore;

// Global connection pool with configurable size
static AD_DB_POOL: Lazy<Pool<Postgres>> = Lazy::new(|| {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let max_connections = env::var("AD_DB_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u32>()
        .unwrap_or(10);
    
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect_lazy(&database_url)
        .expect("Failed to create advertisement database pool")
});

// Semaphore for controlling concurrent operations
static CONCURRENT_UPLOADS: Lazy<Arc<Semaphore>> = Lazy::new(|| {
    let max_concurrent = env::var("MAX_CONCURRENT_AD_UPLOADS")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<usize>()
        .unwrap_or(5);
    
    Arc::new(Semaphore::new(max_concurrent))
});

pub fn get_ad_db_pool() -> &'static Pool<Postgres> {
    &AD_DB_POOL
}

pub async fn acquire_upload_permit() -> tokio::sync::SemaphorePermit {
    CONCURRENT_UPLOADS.acquire().await.expect("Failed to acquire upload permit")
}