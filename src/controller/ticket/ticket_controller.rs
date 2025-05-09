use rocket::serde::json::{Json, json, Value};
use rocket::{State, get, post, put, delete};
use rocket::http::Status;
use uuid::Uuid;
use std::sync::Arc;

use crate::service::ticket::TicketService;

#[derive(serde::Deserialize, Debug)]
pub struct CreateTicketRequest {
    pub event_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub quantity_available: i32,
    pub ticket_type: String,  // Regular, VIP, etc.
    pub sale_start_date: Option<String>,  // ISO 8601 format
    pub sale_end_date: Option<String>,    // ISO 8601 format
}

#[derive(serde::Deserialize)]
pub struct PurchaseTicketRequest {
    pub quantity: i32,
}

#[derive(serde::Serialize, Debug)]
pub struct TicketResponse {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: i64,
    pub quantity_available: i32,
    pub quantity_sold: i32,
    pub ticket_type: String,
    pub status: String,
}

// Get a specific ticket by ID
#[get("/<id>")]
pub fn get_ticket(id: &str, ticket_service: &State<Arc<TicketService>>) -> Result<Json<Value>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest)
    };

    match ticket_service.get_ticket(ticket_id) {
        Ok(Some(ticket)) => Ok(Json(json!({
            "status": "success",
            "data": ticket
        }))),
        Ok(None) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError)
    }
}

// Get all tickets for an event
#[get("/event/<event_id>")]
pub fn get_tickets_by_event(event_id: &str, ticket_service: &State<Arc<TicketService>>) -> Result<Json<Value>, Status> {
    let parsed_event_id = match Uuid::parse_str(event_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest)
    };

    match ticket_service.get_tickets_by_event(parsed_event_id) {
        Ok(tickets) => Ok(Json(json!({
            "status": "success",
            "data": tickets
        }))),
        Err(_) => Err(Status::InternalServerError)
    }
}

// Create a new ticket
#[post("/", data = "<request>")]
pub fn create_ticket(request: Json<CreateTicketRequest>, ticket_service: &State<Arc<TicketService>>) -> Result<Json<Value>, Status> {
    println!("Received create ticket request: {:?}", request);
    
    let event_id = match Uuid::parse_str(&request.event_id) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Invalid UUID format for event_id: {} - Error: {}", request.event_id, e);
            return Err(Status::BadRequest);
        }
    };

    match ticket_service.create_ticket(
        event_id,
        request.name.clone(),
        request.description.clone(),
        request.price,
        request.quantity_available,
        request.ticket_type.clone(),
        request.sale_start_date.clone(),
        request.sale_end_date.clone(),
    ) {
        Ok(ticket) => {
            println!("Successfully created ticket: {:?}", ticket);
            Ok(Json(json!({
                "status": "success",
                "data": ticket
            })))
        },
        Err(e) => {
            eprintln!("Error creating ticket: {}", e);
            Err(Status::BadRequest)
        }
    }
}

// Purchase a ticket
#[put("/<id>/purchase", data = "<request>")]
pub fn purchase_ticket(
    id: &str,
    request: Json<PurchaseTicketRequest>,
    ticket_service: &State<Arc<TicketService>>
) -> Result<Json<Value>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest)
    };

    match ticket_service.purchase_ticket(ticket_id, request.quantity) {
        Ok(ticket) => Ok(Json(json!({
            "status": "success",
            "data": ticket
        }))),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Not enough tickets available") {
                return Err(Status::BadRequest);
            }
            Err(Status::InternalServerError)
        }
    }
}

// Delete a ticket
#[delete("/<id>")]
pub fn delete_ticket(id: &str, ticket_service: &State<Arc<TicketService>>) -> Result<Json<Value>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest)
    };

    match ticket_service.delete_ticket(ticket_id) {
        Ok(_) => Ok(Json(json!({
            "status": "success",
            "message": "Ticket deleted successfully"
        }))),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Cannot delete purchased tickets") {
                return Err(Status::Forbidden);
            }
            Err(Status::NotFound)
        }
    }
}

// Validate a ticket (mark as used)
#[put("/<id>/validate")]
pub fn validate_ticket(
    id: &str, 
    ticket_service: &State<Arc<TicketService>>
) -> Result<Json<Value>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest)
    };

    match ticket_service.validate_ticket(ticket_id) {
        Ok(ticket) => Ok(Json(json!({
            "status": "success",
            "data": ticket,
            "message": "Ticket validated successfully"
        }))),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("Ticket already used") {
                return Err(Status::BadRequest);
            } else if error_msg.contains("Ticket not found") {
                return Err(Status::NotFound);
            } else if error_msg.contains("Unauthorized") {
                return Err(Status::Forbidden);
            }
            Err(Status::InternalServerError)
        }
    }
}

// Get ticket status
#[get("/status/<id>")]
pub fn get_ticket_status(id: &str, ticket_service: &State<Arc<TicketService>>) -> Result<Json<Value>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest)
    };

    match ticket_service.get_ticket_status(ticket_id) {
        Ok(status) => Ok(Json(json!({
            "status": "success",
            "data": {
                "ticket_id": id,
                "status": status
            }
        }))),
        Err(_) => Err(Status::NotFound)
    }
}

