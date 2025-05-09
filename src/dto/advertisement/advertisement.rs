use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::model::advertisement::AdvertisementStatus;

#[derive(Debug, Clone, Deserialize)]
pub struct AdvertisementQueryParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub status: Option<String>,
    pub start_date_from: Option<DateTime<Utc>>,
    pub start_date_to: Option<DateTime<Utc>>,
    pub end_date_from: Option<DateTime<Utc>>,
    pub end_date_to: Option<DateTime<Utc>>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdvertisementResponse {
    pub id: String,
    pub title: String,
    pub image_url: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: String,
    pub click_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdvertisementDetailResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub image_url: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: String,
    pub click_url: String,
    pub position: String,
    pub impressions: i32,
    pub clicks: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationData {
    pub current_page: u32,
    pub total_pages: u32,
    pub total_items: u64,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdvertisementListResponse {
    pub advertisements: Vec<AdvertisementResponse>,
    pub pagination: PaginationData,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub code: u32,
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

// Conversion implementations for DTOs
impl From<crate::model::advertisement::Advertisement> for AdvertisementResponse {
    fn from(ad: crate::model::advertisement::Advertisement) -> Self {
        Self {
            id: ad.id,
            title: ad.title,
            image_url: ad.image_url,
            start_date: ad.start_date,
            end_date: ad.end_date,
            status: match ad.status {
                AdvertisementStatus::Active => "active".to_string(),
                AdvertisementStatus::Inactive => "inactive".to_string(),
                AdvertisementStatus::Expired => "expired".to_string(),
            },
            click_url: ad.click_url,
            created_at: ad.created_at,
            updated_at: ad.updated_at,
        }
    }
}

impl From<crate::model::advertisement::Advertisement> for AdvertisementDetailResponse {
    fn from(ad: crate::model::advertisement::Advertisement) -> Self {
        Self {
            id: ad.id,
            title: ad.title,
            description: ad.description,
            image_url: ad.image_url,
            start_date: ad.start_date,
            end_date: ad.end_date,
            status: match ad.status {
                AdvertisementStatus::Active => "active".to_string(),
                AdvertisementStatus::Inactive => "inactive".to_string(),
                AdvertisementStatus::Expired => "expired".to_string(),
            },
            click_url: ad.click_url,
            position: ad.position,
            impressions: ad.impressions,
            clicks: ad.clicks,
            created_at: ad.created_at,
            updated_at: ad.updated_at,
        }
    }
}

// Validation error structure for form validation errors
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAdvertisementRequest {
    pub title: String,
    pub description: Option<String>,
    #[serde(skip)]
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub click_url: String,
    pub position: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateAdvertisementResponse {
    pub id: String,
    pub title: String,
    pub image_url: String,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: String,
    pub click_url: String,
    pub position: String,
    pub created_at: DateTime<Utc>,
}
