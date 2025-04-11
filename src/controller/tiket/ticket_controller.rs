use crate::model::tiket::ticket::Ticket;
use crate::service::tiket::ticket_service::TicketService;
use rocket::http::Status;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::State;
use uuid::Uuid;
use serde_json::{json, Value};

// Request and Response structures

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateTicketRequest {
    pub event_id: String,
    pub ticket_type: String,
    pub price: f64,
    pub quota: u32,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct UpdateTicketRequest {
    pub ticket_type: Option<String>,
    pub price: Option<f64>,
    pub quota: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AllocateTicketsRequest {
    pub quantity: u32,
}

// Utility function to format successful responses
fn json_success<T: Serialize>(data: T) -> Value {
    json!({
        "success": true,
        "data": data
    })
}

// Utility function to format error responses
fn json_error(message: &str) -> Value {
    json!({
        "success": false,
        "error": message
    })
}

// Create a new ticket
#[post("/events/tickets", format = "json", data = "<request>")]
pub async fn create_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    request: Json<CreateTicketRequest>
) -> (Status, Value) {
    // Parse event_id from string to Uuid
    let event_id = match Uuid::parse_str(&request.event_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid event_id format")),
    };

    // Validate input
    if request.ticket_type.is_empty() {
        return (Status::BadRequest, json_error("Ticket type cannot be empty"));
    }

    if request.price < 0.0 {
        return (Status::BadRequest, json_error("Price cannot be negative"));
    }

    // Call service to create ticket
    match service.create_ticket(
        event_id, 
        request.ticket_type.clone(), 
        request.price, 
        request.quota
    ) {
        Ok(ticket) => (Status::Created, json_success(ticket)),
        Err(error) => (Status::InternalServerError, json_error(&error)),
    }
}

// Get ticket by ID
#[get("/tickets/<ticket_id>")]
pub async fn get_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    ticket_id: &str
) -> (Status, Value) {
    // Parse ticket_id from string to Uuid
    let uuid = match Uuid::parse_str(ticket_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid UUID format")),
    };

    // Call service to get ticket
    match service.get_ticket(&uuid) {
        Ok(Some(ticket)) => (Status::Ok, json_success(ticket)),
        Ok(None) => (Status::NotFound, json_error("Ticket not found")),
        Err(error) => (Status::InternalServerError, json_error(&error)),
    }
}

// Get tickets by event ID
#[get("/events/<event_id>/tickets")]
pub async fn get_tickets_by_event(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    event_id: &str
) -> (Status, Value) {
    // Parse event_id from string to Uuid
    let uuid = match Uuid::parse_str(event_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid UUID format")),
    };

    // Call service to get tickets for event
    match service.get_tickets_by_event(&uuid) {
        Ok(tickets) => (Status::Ok, json_success(tickets)),
        Err(error) => (Status::InternalServerError, json_error(&error)),
    }
}

// Update ticket
#[put("/tickets/<ticket_id>", format = "json", data = "<request>")]
pub async fn update_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    ticket_id: &str,
    request: Json<UpdateTicketRequest>
) -> (Status, Value) {
    // Parse ticket_id from string to Uuid
    let uuid = match Uuid::parse_str(ticket_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid UUID format")),
    };

    // Call service to update ticket
    match service.update_ticket(
        &uuid,
        request.ticket_type.clone(),
        request.price,
        request.quota
    ) {
        Ok(updated) => (Status::Ok, json_success(updated)),
        Err(error) if error == "Ticket not found" => (Status::NotFound, json_error(&error)),
        Err(error) => (Status::InternalServerError, json_error(&error)),
    }
}

// Delete ticket
#[delete("/tickets/<ticket_id>")]
pub async fn delete_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    ticket_id: &str
) -> (Status, Value) {
    // Parse ticket_id from string to Uuid
    let uuid = match Uuid::parse_str(ticket_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid UUID format")),
    };

    // Call service to delete ticket
    match service.delete_ticket(&uuid) {
        Ok(_) => (
            Status::Ok, 
            json!({
                "success": true,
                "message": "Ticket deleted successfully"
            })
        ),
        Err(error) if error == "Ticket not found" => (Status::NotFound, json_error(&error)),
        Err(error) => (Status::InternalServerError, json_error(&error)),
    }
}

// Check ticket availability
#[get("/tickets/<ticket_id>/availability?<quantity>")]
pub async fn check_availability(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    ticket_id: &str,
    quantity: u32
) -> (Status, Value) {
    // Parse ticket_id from string to Uuid
    let uuid = match Uuid::parse_str(ticket_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid UUID format")),
    };

    // Call service to check availability
    match service.check_availability(&uuid, quantity) {
        Ok(available) => (
            Status::Ok, 
            json!({
                "success": true,
                "available": available
            })
        ),
        Err(error) if error == "Ticket not found" => (Status::NotFound, json_error(&error)),
        Err(error) => (Status::InternalServerError, json_error(&error)),
    }
}

// Allocate tickets
#[post("/tickets/<ticket_id>/allocate", format = "json", data = "<request>")]
pub async fn allocate_tickets(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    ticket_id: &str,
    request: Json<AllocateTicketsRequest>
) -> (Status, Value) {
    // Parse ticket_id from string to Uuid
    let uuid = match Uuid::parse_str(ticket_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid UUID format")),
    };

    // Call service to allocate tickets
    match service.allocate_tickets(&uuid, request.quantity) {
        Ok(true) => (
            Status::Ok, 
            json!({
                "success": true,
                "allocated": true
            })
        ),
        Ok(false) => (
            Status::BadRequest, 
            json_error("Insufficient tickets available")
        ),
        Err(error) if error == "Ticket not found" => (Status::NotFound, json_error(&error)),
        Err(error) => (Status::InternalServerError, json_error(&error)),
    }
}

// Configure routes
pub fn routes() -> Vec<rocket::Route> {
    routes![
        create_ticket,
        get_ticket,
        get_tickets_by_event,
        update_ticket,
        delete_ticket,
        check_availability,
        allocate_tickets,
    ]
}
