use crate::model::ticket::ticket::Ticket;
use crate::service::ticket::ticket_service::TicketService;
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

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PurchaseTicketRequest {
    pub user_id: String,
    pub quantity: u32,
    pub payment_method: String,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ValidateTicketRequest {
    pub validator_id: String,
    pub role: String, // "admin" or "organizer"
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

// Purchase tickets
#[post("/tickets/<ticket_id>/purchase", format = "json", data = "<request>")]
pub async fn purchase_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    ticket_id: &str,
    request: Json<PurchaseTicketRequest>
) -> (Status, Value) {
    // Parse ticket_id from string to Uuid
    let ticket_uuid = match Uuid::parse_str(ticket_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid ticket_id format")),
    };
    
    // Parse user_id from string to Uuid
    let user_uuid = match Uuid::parse_str(&request.user_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid user_id format")),
    };

    // Validate input
    if request.quantity <= 0 {
        return (Status::BadRequest, json_error("Quantity must be greater than 0"));
    }

    // Call service to purchase ticket
    match service.purchase_ticket(
        user_uuid,
        &ticket_uuid,
        request.quantity,
        request.payment_method.clone()
    ) {
        Ok((ticket, transaction_id)) => (
            Status::Ok, 
            json!({
                "success": true,
                "data": {
                    "ticket": ticket,
                    "transaction_id": transaction_id,
                    "message": "Ticket purchased successfully"
                }
            })
        ),
        Err(error) if error.contains("Not enough tickets available") => 
            (Status::BadRequest, json_error(&error)),
        Err(error) => (Status::InternalServerError, json_error(&error)),
    }
}

// Validate a ticket
#[put("/tickets/<ticket_id>/validate", format = "json", data = "<request>")]
pub async fn validate_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    ticket_id: &str,
    request: Json<ValidateTicketRequest>
) -> (Status, Value) {
    // Parse ticket_id from string to Uuid
    let ticket_uuid = match Uuid::parse_str(ticket_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid ticket_id format")),
    };
    
    // Parse validator_id from string to Uuid
    let validator_uuid = match Uuid::parse_str(&request.validator_id) {
        Ok(id) => id,
        Err(_) => return (Status::BadRequest, json_error("Invalid validator_id format")),
    };

    // Call service to validate ticket
    match service.validate_ticket(&ticket_uuid, &validator_uuid, &request.role) {
        Ok(ticket) => (
            Status::Ok, 
            json!({
                "success": true,
                "data": ticket,
                "message": "Ticket validated successfully"
            })
        ),
        Err(error) if error.contains("Unauthorized") => 
            (Status::Forbidden, json_error(&error)),
        Err(error) if error.contains("already been used") => 
            (Status::BadRequest, json_error(&error)),
        Err(error) if error.contains("has not been purchased") => 
            (Status::BadRequest, json_error(&error)),
        Err(error) if error.contains("Ticket not found") => 
            (Status::NotFound, json_error(&error)),
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
        purchase_ticket,
        validate_ticket,
    ]
}
