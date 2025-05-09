use crate::model::ticket::ticket::{Ticket, TicketStatus};
use crate::service::ticket::ticket_service::TicketService;
use crate::controller::ticket::ticket_controller::*;
use mockall::predicate::*;
use mockall::mock;
use rstest::*;
use uuid::Uuid;
use rocket::http::{ContentType, Status};
use rocket::local::blocking::Client;
use rocket::serde::json;
use serde_json::{json, Value};

// Import mock from the model tests
use crate::model::ticket::tests::MockTicketService;

// Test fixtures
#[fixture]
fn client(mock_service: MockTicketService) -> Client {
    let rocket = rocket::build()
        .mount("/api", routes())
        .manage(mock_service);
    Client::tracked(rocket).expect("valid rocket instance")
}

#[fixture]
fn event_id() -> Uuid {
    Uuid::new_v4()
}

#[fixture]
fn ticket_id() -> Uuid {
    Uuid::new_v4()
}

#[fixture]
fn sample_ticket(event_id: Uuid, ticket_id: Uuid) -> Ticket {
    let mut ticket = Ticket::new(
        event_id,
        "VIP".to_string(),
        100.0,
        50,
    );
    ticket.id = Some(ticket_id);
    ticket
}

// Tests for create ticket endpoint
#[rstest]
fn test_create_ticket_success(mut mock_service: MockTicketService, event_id: Uuid) {
    // Arrange
    mock_service.expect_create_ticket()
        .with(eq(event_id), eq("VIP".to_string()), eq(100.0), eq(50))
        .times(1)
        .returning(|event_id, ticket_type, price, quota| {
            let mut ticket = Ticket::new(event_id, ticket_type, price, quota);
            ticket.id = Some(Uuid::new_v4());
            Ok(ticket)
        });

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.post("/api/events/tickets")
        .header(ContentType::JSON)
        .body(json!({
            "event_id": event_id.to_string(),
            "ticket_type": "VIP",
            "price": 100.0,
            "quota": 50
        }).to_string())
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::Created);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(response_body["success"].as_bool().unwrap());
    assert!(response_body["data"]["id"].as_str().is_some());
    assert_eq!(response_body["data"]["ticket_type"], "VIP");
    assert_eq!(response_body["data"]["price"], 100.0);
    assert_eq!(response_body["data"]["quota"], 50);
}

#[rstest]
fn test_create_ticket_invalid_input(mut mock_service: MockTicketService) {
    // No expectations on mock service since validation should fail before service call
    
    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act - Missing event_id
    let response = client.post("/api/events/tickets")
        .header(ContentType::JSON)
        .body(json!({
            "ticket_type": "VIP",
            "price": 100.0,
            "quota": 50
        }).to_string())
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::BadRequest);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(!response_body["success"].as_bool().unwrap());
    assert!(response_body["error"].as_str().unwrap().contains("event_id"));
}

#[rstest]
fn test_create_ticket_service_error(mut mock_service: MockTicketService, event_id: Uuid) {
    // Arrange
    mock_service.expect_create_ticket()
        .with(eq(event_id), eq("VIP".to_string()), eq(100.0), eq(50))
        .times(1)
        .returning(|_, _, _, _| {
            Err("Service error".to_string())
        });

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.post("/api/events/tickets")
        .header(ContentType::JSON)
        .body(json!({
            "event_id": event_id.to_string(),
            "ticket_type": "VIP",
            "price": 100.0,
            "quota": 50
        }).to_string())
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::InternalServerError);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(!response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["error"], "Service error");
}

// Tests for get ticket endpoint
#[rstest]
fn test_get_ticket_success(mut mock_service: MockTicketService, ticket_id: Uuid, sample_ticket: Ticket) {
    // Arrange
    mock_service.expect_get_ticket()
        .with(eq(ticket_id))
        .times(1)
        .returning(move |_| Ok(Some(sample_ticket.clone())));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.get(format!("/api/tickets/{}", ticket_id))
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::Ok);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["data"]["id"], ticket_id.to_string());
    assert_eq!(response_body["data"]["ticket_type"], "VIP");
    assert_eq!(response_body["data"]["price"], 100.0);
    assert_eq!(response_body["data"]["quota"], 50);
}

