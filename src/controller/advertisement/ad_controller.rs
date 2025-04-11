use rocket::{State, get};
use rocket::http::Status;
use rocket::serde::json::Json;
use std::sync::Arc;

use crate::dto::advertisement::advertisement::{AdvertisementQueryParams, AdvertisementListResponse, ApiResponse};
use crate::service::advertisement::advertisement_service::AdvertisementService;
use crate::middleware::auth::JwtToken;

#[get("/api/v1/advertisements?<page>&<limit>&<status>&<start_date_from>&<start_date_to>&<end_date_from>&<end_date_to>&<search>")]
pub async fn get_all_advertisements(
    token: JwtToken,
    service: &State<Arc<dyn AdvertisementService>>,
    page: Option<u32>,
    limit: Option<u32>,
    status: Option<String>,
    start_date_from: Option<String>,
    start_date_to: Option<String>,
    end_date_from: Option<String>,
    end_date_to: Option<String>,
    search: Option<String>,
) -> Result<Json<ApiResponse<AdvertisementListResponse>>, Status> {
    // Check if the user is admin
    if !token.is_admin() {
        return Err(Status::Forbidden);
    }
    
    // Convert date strings to DateTime objects
    let start_date_from = start_date_from.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));
    let start_date_to = start_date_to.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));
    let end_date_from = end_date_from.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));
    let end_date_to = end_date_to.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&chrono::Utc)));
    
    let params = AdvertisementQueryParams {
        page,
        limit,
        status,
        start_date_from,
        start_date_to,
        end_date_from,
        end_date_to,
        search,
    };
    
    match service.get_all_advertisements(params).await {
        Ok(result) => Ok(Json(ApiResponse {
            code: 200,
            success: true,
            message: "Daftar iklan berhasil diambil".to_string(),
            data: Some(result),
        })),
        Err(_) => Err(Status::InternalServerError),
    }
}