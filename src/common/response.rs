use rocket::serde::json::Json;
use serde::Serialize;

/// Validation error structure for form validation errors
#[derive(Debug, Clone, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

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
    pub fn created(message: &str, data: T) -> Self {
        Self {
            code: 201,
            success: true,
            message: message.to_string(),
            data: Some(data),
            errors: None,
        }
    }
    
    /// Create a not found error response
    pub fn not_found(message: &str) -> Json<ApiResponse<T>> {
        Json(Self {
            code: 404,
            success: false,
            message: message.to_string(),
            data: None,
            errors: None,
        })
    }

    pub fn not_found_with_data(message: &str, data: T) -> ApiResponse<T> {
        Self {
            code: 404,
            success: false,
            message: message.to_string(),
            data: Some(data),
            errors: None,
        }
    }
    
    /// Create a server error response
    pub fn server_error(message: &str) -> Json<ApiResponse<T>> {
        Json(Self {
            code: 500,
            success: false,
            message: message.to_string(),
            data: None,
            errors: None,
        })
    }
    
    /// Create a forbidden error response
    pub fn forbidden(message: &str) -> Json<ApiResponse<T>> {
        Json(Self {
            code: 403,
            success: false,
            message: message.to_string(),
            data: None,
            errors: None,
        })
    }
    
    /// Create a validation error response
    pub fn validation_error(message: &str, errors: Vec<ValidationError>) -> ApiResponse<Vec<ValidationError>> {
        ApiResponse {
            code: 400,
            success: false,
            message: message.to_string(),
            data: None,
            errors: Some(errors),
        }
    }
    
    /// Create a server error response with data
    pub fn server_error_with_data(message: &str, data: T) -> ApiResponse<T> {
        Self {
            code: 500,
            success: false,
            message: message.to_string(),
            data: Some(data),
            errors: None,
        }
    }
    
    /// Create a forbidden error response with data
    pub fn forbidden_with_data(message: &str, data: T) -> ApiResponse<T> {
        Self {
            code: 403,
            success: false,
            message: message.to_string(),
            data: Some(data),
            errors: None,
        }
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