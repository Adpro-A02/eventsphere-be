use std::sync::Arc;
use chrono::{DateTime, Utc};
use mockall::predicate::*;
use mockall::mock;

use crate::dto::advertisement::advertisement::{AdvertisementQueryParams, AdvertisementListResponse};
use crate::model::advertisement::advertisement::{Advertisement, AdvertisementStatus};
use crate::repository::advertisement::advertisement_repository::AdvertisementRepository;
use crate::service::advertisement::advertisement_service::{AdvertisementService, AdvertisementServiceImpl};

mock! {
    pub AdvertisementRepo {}
    
    #[async_trait]
    impl AdvertisementRepository for AdvertisementRepo {
        async fn find_all(&self, params: &AdvertisementQueryParams) -> Result<(Vec<Advertisement>, i64), anyhow::Error>;
        async fn find_by_id(&self, id: &str) -> Result<Option<Advertisement>, anyhow::Error>;
        async fn create(&self, advertisement: &Advertisement) -> Result<Advertisement, anyhow::Error>;
        async fn update(&self, advertisement: &Advertisement) -> Result<Advertisement, anyhow::Error>;
        async fn delete(&self, id: &str) -> Result<bool, anyhow::Error>;
    }
}

#[tokio::test]
async fn test_get_all_advertisements_success() {
    // Arrange
    let mut mock_repo = MockAdvertisementRepo::new();
    
    let now = Utc::now();
    let test_ads = vec![
        Advertisement {
            id: "1".to_string(),
            title: "Test Ad 1".to_string(),
            image_url: "http://example.com/ad1.jpg".to_string(),
            start_date: now,
            end_date: now + chrono::Duration::days(7),
            status: AdvertisementStatus::Active,
            click_url: "http://example.com/click1".to_string(),
            created_at: now,
            updated_at: now,
        },
        Advertisement {
            id: "2".to_string(),
            title: "Test Ad 2".to_string(),
            image_url: "http://example.com/ad2.jpg".to_string(),
            start_date: now,
            end_date: now + chrono::Duration::days(7),
            status: AdvertisementStatus::Inactive,
            click_url: "http://example.com/click2".to_string(),
            created_at: now,
            updated_at: now,
        },
    ];
    
    let params = AdvertisementQueryParams {
        page: Some(1),
        limit: Some(10),
        status: None,
        start_date_from: None,
        start_date_to: None,
        end_date_from: None,
        end_date_to: None,
        search: None,
    };
    
    mock_repo.expect_find_all()
        .with(function(|p: &AdvertisementQueryParams| {
            p.page == Some(1) && p.limit == Some(10)
        }))
        .times(1)
        .returning(move |_| Ok((test_ads.clone(), 2)));
    
    let service = AdvertisementServiceImpl::new(Arc::new(mock_repo));
    
    // Act
    let result = service.get_all_advertisements(params).await;
    
    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.advertisements.len(), 2);
    assert_eq!(response.pagination.total_items, 2);
    assert_eq!(response.pagination.current_page, 1);
}

#[tokio::test]
async fn test_get_all_advertisements_empty() {
    // Arrange
    let mut mock_repo = MockAdvertisementRepo::new();
    
    let params = AdvertisementQueryParams {
        page: Some(1),
        limit: Some(10),
        status: None,
        start_date_from: None,
        start_date_to: None,
        end_date_from: None,
        end_date_to: None,
        search: None,
    };
    
    mock_repo.expect_find_all()
        .with(function(|p: &AdvertisementQueryParams| {
            p.page == Some(1) && p.limit == Some(10)
        }))
        .times(1)
        .returning(|_| Ok((vec![], 0)));
    
    let service = AdvertisementServiceImpl::new(Arc::new(mock_repo));
    
    // Act
    let result = service.get_all_advertisements(params).await;
    
    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.advertisements.len(), 0);
    assert_eq!(response.pagination.total_items, 0);
}

#[tokio::test]
async fn test_get_all_advertisements_repository_error() {
    // Arrange
    let mut mock_repo = MockAdvertisementRepo::new();
    
    let params = AdvertisementQueryParams {
        page: Some(1),
        limit: Some(10),
        status: None,
        start_date_from: None,
        start_date_to: None,
        end_date_from: None,
        end_date_to: None,
        search: None,
    };
    
    mock_repo.expect_find_all()
        .with(function(|p: &AdvertisementQueryParams| {
            p.page == Some(1) && p.limit == Some(10)
        }))
        .times(1)
        .returning(|_| Err(anyhow::anyhow!("Database error")));
    
    let service = AdvertisementServiceImpl::new(Arc::new(mock_repo));
    
    // Act
    let result = service.get_all_advertisements(params).await;
    
    // Assert
    assert!(result.is_err());
}