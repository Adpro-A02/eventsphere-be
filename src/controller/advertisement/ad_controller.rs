use rocket::{State, get, post, form::Form, fs::TempFile};
use rocket::serde::json::Json;
use rocket::put;
use rocket::response::status::Created;
use std::sync::Arc;
use std::io::Read;

use crate::common::api_response::ApiResponse;
use crate::dto::advertisement::advertisement::{
    AdvertisementQueryParams, AdvertisementListResponse, AdvertisementDetailResponse,
    CreateAdvertisementRequest, CreateAdvertisementResponse, ValidationError,
    UpdateAdvertisementRequest, UpdateAdvertisementResponse,
};
use crate::service::advertisement::ad_service::AdvertisementService;
use crate::middleware::auth::AuthenticatedUser;
use super::validation::{parse_rfc3339_date, validate_advertisement_form, validate_advertisement_update_form};

/// Form data structure for creating an advertisement
#[derive(FromForm)]
pub struct AdvertisementForm<'r> {
    pub title: String,
    pub description: Option<String>,
    pub image: TempFile<'r>,
    pub start_date: String,
    pub end_date: String,
    pub click_url: String,
    pub position: String,
}

#[derive(FromForm)]
pub struct UpdateAdvertisementForm<'r> {
    pub title: String,
    pub description: Option<String>,
    pub image: Option<TempFile<'r>>,
    pub start_date: String,
    pub end_date: String,
    pub click_url: String,
    pub position: String,
}

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
    // Check if the user is admin
    if !auth.is_admin() {
        return ApiResponse::forbidden("Anda tidak memiliki akses untuk melihat detail iklan");
    }

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

/// Add new advertisement
#[post("/advertisements", data = "<form>")]
pub async fn create_advertisement(
    form: Form<AdvertisementForm<'_>>,
    auth: AuthenticatedUser,
    service: &State<Arc<dyn AdvertisementService>>,
) -> Result<Created<Json<ApiResponse<CreateAdvertisementResponse>>>, ApiResponse<Vec<ValidationError>>> {
    // Check if the user is admin
    if !auth.is_admin() {
        return Err(ApiResponse::forbidden_with_data("Anda tidak memiliki akses untuk menambahkan iklan", vec![]));
    }
    
    // Validate form data
    let validation_errors = validate_advertisement_form(&form);
    
    // Return validation errors if any
    if !validation_errors.is_empty() {
        return Err(ApiResponse::validation_error("Validasi gagal", validation_errors));
    }
    
    // Parse dates
    let start_date = parse_rfc3339_date(Some(form.start_date.clone())).unwrap();
    let end_date = parse_rfc3339_date(Some(form.end_date.clone())).unwrap();
    
    // Read image data
    let mut image_data = Vec::new();
    if let Err(e) = form.image.open().read_to_end(&mut image_data) {
        return Err(ApiResponse::server_error_with_data(
            &format!("Gagal membaca data gambar: {}", e),
            vec![]
        ));
    }
    
    // Create request DTO
    let request = CreateAdvertisementRequest {
        title: form.title.clone(),
        description: form.description.clone(),
        start_date,
        end_date,
        click_url: form.click_url.clone(),
        position: form.position.clone(),
    };
    
    // Call service
    match service.create_advertisement(request, image_data).await {
        Ok(advertisement) => {
            let response = ApiResponse::created("Iklan berhasil ditambahkan", advertisement);
            Ok(Created::new("/").body(Json(response)))
        },
        Err(e) => {
            Err(ApiResponse::server_error_with_data(
                &format!("Gagal menambahkan iklan: {}", e),
                vec![]
            ))
        }
    }
}

/// Update an existing advertisement
#[put("/advertisements/<id>", data = "<form>")]
pub async fn update_advertisement(
    id: String,
    form: Form<UpdateAdvertisementForm<'_>>,
    auth: AuthenticatedUser,
    service: &State<Arc<dyn AdvertisementService>>,
) -> Result<Json<ApiResponse<UpdateAdvertisementResponse>>, ApiResponse<Vec<ValidationError>>> {
    // Check if the user is admin
    if !auth.is_admin() {
        return Err(ApiResponse::forbidden_with_data("Anda tidak memiliki akses untuk mengubah iklan", vec![]));
    }
    
    // Validate form data
    let validation_errors = validate_advertisement_update_form(&form);
    
    // Return validation errors if any
    if !validation_errors.is_empty() {
        return Err(ApiResponse::validation_error("Validasi gagal", validation_errors));
    }
    
    // Parse dates
    let start_date = parse_rfc3339_date(Some(form.start_date.clone())).unwrap();
    let end_date = parse_rfc3339_date(Some(form.end_date.clone())).unwrap();
    
    // Read image data if provided
    let image_data = if let Some(ref image) = form.image {
        if image.len() > 0 {
            let mut data = Vec::new();
            if let Err(e) = image.open().read_to_end(&mut data) {
                return Err(ApiResponse::server_error_with_data(
                    &format!("Gagal membaca data gambar: {}", e),
                    vec![]
                ));
            }
            Some(data)
        } else {
            None
        }
    } else {
        None
    };
    
    // Create request DTO
    let request = UpdateAdvertisementRequest {
        title: form.title.clone(),
        description: form.description.clone(),
        start_date,
        end_date,
        click_url: form.click_url.clone(),
        position: form.position.clone(),
    };
    
    // Call service
    match service.update_advertisement(&id, request, image_data).await {
        Ok(advertisement) => {
            let response = ApiResponse::success("Iklan berhasil diperbarui", advertisement);
            Ok(Json(response))
        },
        Err(e) => {
            if e.to_string().contains("not found") {
                Err(ApiResponse::not_found_with_data(
                    &format!("Iklan dengan ID {} tidak ditemukan", id),
                    vec![]
                ))
            } else {
                Err(ApiResponse::server_error_with_data(
                    &format!("Gagal memperbarui iklan: {}", e),
                    vec![]
                ))
            }
        }
    }
}

/// Register all advertisement routes
pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        get_all_advertisements,
        get_advertisement_by_id,
        create_advertisement,
        update_advertisement,
    ]
}