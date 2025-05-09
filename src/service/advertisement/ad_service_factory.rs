use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::dto::advertisement::advertisement::{
    AdvertisementQueryParams, AdvertisementResponse, AdvertisementDetailResponse, 
    AdvertisementListResponse, CreateAdvertisementRequest, CreateAdvertisementResponse
};
use crate::model::advertisement::advertisement::{Advertisement, AdvertisementStatus};
use crate::repository::advertisement::ad_repository::AdvertisementRepository;
use crate::service::advertisement::ad_service::{
    AdvertisementService, ServiceResult, map_error, status_to_string, create_pagination
};

/// Factory function to create a service with dynamic dispatch
pub fn new_advertisement_service(repository: Arc<dyn AdvertisementRepository + Send + Sync>) 
    -> impl AdvertisementService {
    struct DynamicService { 
        repo: Arc<dyn AdvertisementRepository + Send + Sync> 
    }
    
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
    
    impl AdvertisementConverter for DynamicService {}
    
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