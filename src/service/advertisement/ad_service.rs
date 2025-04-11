use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

use crate::dto::advertisement::advertisement::{
    AdvertisementQueryParams, AdvertisementResponse, AdvertisementListResponse, PaginationData
};
use crate::model::advertisement::advertisement::Advertisement;
use crate::repository::advertisement::advertisement_repository::AdvertisementRepository;
use crate::infrastructure::strategy::advertisement::display_strategy::DisplayStrategyFactory;

#[async_trait]
pub trait AdvertisementService: Send + Sync {
    async fn get_all_advertisements(&self, params: AdvertisementQueryParams) -> Result<AdvertisementListResponse, anyhow::Error>;
}

pub struct AdvertisementServiceImpl<R: AdvertisementRepository> {
    repository: Arc<R>,
    strategy_factory: DisplayStrategyFactory,
}

impl<R: AdvertisementRepository> AdvertisementServiceImpl<R> {
    pub fn new(repository: Arc<R>) -> Self {
        AdvertisementServiceImpl {
            repository,
            strategy_factory: DisplayStrategyFactory::new(),
        }
    }
    
    fn to_response(&self, advertisement: Advertisement) -> AdvertisementResponse {
        AdvertisementResponse {
            id: advertisement.id,
            title: advertisement.title,
            image_url: advertisement.image_url,
            start_date: advertisement.start_date,
            end_date: advertisement.end_date,
            status: advertisement.status.to_string(),
            click_url: advertisement.click_url,
            created_at: advertisement.created_at,
            updated_at: advertisement.updated_at,
        }
    }
}

#[async_trait]
impl<R: AdvertisementRepository> AdvertisementService for AdvertisementServiceImpl<R> {
    async fn get_all_advertisements(&self, params: AdvertisementQueryParams) -> Result<AdvertisementListResponse, anyhow::Error> {
        let (advertisements, total) = self.repository.find_all(&params).await?;
        
        let strategy = self.strategy_factory.get_strategy("default");
        
        let mut transformed_ads = Vec::new();
        for ad in advertisements {
            let prepared_ad = strategy.prepare_for_display(ad).await;
            transformed_ads.push(self.to_response(prepared_ad));
        }
        
        let limit = params.limit.unwrap_or(10).min(50);
        let page = params.page.unwrap_or(1);
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        
        Ok(AdvertisementListResponse {
            advertisements: transformed_ads,
            pagination: PaginationData {
                current_page: page,
                total_pages,
                total_items: total,
                limit,
            },
        })
    }
}