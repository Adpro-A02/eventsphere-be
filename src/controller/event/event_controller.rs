use actix_web::{web, HttpResponse, Responder, HttpRequest};
use actix_web::http::StatusCode;
use tracing::event;
use uuid::Uuid;
use std::sync::Arc;

use crate::model::event::event::{CreateEventDto, UpdateEventDto};
use crate::service::event::event_service::{EventService, ServiceError};
use crate::repository::event::EventRepository;

// Helper function to map service errors to Actix responses
fn map_error_to_response(error: ServiceError) -> HttpResponse {
    match error {
        ServiceError::NotFound(msg) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "status": "error",
                "message": msg
            }))
        }
        ServiceError::InvalidInput(msg) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "status": "error",
                "message": msg
            }))
        }
        ServiceError::RepositoryError(msg) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", msg)
            }))
        }
        ServiceError::InternalError(msg) => {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Internal server error: {}", msg)
            }))
        }
    }
}


pub trait EventServiceTrait {
    fn create_event(&self, dto: CreateEventDto) -> Result<serde_json::Value, ServiceError>;
    fn list_events(&self) -> Result<serde_json::Value, ServiceError>;
    fn get_event(&self, event_id: &str) -> Result<serde_json::Value, ServiceError>;
    fn update_event(&self, event_id: &str, dto: UpdateEventDto) -> Result<serde_json::Value, ServiceError>;
    fn delete_event(&self, event_id: &str) -> Result<(), ServiceError>;
    fn publish_event(&self, event_id: &str) -> Result<serde_json::Value, ServiceError>;
    fn cancel_event(&self, event_id: &str) -> Result<serde_json::Value, ServiceError>;
    fn complete_event(&self, event_id: &str) -> Result<serde_json::Value, ServiceError>;
}

// Implement the trait for any EventService with any EventRepository
impl<R: EventRepository> EventServiceTrait for EventService<R> {
    fn create_event(&self, dto: CreateEventDto) -> Result<serde_json::Value, ServiceError> {
        self.create_event(dto).map(|event| serde_json::json!(event))
    }

    fn list_events(&self) -> Result<serde_json::Value, ServiceError> {
        self.list_events().map(|events| serde_json::json!(events))
    }

    fn get_event(&self, event_id: &str) -> Result<serde_json::Value, ServiceError> {
        self.get_event(event_id).map(|event| serde_json::json!(event))
    }

    fn update_event(&self, event_id: &str, dto: UpdateEventDto) -> Result<serde_json::Value, ServiceError> {
        self.update_event(event_id, dto).map(|event| serde_json::json!(event))
    }

    fn delete_event(&self, event_id: &str) -> Result<(), ServiceError> {
        self.delete_event(event_id)
    }

    fn publish_event(&self, event_id: &str) -> Result<serde_json::Value, ServiceError> {
        self.publish_event(event_id).map(|event| serde_json::json!(event))
    }

    fn cancel_event(&self, event_id: &str) -> Result<serde_json::Value, ServiceError> {
        self.cancel_event(event_id).map(|event| serde_json::json!(event))
    }

    fn complete_event(&self, event_id: &str) -> Result<serde_json::Value, ServiceError> {
        self.complete_event(event_id).map(|event| serde_json::json!(event))
    }
}

// Create a type alias for the service with type erasure
pub type DynEventService = Arc<dyn EventServiceTrait + Send + Sync>;

// Create a new event
async fn create_event(
    service: web::Data<DynEventService>,
    dto: web::Json<CreateEventDto>,
) -> impl Responder {
    match service.create_event(dto.into_inner()) {
        Ok(event) => {
        
            let id = event.get("id").and_then(|id| id.as_str()).unwrap_or("unknown");
            let location = format!("/api/events/{}", id);
            
            HttpResponse::Created()
                .insert_header(("Location", location))
                .json(event)
        },
        Err(e) => map_error_to_response(e),
    }
}

// List all events
async fn list_events(
    service: web::Data<DynEventService>,
) -> impl Responder {
    match service.list_events() {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => map_error_to_response(e),
    }
}

// Get a specific event
async fn get_event(
    service: web::Data<DynEventService>,
    path: web::Path<String>,
) -> impl Responder {
    let event_id = path.into_inner();
    match service.get_event(&event_id) {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => map_error_to_response(e),
    }
}

// Update an event
async fn update_event(
    service: web::Data<DynEventService>,
    path: web::Path<String>,
    dto: web::Json<UpdateEventDto>,
) -> impl Responder {
    let event_id = path.into_inner();
    match service.update_event(&event_id, dto.into_inner()) {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => map_error_to_response(e),
    }
}

// Delete an event
async fn delete_event(
    service: web::Data<DynEventService>,
    path: web::Path<String>,
    
) -> impl Responder {
    let event_id = path.into_inner();
    match service.delete_event(&event_id) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": format!("Event dengan ID {} berhasil dihapus", event_id)
        })),
        Err(e) => map_error_to_response(e),
    }
}

// Publish an event
async fn publish_event(
    service: web::Data<DynEventService>,
    path: web::Path<String>,
) -> impl Responder {
    let event_id = path.into_inner();
    match service.publish_event(&event_id) {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => map_error_to_response(e),
    }
}

// Cancel an event
async fn cancel_event(
    service: web::Data<DynEventService>,
    path: web::Path<String>,
) -> impl Responder {
    let event_id = path.into_inner();
    match service.cancel_event(&event_id) {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => map_error_to_response(e),
    }
}

// Complete an event
async fn complete_event(
    service: web::Data<DynEventService>,
    path: web::Path<String>,
) -> impl Responder {
    let event_id = path.into_inner();
    match service.complete_event(&event_id) {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => map_error_to_response(e),
    }
}

// Function to configure and register all routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(
                web::resource("/events")
                    .route(web::post().to(create_event))
                    .route(web::get().to(list_events))
            )
            .service(
                web::resource("/events/{event_id}")
                    .route(web::get().to(get_event))
                    .route(web::put().to(update_event))
                    .route(web::delete().to(delete_event))
            )
            .service(web::resource("/events/{event_id}/publish").route(web::post().to(publish_event)))
            .service(web::resource("/events/{event_id}/cancel").route(web::post().to(cancel_event)))
            .service(web::resource("/events/{event_id}/complete").route(web::post().to(complete_event)))
    );
}