#[rstest]
fn test_get_ticket_not_found(mut mock_service: MockTicketService, ticket_id: Uuid) {
    // Arrange
    mock_service.expect_get_ticket()
        .with(eq(ticket_id))
        .times(1)
        .returning(|_| Ok(None));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.get(format!("/api/tickets/{}", ticket_id))
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::NotFound);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(!response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["error"], "Ticket not found");
}

#[rstest]
fn test_get_tickets_by_event_success(mut mock_service: MockTicketService, event_id: Uuid, sample_ticket: Ticket) {
    // Arrange
    mock_service.expect_get_tickets_by_event()
        .with(eq(event_id))
        .times(1)
        .returning(move |_| Ok(vec![sample_ticket.clone()]));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.get(format!("/api/events/{}/tickets", event_id))
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::Ok);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["data"].as_array().unwrap().len(), 1);
    assert_eq!(response_body["data"][0]["ticket_type"], "VIP");
    assert_eq!(response_body["data"][0]["price"], 100.0);
}

// Tests for update ticket endpoint
#[rstest]
fn test_update_ticket_success(mut mock_service: MockTicketService, ticket_id: Uuid, sample_ticket: Ticket) {
    // Arrange
    let mut updated_ticket = sample_ticket.clone();
    updated_ticket.ticket_type = "Premium".to_string();
    updated_ticket.price = 150.0;
    
    mock_service.expect_update_ticket()
        .with(eq(ticket_id), eq(Some("Premium".to_string())), eq(Some(150.0)), eq(None))
        .times(1)
        .returning(move |_, _, _, _| Ok(updated_ticket.clone()));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.put(format!("/api/tickets/{}", ticket_id))
        .header(ContentType::JSON)
        .body(json!({
            "ticket_type": "Premium",
            "price": 150.0
        }).to_string())
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::Ok);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["data"]["ticket_type"], "Premium");
    assert_eq!(response_body["data"]["price"], 150.0);
}

#[rstest]
fn test_update_ticket_not_found(mut mock_service: MockTicketService, ticket_id: Uuid) {
    // Arrange
    mock_service.expect_update_ticket()
        .with(eq(ticket_id), eq(Some("Premium".to_string())), eq(Some(150.0)), eq(None))
        .times(1)
        .returning(|_, _, _, _| Err("Ticket not found".to_string()));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.put(format!("/api/tickets/{}", ticket_id))
        .header(ContentType::JSON)
        .body(json!({
            "ticket_type": "Premium",
            "price": 150.0
        }).to_string())
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::NotFound);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(!response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["error"], "Ticket not found");
}

// Tests for delete ticket endpoint
#[rstest]
fn test_delete_ticket_success(mut mock_service: MockTicketService, ticket_id: Uuid) {
    // Arrange
    mock_service.expect_delete_ticket()
        .with(eq(ticket_id))
        .times(1)
        .returning(|_| Ok(()));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.delete(format!("/api/tickets/{}", ticket_id))
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::Ok);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["message"], "Ticket deleted successfully");
}

#[rstest]
fn test_delete_ticket_not_found(mut mock_service: MockTicketService, ticket_id: Uuid) {
    // Arrange
    mock_service.expect_delete_ticket()
        .with(eq(ticket_id))
        .times(1)
        .returning(|_| Err("Ticket not found".to_string()));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.delete(format!("/api/tickets/{}", ticket_id))
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::NotFound);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(!response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["error"], "Ticket not found");
}

