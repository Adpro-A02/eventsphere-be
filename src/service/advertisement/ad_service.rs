use async_trait::async_trait;
use std::error::Error as StdError;

use crate::dto::advertisement::advertisement::{
    AdvertisementQueryParams, AdvertisementDetailResponse, 
    AdvertisementListResponse, CreateAdvertisementRequest, CreateAdvertisementResponse,
    PaginationData
};
use crate::model::advertisement::advertisement::AdvertisementStatus;

/// Type alias for service results with thread-safe error handling
pub type ServiceResult<T> = Result<T, Box<dyn StdError + Send + Sync>>;

/// Helper function to convert AdvertisementStatus to string
pub fn status_to_string(status: &AdvertisementStatus) -> String {
    match status {
        AdvertisementStatus::Active => "active".to_string(),
        AdvertisementStatus::Inactive => "inactive".to_string(),
        AdvertisementStatus::Expired => "expired".to_string(),
    }
}

/// Helper function to convert regular errors to thread-safe errors
pub fn map_error<E: std::fmt::Display>(err: E) -> Box<dyn StdError + Send + Sync> {
    Box::<dyn StdError + Send + Sync>::from(format!("{}", err))
}

/// Create pagination data from query parameters and total count
pub fn create_pagination(params: &AdvertisementQueryParams, total: i64) -> PaginationData {
    let limit = params.limit.unwrap_or(10).min(50);
    let page = params.page.unwrap_or(1);
    
    PaginationData {
        current_page: page,
        total_pages: ((total as f64) / (limit as f64)).ceil() as u32,
        total_items: total as u64,
        limit,
    }
}

/// Advertisement service interface defining core operations
#[async_trait]
pub trait AdvertisementService: Send + Sync {
    /// Retrieve all advertisements based on filtering parameters
    async fn get_all_advertisements(&self, params: AdvertisementQueryParams) -> ServiceResult<AdvertisementListResponse>;
    
    /// Get a single advertisement by its ID
    async fn get_advertisement_by_id(&self, id: &str) -> ServiceResult<AdvertisementDetailResponse>;
    
    /// Create a new advertisement with image data
    async fn create_advertisement(&self, request: CreateAdvertisementRequest, image_data: Vec<u8>) 
        -> ServiceResult<CreateAdvertisementResponse>;

    async fn update_advertisement(&self, id: &str, request: UpdateAdvertisementRequest, image_data: Option<Vec<u8>>) 
    -> ServiceResult<UpdateAdvertisementResponse>;
    }
