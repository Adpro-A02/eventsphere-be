use async_trait::async_trait;
use std::sync::Arc;
use std::error::Error as StdError;
use uuid::Uuid;

use crate::dto::advertisement::advertisement::{
    AdvertisementQueryParams, AdvertisementResponse, AdvertisementDetailResponse, 
    AdvertisementListResponse, PaginationData, CreateAdvertisementRequest, CreateAdvertisementResponse
};
use crate::model::advertisement::advertisement::{Advertisement, AdvertisementStatus};
use crate::repository::advertisement::ad_repository::AdvertisementRepository;

/// Type alias for service results with thread-safe error handling
type ServiceResult<T> = Result<T, Box<dyn StdError + Send + Sync>>;

/// Helper function to convert regular errors to thread-safe errors
fn map_error<E: std::fmt::Display>(err: E) -> Box<dyn StdError + Send + Sync> {
    Box::<dyn StdError + Send + Sync>::from(format!("{}", err))
}

/// Convert Advertisement to different response DTOs
trait AdvertisementConverter {
    fn to_response(&self, ad: &Advertisement) -> AdvertisementResponse {
        AdvertisementResponse {
            id: ad.id.clone(),
            title: ad.title.clone(),
            image_url: ad.image_url.clone(),
            start_date: ad.start_date,
            end_date: ad.end_date,
            status: status_to_string(&ad.status),
            click_url: ad.click_url.clone(),
            created_at: ad.created_at,
            updated_at: ad.updated_at,
        }
    }
    
    fn to_detail_response(&self, ad: &Advertisement) -> AdvertisementDetailResponse {
        AdvertisementDetailResponse {
            id: ad.id.clone(),
            title: ad.title.clone(),
            description: ad.description.clone(),
            image_url: ad.image_url.clone(),
            start_date: ad.start_date,
            end_date: ad.end_date,
            status: status_to_string(&ad.status),
            click_url: ad.click_url.clone(),
            position: ad.position.clone(),     
            impressions: ad.impressions,       
            clicks: ad.clicks,                 
            created_at: ad.created_at,
            updated_at: ad.updated_at,
        }
    }
    
    fn to_create_response(&self, ad: &Advertisement) -> CreateAdvertisementResponse {
        CreateAdvertisementResponse {
            id: ad.id.clone(),
            title: ad.title.clone(),
            image_url: ad.image_url.clone(),
            start_date: ad.start_date,
            end_date: ad.end_date,
            status: status_to_string(&ad.status),
            click_url: ad.click_url.clone(),
            position: ad.position.clone(),
            created_at: ad.created_at,
        }
    }
}

/// Helper function to convert AdvertisementStatus to string
fn status_to_string(status: &AdvertisementStatus) -> String {
    match status {
        AdvertisementStatus::Active => "active".to_string(),
        AdvertisementStatus::Inactive => "inactive".to_string(),
        AdvertisementStatus::Expired => "expired".to_string(),
    }
}

/// Create pagination data from query parameters and total count
fn create_pagination(params: &AdvertisementQueryParams, total: i64) -> PaginationData {
    let limit = params.limit.unwrap_or(10).min(50);
    let page = params.page.unwrap_or(1);
    
    PaginationData {
        current_page: page,
        total_pages: ((total as f64) / (limit as f64)).ceil() as u32,
        total_items: total as u64,
        limit,
    }
}

/// Advertisement service interface
#[async_trait]
pub trait AdvertisementService: Send + Sync {
    async fn get_all_advertisements(&self, params: AdvertisementQueryParams) -> ServiceResult<AdvertisementListResponse>;
    async fn get_advertisement_by_id(&self, id: &str) -> ServiceResult<AdvertisementDetailResponse>;
    async fn create_advertisement(&self, request: CreateAdvertisementRequest, image_data: Vec<u8>) 
        -> ServiceResult<CreateAdvertisementResponse>;
}

/// Service implementation that works with any repository implementing AdvertisementRepository
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
    
    // Upload image to storage and return URL
    async fn upload_image(&self, image_data: Vec<u8>) -> Result<String, Box<dyn StdError + Send + Sync>> {
        //TODO: store as BYTE at db, ini masih nyimpen di local storage
        let filename = format!("ad_{}.jpg", Uuid::new_v4());
        let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
        

        std::fs::create_dir_all(&upload_dir)?;
        let file_path = format!("{}/{}", upload_dir, filename);
        
        std::fs::write(&file_path, &image_data)?;
        let base_url = std::env::var("MEDIA_BASE_URL").unwrap_or_else(|_| "http://localhost:8000/media".to_string());
        
        Ok(format!("{}/{}", base_url, filename))
    }
}

