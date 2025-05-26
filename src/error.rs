use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::{json, Value};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Main application error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

/// Validation error details
#[derive(Serialize, Deserialize, Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl AppError {
    /// Maps the error to an HTTP status code
    pub fn to_status(&self) -> Status {
        match self {
            AppError::NotFound(_) => Status::NotFound,
            AppError::Validation(_) => Status::BadRequest,
            AppError::Authentication(_) => Status::Unauthorized,
            AppError::Authorization(_) => Status::Forbidden,
            AppError::Database(_) | AppError::Storage(_) | 
            AppError::Infrastructure(_) | AppError::Internal(_) => Status::InternalServerError,
        }
    }

    pub fn to_status_http(&self) -> warp::http::StatusCode {
        match self {
            AppError::NotFound(_) => warp::http::StatusCode::NOT_FOUND,
            AppError::Validation(_) => warp::http::StatusCode::BAD_REQUEST,
            AppError::Authentication(_) => warp::http::StatusCode::UNAUTHORIZED,
            AppError::Authorization(_) => warp::http::StatusCode::FORBIDDEN,
            AppError::Database(_) | AppError::Storage(_) |
            AppError::Infrastructure(_) | AppError::Internal(_) => warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    /// Converts the error to a JSON response
    pub fn to_json(&self, validation_errors: Option<Vec<ValidationError>>) -> Value {
        let code = self.to_status().code;
        let message = self.to_string();
        
        match validation_errors {
            Some(errors) => json!({
                "code": code,
                "success": false,
                "message": "Validasi gagal",
                "errors": errors
            }),
            None => json!({
                "code": code,
                "success": false,
                "message": message
            }),
        }
    }
}

/// Implement Rocket's Responder for AppError
impl<'r> Responder<'r, 'static> for AppError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let status = self.to_status();
        let json = self.to_json(None);
        
        Response::build()
            .status(status)
            .header(rocket::http::ContentType::JSON)
            .sized_body(json.to_string().len(), std::io::Cursor::new(json.to_string()))
            .ok()
    }
}

/// Module for Rocket error catchers
pub mod handlers {
    use rocket::{catch, Request};
    use rocket::serde::json::{json, Value};
    
    #[catch(404)]
    pub fn not_found(_: &Request) -> Value {
        json!({
            "code": 404,
            "success": false,
            "message": "Resource tidak ditemukan"
        })
    }
    
    #[catch(422)]
    pub fn unprocessable_entity(req: &Request) -> Value {
        json!({
            "code": 422,
            "success": false,
            "message": format!("Parameter tidak valid: {}", req.uri())
        })
    }
    
    #[catch(500)]
    pub fn server_error(_: &Request) -> Value {
        json!({
            "code": 500,
            "success": false,
            "message": "Terjadi kesalahan pada server"
        })
    }
    
    #[catch(401)]
    pub fn unauthorized(_: &Request) -> Value {
        json!({
            "code": 401,
            "success": false,
            "message": "Token autentikasi diperlukan"
        })
    }
    
    #[catch(403)]
    pub fn forbidden(_: &Request) -> Value {
        json!({
            "code": 403,
            "success": false,
            "message": "Anda tidak memiliki akses untuk melakukan operasi ini"
        })
    }
}