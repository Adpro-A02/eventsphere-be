use std::io::Read;
use url::Url;
use image::GenericImageView;
use chrono::Utc;

use crate::dto::advertisement::advertisement::ValidationError;

/// Check if a string is a valid URL
pub fn is_valid_url(url: &str) -> bool {
    Url::parse(url).is_ok()
}

/// Check if position is valid
pub fn is_valid_position(position: &str) -> bool {
    matches!(position, "homepage_top" | "homepage_middle" | "homepage_bottom")
}

/// Check if the content type is a valid image format
pub fn is_valid_image_format(content_type: &str) -> bool {
    matches!(content_type, "image/jpeg" | "image/png" | "image/gif")
}

/// Validate image dimensions
pub fn validate_image_dimensions(image_data: &[u8]) -> Result<(), String> {
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

/// Helper function to parse RFC3339 date strings
pub fn parse_rfc3339_date(date_str: Option<String>) -> Option<chrono::DateTime<chrono::Utc>> {
    date_str.and_then(|d| {
        chrono::DateTime::parse_from_rfc3339(&d)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    })
}

/// Validate advertisement form data and return any validation errors
pub fn validate_advertisement_form<'r>(
    form: &rocket::form::Form<super::ad_controller::AdvertisementForm<'r>>
) -> Vec<ValidationError> {
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
    let now = Utc::now();
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
    
    validation_errors
}