use rocket::{State, get, post, form::Form, fs::TempFile, data::ToByteUnit};
use rocket::serde::json::Json;
use rocket::response::status::Created;
use std::sync::Arc;
use std::io::Read;
use url::Url;
use image::{GenericImageView, ImageFormat};

use crate::common::api_response::ApiResponse;
use crate::dto::advertisement::advertisement::{
    AdvertisementQueryParams, AdvertisementListResponse, AdvertisementDetailResponse,
    CreateAdvertisementRequest, CreateAdvertisementResponse, ValidationError
};
use crate::service::advertisement::ad_service::AdvertisementService;
use crate::middleware::auth::AuthenticatedUser;

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
    
    // Validate inputs
    let mut validation_errors = Vec::new();
    
    // Title validation
    if form.title.is_empty() {
        validation_errors.push(ValidationError {
            field: "title".to_string(),
            message: "Judul iklan wajib diisi".to_string(),
        });
    } else if form.title.len() > 100 {
        validation_errors.push(ValidationError {
            field: "title".to_string(),
            message: "Judul iklan maksimal 100 karakter".to_string(),
        });
    }
    
    // Description validation
    if let Some(desc) = &form.description {
        if desc.len() > 500 {
            validation_errors.push(ValidationError {
                field: "description".to_string(),
                message: "Deskripsi iklan maksimal 500 karakter".to_string(),
            });
        }
    }
    
    // Image validation
    if form.image.len() == 0 {
        validation_errors.push(ValidationError {
            field: "image".to_string(),
            message: "Gambar iklan wajib diunggah".to_string(),
        });
    } else {
        let content_type = form.image.content_type().unwrap_or_else(|| "".into()).to_string();
        
        // Check file size (max 2MB)
        let file_size = form.image.len();
        if file_size > 2 * 1024 * 1024 {
            validation_errors.push(ValidationError {
                field: "image".to_string(),
                message: "Ukuran gambar tidak boleh melebihi 2MB".to_string(),
            });
        }
        
        // Validate image format
        if !is_valid_image_format(&content_type) {
            validation_errors.push(ValidationError {
                field: "image".to_string(),
                message: "Format gambar harus JPG, PNG, atau GIF".to_string(),
            });
        } else {
            // Check image dimensions
            let mut image_data = Vec::new();
            if let Err(_) = form.image.open().read_to_end(&mut image_data) {
                validation_errors.push(ValidationError {
                    field: "image".to_string(),
                    message: "Gagal membaca data gambar".to_string(),
                });
            } else {
                match validate_image_dimensions(&image_data) {
                    Err(msg) => {
                        validation_errors.push(ValidationError {
                            field: "image".to_string(),
                            message: msg,
                        });
                    },
                    _ => {}
                }
            }
        }
    }
    
    // Date validation
    let now = chrono::Utc::now();
    let start_date = match parse_rfc3339_date(Some(form.start_date.clone())) {
        Some(date) => {
            if date < now {
                validation_errors.push(ValidationError {
                    field: "start_date".to_string(),
                    message: "Tanggal mulai harus di masa depan".to_string(),
                });
            }
            date
        },
        None => {
            validation_errors.push(ValidationError {
                field: "start_date".to_string(),
                message: "Format tanggal mulai tidak valid".to_string(),
            });
            now // Default value, won't be used if validation fails
        }
    };
    
    let end_date = match parse_rfc3339_date(Some(form.end_date.clone())) {
        Some(date) => {
            if date <= start_date {
                validation_errors.push(ValidationError {
                    field: "end_date".to_string(),
                    message: "Tanggal selesai harus setelah tanggal mulai".to_string(),
                });
            }
            date
        },
        None => {
            validation_errors.push(ValidationError {
                field: "end_date".to_string(),
                message: "Format tanggal selesai tidak valid".to_string(),
            });
            now // Default value, won't be used if validation fails
        }
    };
    
    // Click URL validation
    if form.click_url.is_empty() {
        validation_errors.push(ValidationError {
            field: "click_url".to_string(),
            message: "URL klik wajib diisi".to_string(),
        });
    } else if !is_valid_url(&form.click_url) {
        validation_errors.push(ValidationError {
            field: "click_url".to_string(),
            message: "URL klik harus berupa URL yang valid".to_string(),
        });
    }
    
    // Position validation
    if form.position.is_empty() {
        validation_errors.push(ValidationError {
            field: "position".to_string(),
            message: "Posisi iklan wajib diisi".to_string(),
        });
    } else if !is_valid_position(&form.position) {
        validation_errors.push(ValidationError {
            field: "position".to_string(),
            message: "Posisi iklan harus salah satu dari: homepage_top, homepage_middle, homepage_bottom".to_string(),
        });
    }
    
    // Return validation errors if any
    if !validation_errors.is_empty() {
        return Err(ApiResponse::validation_error("Validasi gagal", validation_errors));
    }
    
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

/// Helper function to parse RFC3339 date strings
fn parse_rfc3339_date(date_str: Option<String>) -> Option<chrono::DateTime<chrono::Utc>> {
    date_str.and_then(|d| {
        chrono::DateTime::parse_from_rfc3339(&d)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    })
}

/// Check if a string is a valid URL
fn is_valid_url(url: &str) -> bool {
    Url::parse(url).is_ok()
}

/// Check if position is valid
fn is_valid_position(position: &str) -> bool {
    matches!(position, "homepage_top" | "homepage_middle" | "homepage_bottom")
}

/// Check if the content type is a valid image format
fn is_valid_image_format(content_type: &str) -> bool {
    matches!(content_type, "image/jpeg" | "image/png" | "image/gif")
}

/// Validate image dimensions
fn validate_image_dimensions(image_data: &[u8]) -> Result<(), String> {
    let img = image::load_from_memory(image_data)
        .map_err(|_| "Gagal memproses gambar".to_string())?;
    
    let (width, height) = img.dimensions();
    
    // Check minimum dimensions (800x400)
    if width < 800 || height < 400 {
        return Err("Resolusi gambar minimal 800x400 piksel".to_string());
    }
    
    // Check maximum dimensions (2000x1000)
    if width > 2000 || height > 1000 {
        return Err("Resolusi gambar maksimal 2000x1000 piksel".to_string());
    }
    
    Ok(())
}

/// Register all advertisement routes
pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![
        get_all_advertisements,
        get_advertisement_by_id,
        create_advertisement
    ]
}