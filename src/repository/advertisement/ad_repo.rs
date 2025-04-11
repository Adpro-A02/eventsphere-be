use async_trait::async_trait;
use crate::dto::advertisement::advertisement::AdvertisementQueryParams;
use crate::model::advertisement::advertisement::{Advertisement, AdvertisementStatus};

#[async_trait]
pub trait AdvertisementRepository: Send + Sync {
    async fn find_all(&self, params: &AdvertisementQueryParams) -> Result<(Vec<Advertisement>, i64), anyhow::Error>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Advertisement>, anyhow::Error>;
    async fn create(&self, advertisement: &Advertisement) -> Result<Advertisement, anyhow::Error>;
    async fn update(&self, advertisement: &Advertisement) -> Result<Advertisement, anyhow::Error>;
    async fn delete(&self, id: &str) -> Result<bool, anyhow::Error>;
}