// Tests for check availability endpoint
#[rstest]
fn test_check_availability_success(mut mock_service: MockTicketService, ticket_id: Uuid) {
    // Arrange
    mock_service.expect_check_availability()
        .with(eq(ticket_id), eq(10))
        .times(1)
        .returning(|_, _| Ok(true));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.get(format!("/api/tickets/{}/availability?quantity=10", ticket_id))
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::Ok);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(response_body["success"].as_bool().unwrap());
    assert!(response_body["available"].as_bool().unwrap());
}

#[rstest]
fn test_check_availability_insufficient(mut mock_service: MockTicketService, ticket_id: Uuid) {
    // Arrange
    mock_service.expect_check_availability()
        .with(eq(ticket_id), eq(100))
        .times(1)
        .returning(|_, _| Ok(false));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.get(format!("/api/tickets/{}/availability?quantity=100", ticket_id))
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::Ok);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(response_body["success"].as_bool().unwrap());
    assert!(!response_body["available"].as_bool().unwrap());
}

#[rstest]
fn test_check_availability_not_found(mut mock_service: MockTicketService, ticket_id: Uuid) {
    // Arrange
    mock_service.expect_check_availability()
        .with(eq(ticket_id), eq(10))
        .times(1)
        .returning(|_, _| Err("Ticket not found".to_string()));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.get(format!("/api/tickets/{}/availability?quantity=10", ticket_id))
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::NotFound);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(!response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["error"], "Ticket not found");
}

// Tests for allocate tickets endpoint
#[rstest]
fn test_allocate_tickets_success(mut mock_service: MockTicketService, ticket_id: Uuid) {
    // Arrange
    mock_service.expect_allocate_tickets()
        .with(eq(ticket_id), eq(5))
        .times(1)
        .returning(|_, _| Ok(true));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.post(format!("/api/tickets/{}/allocate", ticket_id))
        .header(ContentType::JSON)
        .body(json!({
            "quantity": 5
        }).to_string())
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::Ok);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(response_body["success"].as_bool().unwrap());
    assert!(response_body["allocated"].as_bool().unwrap());
}

#[rstest]
fn test_allocate_tickets_insufficient(mut mock_service: MockTicketService, ticket_id: Uuid) {
    // Arrange
    mock_service.expect_allocate_tickets()
        .with(eq(ticket_id), eq(100))
        .times(1)
        .returning(|_, _| Ok(false));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.post(format!("/api/tickets/{}/allocate", ticket_id))
        .header(ContentType::JSON)
        .body(json!({
            "quantity": 100
        }).to_string())
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::BadRequest);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(!response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["error"], "Insufficient tickets available");
}

#[rstest]
fn test_allocate_tickets_not_found(mut mock_service: MockTicketService, ticket_id: Uuid) {
    // Arrange
    mock_service.expect_allocate_tickets()
        .with(eq(ticket_id), eq(5))
        .times(1)
        .returning(|_, _| Err("Ticket not found".to_string()));

    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(mock_service)
    ).expect("valid rocket instance");

    // Act
    let response = client.post(format!("/api/tickets/{}/allocate", ticket_id))
        .header(ContentType::JSON)
        .body(json!({
            "quantity": 5
        }).to_string())
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::NotFound);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(!response_body["success"].as_bool().unwrap());
    assert_eq!(response_body["error"], "Ticket not found");
}

// Test invalid UUID handling
#[rstest]
fn test_invalid_uuid_handling() {
    let client = Client::tracked(
        rocket::build()
            .mount("/api", routes())
            .manage(MockTicketService::new())
    ).expect("valid rocket instance");

    // Act
    let response = client.get("/api/tickets/not-a-uuid")
        .dispatch();

    // Assert
    assert_eq!(response.status(), Status::BadRequest);
    
    let response_body: Value = serde_json::from_str(&response.into_string().unwrap()).unwrap();
    assert!(!response_body["success"].as_bool().unwrap());
    assert!(response_body["error"].as_str().unwrap().contains("Invalid UUID"));
}
