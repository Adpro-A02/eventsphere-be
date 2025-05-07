use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advertisement {
    pub id: String,
    pub title: String,
    pub image_url: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: AdvertisementStatus,
    pub click_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdvertisementStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
    #[serde(rename = "expired")]
    Expired,
}

impl From<String> for AdvertisementStatus {
    fn from(status: String) -> Self {
        match status.to_lowercase().as_str() {
            "active" => AdvertisementStatus::Active,
            "inactive" => AdvertisementStatus::Inactive,
            "expired" => AdvertisementStatus::Expired,
            _ => AdvertisementStatus::Inactive,
        }
    }
}