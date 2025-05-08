use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advertisement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub image_url: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub status: AdvertisementStatus,
    pub click_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub position: String,
    pub impressions: i32,
    pub clicks: i32,


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

impl Advertisement {
    pub fn new(
        id: String,
        title: String,
        description: String,
        image_url: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        status: AdvertisementStatus,
        click_url: String,
        position: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            description,
            image_url,
            start_date,
            end_date,
            status,
            click_url,
            created_at: now,
            updated_at: now,
            position,
            impressions: 0,
            clicks: 0,
        }
    }
}