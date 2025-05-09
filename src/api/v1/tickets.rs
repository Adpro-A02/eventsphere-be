use rocket::{routes, Route, State};
use rocket::serde::json::Json;
use rocket::http::Status;
use uuid::Uuid;
use crate::controller::ticket::ticket_controller;
use crate::service::ticket::ticket_service::TicketService;
use crate::common::response::ApiResponse;
use crate::model::ticket::ticket::Ticket;

/// Collection of ticket-related routes
pub fn routes() -> Vec<Route> {
    routes![
        create_ticket,
        get_ticket,
        get_tickets_by_event,
        update_ticket,
        delete_ticket,
        check_availability,
        allocate_tickets,
        purchase_ticket,
        validate_ticket
    ]
}

/// Create a new event ticket
/// 
/// Returns the newly created ticket.
#[post("/tickets", format = "json", data = "<request>")]
async fn create_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    request: Json<ticket_controller::CreateTicketRequest>
) -> Result<Json<ApiResponse<Ticket>>, Status> {
    let event_id = match Uuid::parse_str(&request.event_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };

    if request.ticket_type.is_empty() || request.price < 0.0 {
        return Err(Status::BadRequest);
    }

    match service.create_ticket(event_id, request.ticket_type.clone(), request.price, request.quota) {
        Ok(ticket) => Ok(ApiResponse::created("Ticket created successfully", ticket)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Get a ticket by ID
#[get("/tickets/<id>")]
async fn get_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    id: &str
) -> Result<Json<ApiResponse<Ticket>>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.get_ticket(&ticket_id) {
        Ok(Some(ticket)) => Ok(ApiResponse::success("Ticket retrieved successfully", ticket)),
        Ok(None) => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Get all tickets for an event
#[get("/events/<id>/tickets")]
async fn get_tickets_by_event(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    id: &str
) -> Result<Json<ApiResponse<Vec<Ticket>>>, Status> {
    let event_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.get_tickets_by_event(&event_id) {
        Ok(tickets) => Ok(ApiResponse::success("Event tickets retrieved successfully", tickets)),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Update ticket details
#[put("/tickets/<id>", format = "json", data = "<request>")]
async fn update_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    id: &str,
    request: Json<ticket_controller::UpdateTicketRequest>
) -> Result<Json<ApiResponse<Ticket>>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.update_ticket(&ticket_id, request.ticket_type.clone(), request.price, request.quota) {
        Ok(ticket) => Ok(ApiResponse::success("Ticket updated successfully", ticket)),
        Err(e) if e == "Ticket not found" => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Delete a ticket
#[delete("/tickets/<id>")]
async fn delete_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    id: &str
) -> Result<Json<ApiResponse<()>>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.delete_ticket(&ticket_id) {
        Ok(_) => Ok(ApiResponse::success("Ticket deleted successfully", ())),
        Err(e) if e == "Ticket not found" => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Check ticket availability
#[get("/tickets/<id>/availability?<quantity>")]
async fn check_availability(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    id: &str,
    quantity: u32
) -> Result<Json<ApiResponse<bool>>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.check_availability(&ticket_id, quantity) {
        Ok(available) => Ok(ApiResponse::success("Ticket availability checked", available)),
        Err(e) if e == "Ticket not found" => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Allocate tickets
#[post("/tickets/<id>/allocate", format = "json", data = "<request>")]
async fn allocate_tickets(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    id: &str,
    request: Json<ticket_controller::AllocateTicketsRequest>
) -> Result<Json<ApiResponse<bool>>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };
    
    match service.allocate_tickets(&ticket_id, request.quantity) {
        Ok(true) => Ok(ApiResponse::success("Tickets allocated successfully", true)),
        Ok(false) => Ok(ApiResponse::success("Insufficient tickets available", false)),
        Err(e) if e == "Ticket not found" => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Purchase ticket
#[post("/tickets/<id>/purchase", format = "json", data = "<request>")]
async fn purchase_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    id: &str,
    request: Json<ticket_controller::PurchaseTicketRequest>
) -> Result<Json<ApiResponse<PurchaseResponse>>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };

    let user_id = match Uuid::parse_str(&request.user_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };

    if request.quantity == 0 {
        return Err(Status::BadRequest);
    }

    match service.purchase_ticket(user_id, &ticket_id, request.quantity, request.payment_method.clone()) {
        Ok((ticket, transaction_id)) => {
            let response = PurchaseResponse {
                ticket,
                transaction_id,
            };
            Ok(ApiResponse::success("Ticket purchased successfully", response))
        },
        Err(e) if e.contains("Not enough tickets available") => Err(Status::BadRequest),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Validate a ticket
#[put("/tickets/<id>/validate", format = "json", data = "<request>")]
async fn validate_ticket(
    service: &State<Box<dyn TicketService + Send + Sync>>,
    id: &str,
    request: Json<ticket_controller::ValidateTicketRequest>
) -> Result<Json<ApiResponse<Ticket>>, Status> {
    let ticket_id = match Uuid::parse_str(id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };

    let validator_id = match Uuid::parse_str(&request.validator_id) {
        Ok(id) => id,
        Err(_) => return Err(Status::BadRequest),
    };

    match service.validate_ticket(&ticket_id, &validator_id, &request.role) {
        Ok(ticket) => Ok(ApiResponse::success("Ticket validated successfully", ticket)),
        Err(e) if e.contains("Unauthorized") => Err(Status::Forbidden),
        Err(e) if e.contains("already been used") => Err(Status::BadRequest),
        Err(e) if e.contains("has not been purchased") => Err(Status::BadRequest),  
        Err(e) if e.contains("Ticket not found") => Err(Status::NotFound),
        Err(_) => Err(Status::InternalServerError),
    }
}

/// Response structure for ticket purchase
#[derive(serde::Serialize)]
struct PurchaseResponse {
    ticket: Ticket,
    transaction_id: Uuid,
}
