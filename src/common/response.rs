use rocket::serde::json::Json;
use serde::Serialize;

use crate::error::ValidationError;

/// Standard API response wrapper
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: u16,
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ValidationError>>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response with data
    pub fn success(message: &str, data: T) -> Json<Self> {
        Json(Self {
            code: 200,
            success: true,
            message: message.to_string(),
            data: Some(data),
            errors: None,
        })
    }
    
    /// Create a response for created resources
    pub fn created(message: &str, data: T) -> Json<Self> {
        Json(Self {
            code: 201,
            success: true,
            message: message.to_string(),
            data: Some(data),
            errors: None,
        })
    }
    
    /// Create an error response
    pub fn error(code: u16, message: &str) -> Json<Self> {
        Json(Self {
            code,
            success: false,
            message: message.to_string(),
            data: None,
            errors: None,
        })
    }
    
    /// Create a validation error response
    pub fn validation_error(message: &str, errors: Vec<ValidationError>) -> Json<Self> {
        Json(Self {
            code: 400,
            success: false,
            message: message.to_string(),
            data: None,
            errors: Some(errors),
        })
    }
}

/// Simple response without data
#[derive(Serialize)]
pub struct SimpleResponse {
    pub code: u16,
    pub success: bool,
    pub message: String,
}

impl SimpleResponse {
    /// Create a successful simple response
    pub fn success(message: &str) -> Json<Self> {
        Json(Self {
            code: 200,
            success: true,
            message: message.to_string(),
        })
    }
}