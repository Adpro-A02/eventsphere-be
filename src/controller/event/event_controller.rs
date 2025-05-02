use rocket::{State, serde::json::Json, response::status, http::Status, post, get, put, delete, routes};
use rocket::response::status::Custom;
use rocket::serde::json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::model::event::event::{CreateEventDto, UpdateEventDto};
use crate::service::event::event_service::{EventService, ServiceError};
use crate::repository::event::EventRepository;

// Helper function to map service errors to Rocket responses
fn map_error_to_response(error: ServiceError) -> Custom<Value> {
    match error {
        ServiceError::NotFound(msg) => {
            Custom(Status::NotFound, json!({
                "status": "error",
                "message": msg
            }))
        }
        ServiceError::InvalidInput(msg) => {
            Custom(Status::BadRequest, json!({
                "status": "error",
                "message": msg
            }))
        }
        ServiceError::RepositoryError(msg) => {
            Custom(Status::InternalServerError, json!({
                "status": "error",
                "message": format!("Database error: {}", msg)
            }))
        }
        ServiceError::InternalError(msg) => {
            Custom(Status::InternalServerError, json!({
                "status": "error",
                "message": format!("Internal server error: {}", msg)
            }))
        }
    }
}

// Create a type alias for the service with type erasure
pub type DynEventService = Arc<dyn EventServiceTrait + Send + Sync>;

// Define a trait that abstracts over the generic EventService
pub trait EventServiceTrait {
    fn create_event(&self, dto: CreateEventDto) -> Result<Value, ServiceError>;
    fn list_events(&self) -> Result<Value, ServiceError>;
    fn get_event(&self, event_id: &str) -> Result<Value, ServiceError>;
    fn update_event(&self, event_id: &str, dto: UpdateEventDto) -> Result<Value, ServiceError>;
    fn delete_event(&self, event_id: &str) -> Result<(), ServiceError>;
    fn publish_event(&self, event_id: &str) -> Result<Value, ServiceError>;
    fn cancel_event(&self, event_id: &str) -> Result<Value, ServiceError>;
    fn complete_event(&self, event_id: &str) -> Result<Value, ServiceError>;
}

// Implement the trait for any EventService with any EventRepository
impl<R: EventRepository> EventServiceTrait for EventService<R> {
    fn create_event(&self, dto: CreateEventDto) -> Result<Value, ServiceError> {
        self.create_event(dto).map(|event| json!(event))
    }

    fn list_events(&self) -> Result<Value, ServiceError> {
        self.list_events().map(|events| json!(events))
    }

    fn get_event(&self, event_id: &str) -> Result<Value, ServiceError> {
        self.get_event(event_id).map(|event| json!(event))
    }

    fn update_event(&self, event_id: &str, dto: UpdateEventDto) -> Result<Value, ServiceError> {
        self.update_event(event_id, dto).map(|event| json!(event))
    }

    fn delete_event(&self, event_id: &str) -> Result<(), ServiceError> {
        self.delete_event(event_id)
    }

    fn publish_event(&self, event_id: &str) -> Result<Value, ServiceError> {
        self.publish_event(event_id).map(|event| json!(event))
    }

    fn cancel_event(&self, event_id: &str) -> Result<Value, ServiceError> {
        self.cancel_event(event_id).map(|event| json!(event))
    }

    fn complete_event(&self, event_id: &str) -> Result<Value, ServiceError> {
        self.complete_event(event_id).map(|event| json!(event))
    }
}

// Create a new event
#[post("/events", format = "json", data = "<dto>")]
pub async fn create_event(
    service: &State<DynEventService>,
    dto: Json<CreateEventDto>,
) -> Result<status::Created<Json<Value>>, Custom<Value>> {
    match service.create_event(dto.into_inner()) {
        Ok(event) => {
            // Extract the ID from the JSON value
            let id = event.get("id").and_then(|id| id.as_str()).unwrap_or("unknown");
            let location = format!("/api/events/{}", id);
            Ok(status::Created::new(location).body(Json(event)))
        },
        Err(e) => Err(map_error_to_response(e)),
    }
}

// List all events
#[get("/events")]
pub async fn list_events(
    service: &State<DynEventService>,
) -> Result<Json<Value>, Custom<Value>> {
    match service.list_events() {
        Ok(events) => Ok(Json(events)),
        Err(e) => Err(map_error_to_response(e)),
    }
}

// Get a specific event
#[get("/events/<event_id>")]
pub async fn get_event(
    service: &State<DynEventService>,
    event_id: &str,
) -> Result<Json<Value>, Custom<Value>> {
    match service.get_event(event_id) {
        Ok(event) => Ok(Json(event)),
        Err(e) => Err(map_error_to_response(e)),
    }
}

// Update an event
#[put("/events/<event_id>", format = "json", data = "<dto>")]
pub async fn update_event(
    service: &State<DynEventService>,
    event_id: &str,
    dto: Json<UpdateEventDto>,
) -> Result<Json<Value>, Custom<Value>> {
    match service.update_event(event_id, dto.into_inner()) {
        Ok(event) => Ok(Json(event)),
        Err(e) => Err(map_error_to_response(e)),
    }
}

// Delete an event
#[delete("/events/<event_id>")]
pub async fn delete_event(
    service: &State<DynEventService>,
    event_id: &str,
) -> Result<Status, Custom<Value>> {
    match service.delete_event(event_id) {
        Ok(_) => Ok(Status::NoContent),
        Err(e) => Err(map_error_to_response(e)),
    }
}

// Publish an event
#[post("/events/<event_id>/publish")]
pub async fn publish_event(
    service: &State<DynEventService>,
    event_id: &str,
) -> Result<Json<Value>, Custom<Value>> {
    match service.publish_event(event_id) {
        Ok(event) => Ok(Json(event)),
        Err(e) => Err(map_error_to_response(e)),
    }
}

// Cancel an event
#[post("/events/<event_id>/cancel")]
pub async fn cancel_event(
    service: &State<DynEventService>,
    event_id: &str,
) -> Result<Json<Value>, Custom<Value>> {
    match service.cancel_event(event_id) {
        Ok(event) => Ok(Json(event)),
        Err(e) => Err(map_error_to_response(e)),
    }
}

// Complete an event
#[post("/events/<event_id>/complete")]
pub async fn complete_event(
    service: &State<DynEventService>,
    event_id: &str,
) -> Result<Json<Value>, Custom<Value>> {
    match service.complete_event(event_id) {
        Ok(event) => Ok(Json(event)),
        Err(e) => Err(map_error_to_response(e)),
    }
}

// Function to register all routes
pub fn routes() -> Vec<rocket::Route> {
    routes![
        create_event,
        list_events,
        get_event,
        update_event,
        delete_event,
        publish_event,
        cancel_event,
        complete_event
    ]
}