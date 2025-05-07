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
    pub end_date: DateTime<Utc>,
    pub status: String,
    pub click_url: String,
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