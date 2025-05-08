use async_trait::async_trait;
use std::sync::Arc;

use crate::dto::advertisement::advertisement::{
    AdvertisementQueryParams, AdvertisementResponse, AdvertisementDetailResponse, 
    AdvertisementListResponse, PaginationData
};
use crate::model::advertisement::advertisement::Advertisement;
use crate::repository::advertisement::advertisement_repository::AdvertisementRepository;

#[async_trait]
pub trait AdvertisementService: Send + Sync {
    async fn get_all_advertisements(&self, params: AdvertisementQueryParams) -> Result<AdvertisementListResponse, anyhow::Error>;
    async fn get_advertisement_by_id(&self, id: &str) -> Result<AdvertisementDetailResponse, anyhow::Error>;
}

// Service implementation that works with any repository implementing AdvertisementRepository
pub struct AdvertisementServiceImpl<R> {
    repository: Arc<R>,
}

impl<R> AdvertisementServiceImpl<R>
where
    R: AdvertisementRepository + Send + Sync,
{
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

// Trait for converting Advertisement to response types
trait AdvertisementConverter {
    fn to_response(&self, ad: Advertisement) -> AdvertisementResponse;
    fn to_detail_response(&self, ad: Advertisement) -> AdvertisementDetailResponse;
}

// Default implementation works for any type
impl<T> AdvertisementConverter for T {
    fn to_response(&self, ad: Advertisement) -> AdvertisementResponse {
        AdvertisementResponse {
            id: ad.id,
            title: ad.title,
            image_url: ad.image_url,
            start_date: ad.start_date,
            end_date: ad.end_date,
            status: ad.status.to_string(),
            click_url: ad.click_url,
            created_at: ad.created_at,
            updated_at: ad.updated_at,
        }
    }
    
    fn to_detail_response(&self, ad: Advertisement) -> AdvertisementDetailResponse {
        AdvertisementDetailResponse {
            id: ad.id,
            title: ad.title,
            description: ad.description,
            image_url: ad.image_url,
            start_date: ad.start_date,
            end_date: ad.end_date,
            status: ad.status.to_string(),
            click_url: ad.click_url,
            target_audience: ad.target_audience,
            impression_count: ad.impression_count,
            click_count: ad.click_count,
            created_at: ad.created_at,
            updated_at: ad.updated_at,
        }
    }
}

#[async_trait]
impl<R> AdvertisementService for AdvertisementServiceImpl<R> 
where
    R: AdvertisementRepository + Send + Sync,
{
    async fn get_all_advertisements(&self, params: AdvertisementQueryParams) -> Result<AdvertisementListResponse, anyhow::Error> {
        let (advertisements, total) = self.repository.find_all(&params).await?;
        
        let limit = params.limit.unwrap_or(10).min(50);
        let page = params.page.unwrap_or(1);
        
        Ok(AdvertisementListResponse {
            advertisements: advertisements.into_iter().map(|ad| self.to_response(ad)).collect(),
            pagination: PaginationData {
                current_page: page,
                total_pages: ((total as f64) / (limit as f64)).ceil() as u32,
                total_items: total,
                limit,
            },
        })
    }
    
    async fn get_advertisement_by_id(&self, id: &str) -> Result<AdvertisementDetailResponse, anyhow::Error> {
        let advertisement = self.repository.find_by_id(id).await?
            .ok_or_else(|| anyhow::anyhow!("Advertisement with ID {} not found", id))?;
        
        Ok(self.to_detail_response(advertisement))
    }
}

// Factory function using trait object
pub fn new_advertisement_service(repository: Arc<dyn AdvertisementRepository + Send + Sync>) -> impl AdvertisementService {
    struct DynamicService { repo: Arc<dyn AdvertisementRepository + Send + Sync> }
    
    #[async_trait]
    impl AdvertisementService for DynamicService {
        async fn get_all_advertisements(&self, params: AdvertisementQueryParams) -> Result<AdvertisementListResponse, anyhow::Error> {
            let (advertisements, total) = self.repo.find_all(&params).await?;
            let limit = params.limit.unwrap_or(10).min(50);
            let page = params.page.unwrap_or(1);
            
            Ok(AdvertisementListResponse {
                advertisements: advertisements.into_iter().map(|ad| self.to_response(ad)).collect(),
                pagination: PaginationData {
                    current_page: page,
                    total_pages: ((total as f64) / (limit as f64)).ceil() as u32,
                    total_items: total,
                    limit,
                },
            })
        }
        
        async fn get_advertisement_by_id(&self, id: &str) -> Result<AdvertisementDetailResponse, anyhow::Error> {
            let advertisement = self.repo.find_by_id(id).await?
                .ok_or_else(|| anyhow::anyhow!("Advertisement with ID {} not found", id))?;
            
            Ok(self.to_detail_response(advertisement))
        }
    }
    
    DynamicService { repo: repository }
}