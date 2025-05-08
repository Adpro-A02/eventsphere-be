use rocket::{State, get};
use std::sync::Arc;

use crate::common::api_response::ApiResponse;
use crate::dto::advertisement::advertisement::{
    AdvertisementQueryParams, AdvertisementListResponse, AdvertisementDetailResponse
};
use crate::service::advertisement::advertisement_service::AdvertisementService;
use crate::middleware::auth::{JwtToken, AuthenticatedUser};

/// Get all advertisements with filtering and pagination
#[get("/advertisements?<page>&<limit>&<status>&<start_date_from>&<start_date_to>&<end_date_from>&<end_date_to>&<search>")]
pub async fn get_all_advertisements(
    auth: AuthenticatedUser,
    service: &State<Arc<dyn AdvertisementService>>,
    page: Option<u32>,
    limit: Option<u32>,
    status: Option<String>,
    start_date_from: Option<String>,
    start_date_to: Option<String>,
    end_date_from: Option<String>,
    end_date_to: Option<String>,
    search: Option<String>,
) -> ApiResponse<AdvertisementListResponse> {
    // Check if the user is admin
    if !auth.is_admin() {
        return ApiResponse::forbidden("Anda tidak memiliki akses untuk melihat daftar iklan");
    }
    
    // Parse date strings to DateTime objects
    let start_date_from = parse_rfc3339_date(start_date_from);
    let start_date_to = parse_rfc3339_date(start_date_to);
    let end_date_from = parse_rfc3339_date(end_date_from);
    let end_date_to = parse_rfc3339_date(end_date_to);
    
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
        Ok(result) => ApiResponse::success("Daftar iklan berhasil diambil", result),
        Err(e) => ApiResponse::server_error(&format!("Gagal mengambil daftar iklan: {}", e))
    }
}

/// Get advertisement by ID
#[get("/advertisements/<id>")]
pub async fn get_advertisement_by_id(
    id: String,
    auth: AuthenticatedUser,
    service: &State<Arc<dyn AdvertisementService>>,
) -> ApiResponse<AdvertisementDetailResponse> {
    match service.get_advertisement_by_id(&id).await {
        Ok(advertisement) => ApiResponse::success("Detail iklan berhasil diambil", advertisement),
        Err(e) => {
            if e.to_string().contains("not found") {
                ApiResponse::not_found(&format!("Iklan dengan ID {} tidak ditemukan", id))
            } else {
                ApiResponse::server_error(&format!("Gagal mengambil detail iklan: {}", e))
            }
        }
    }
}

/// Helper function to parse RFC3339 date strings
fn parse_rfc3339_date(date_str: Option<String>) -> Option<chrono::DateTime<chrono::Utc>> {
    date_str.and_then(|d| {
        chrono::DateTime::parse_from_rfc3339(&d)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    })
}

/// Register all advertisement routes
pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        get_all_advertisements,
        get_advertisement_by_id,
    ]
}