// Default implementation works for any type
impl<T> AdvertisementConverter for T {}

#[async_trait]
impl<R> AdvertisementService for AdvertisementServiceImpl<R> 
where
    R: AdvertisementRepository + Send + Sync,
{
    async fn get_all_advertisements(&self, params: AdvertisementQueryParams) -> ServiceResult<AdvertisementListResponse> {
        let (advertisements, total) = self.repository.find_all(&params).await
            .map_err(map_error)?;
        
        Ok(AdvertisementListResponse {
            advertisements: advertisements.iter().map(|ad| self.to_response(ad)).collect(),
            pagination: create_pagination(&params, total),
        })
    }
    
    async fn get_advertisement_by_id(&self, id: &str) -> ServiceResult<AdvertisementDetailResponse> {
        let advertisement = self.repository.find_by_id(id).await
            .map_err(map_error)?
            .ok_or_else(|| map_error(format!("Advertisement with ID {} not found", id)))?;
        
        Ok(self.to_detail_response(&advertisement))
    }
    
    async fn create_advertisement(&self, request: CreateAdvertisementRequest, image_data: Vec<u8>) 
        -> ServiceResult<CreateAdvertisementResponse> {
        // Upload image to storage and get URL
        let image_url = self.upload_image(image_data).await
            .map_err(|e| map_error(format!("Failed to upload image: {}", e)))?;
        
        // Generate a new UUID for the advertisement
        let id = Uuid::new_v4().to_string();
        
        // Create advertisement model
        let advertisement = Advertisement::new(
            id,
            request.title,
            request.description.unwrap_or_default(),
            image_url,
            request.start_date,
            Some(request.end_date),
            AdvertisementStatus::Active,
            request.click_url,
            request.position,
        );
        

        let created = self.repository.create(&advertisement).await
            .map_err(map_error)?;
        
        Ok(self.to_create_response(&created))
    }
}

/// Factory function to create a service with dynamic dispatch
pub fn new_advertisement_service(repository: Arc<dyn AdvertisementRepository + Send + Sync>) 
    -> impl AdvertisementService {
    struct DynamicService { repo: Arc<dyn AdvertisementRepository + Send + Sync> }
    
    #[async_trait]
    impl AdvertisementService for DynamicService {
        async fn get_all_advertisements(&self, params: AdvertisementQueryParams) -> ServiceResult<AdvertisementListResponse> {
            let (advertisements, total) = self.repo.find_all(&params).await
                .map_err(map_error)?;
                
            Ok(AdvertisementListResponse {
                advertisements: advertisements.iter().map(|ad| self.to_response(ad)).collect(),
                pagination: create_pagination(&params, total),
            })
        }
        
        async fn get_advertisement_by_id(&self, id: &str) -> ServiceResult<AdvertisementDetailResponse> {
            let advertisement = self.repo.find_by_id(id).await
                .map_err(map_error)?
                .ok_or_else(|| map_error(format!("Advertisement with ID {} not found", id)))?;
            
            Ok(self.to_detail_response(&advertisement))
        }
        
        async fn create_advertisement(&self, request: CreateAdvertisementRequest, image_data: Vec<u8>) 
            -> ServiceResult<CreateAdvertisementResponse> {
            // Upload image to storage and get URL
            let filename = format!("ad_{}.jpg", Uuid::new_v4());
            let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
            
            // Create directory if it doesn't exist
            std::fs::create_dir_all(&upload_dir)
                .map_err(|e| map_error(format!("Failed to create upload directory: {}", e)))?;
            
            // Save file to disk
            let file_path = format!("{}/{}", upload_dir, filename);
            std::fs::write(&file_path, &image_data)
                .map_err(|e| map_error(format!("Failed to write image file: {}", e)))?;
            
            // Get base URL from environment
            let base_url = std::env::var("MEDIA_BASE_URL").unwrap_or_else(|_| "http://localhost:8000/media".to_string());
            let image_url = format!("{}/{}", base_url, filename);
            
            // Generate a new UUID for the advertisement
            let id = Uuid::new_v4().to_string();
            
            // Create advertisement model
            let advertisement = Advertisement::new(
                id,
                request.title,
                request.description.unwrap_or_default(),
                image_url,
                request.start_date,
                Some(request.end_date),
                AdvertisementStatus::Active, 
                request.click_url,
                request.position,
            );
            
            // Save to repository
            let created = self.repo.create(&advertisement).await
                .map_err(map_error)?;
            
            // Map to response
            Ok(self.to_create_response(&created))
        }
    }
    
    DynamicService { repo: repository }
}