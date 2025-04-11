use crate::model::tiket::ticket::{Ticket, TicketStatus};
use crate::repository::tiket::TicketRepository;
use crate::service::tiket::ticket_service::TicketService;
use mockall::predicate::*;
use mockall::mock;
use rstest::*;
use uuid::Uuid;

// Import the mock from the model tests
use crate::model::tiket::tests::MockTicketRepository;

// Fixture for common objects
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

// Test create_ticket method
#[rstest]
fn test_create_ticket_success(event_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_save()
        .with(predicate::function(|ticket: &Ticket| {
            ticket.event_id == event_id && 
            ticket.ticket_type == "VIP" && 
            ticket.price == 100.0 && 
            ticket.quota == 50
        }))
        .times(1)
        .returning(|ticket| {
            let mut saved_ticket = ticket.clone();
            saved_ticket.id = Some(Uuid::new_v4());
            Ok(saved_ticket)
        });

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.create_ticket(event_id, "VIP".to_string(), 100.0, 50);
    
    assert!(result.is_ok());
    let ticket = result.unwrap();
    assert!(ticket.id.is_some());
    assert_eq!(ticket.event_id, event_id);
    assert_eq!(ticket.ticket_type, "VIP");
    assert_eq!(ticket.price, 100.0);
    assert_eq!(ticket.quota, 50);
    assert_eq!(ticket.status, TicketStatus::AVAILABLE);
}

#[rstest]
fn test_create_ticket_invalid_price() {
    let mock_repo = MockTicketRepository::new();
    let service = TicketService::new(Box::new(mock_repo));
    
    let event_id = Uuid::new_v4();
    let result = service.create_ticket(event_id, "VIP".to_string(), -50.0, 50);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Ticket price cannot be negative");
}

#[rstest]
fn test_create_ticket_repository_error(event_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_save()
        .times(1)
        .returning(|_| Err("Database error".to_string()));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.create_ticket(event_id, "VIP".to_string(), 100.0, 50);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Database error");
}

// Test get_ticket method
#[rstest]
fn test_get_ticket_found(ticket_id: Uuid, sample_ticket: Ticket) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(move |_| Ok(Some(sample_ticket.clone())));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.get_ticket(&ticket_id);
    
    assert!(result.is_ok());
    let ticket_option = result.unwrap();
    assert!(ticket_option.is_some());
    let ticket = ticket_option.unwrap();
    assert_eq!(ticket.id, Some(ticket_id));
}

#[rstest]
fn test_get_ticket_not_found(ticket_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(|_| Ok(None));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.get_ticket(&ticket_id);
    
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[rstest]
fn test_get_ticket_repository_error(ticket_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(|_| Err("Database error".to_string()));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.get_ticket(&ticket_id);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Database error");
}

// Test get_tickets_by_event method
#[rstest]
fn test_get_tickets_by_event(event_id: Uuid, sample_ticket: Ticket) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_find_by_event_id()
        .with(eq(event_id))
        .times(1)
        .returning(move |_| Ok(vec![sample_ticket.clone()]));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.get_tickets_by_event(&event_id);
    
    assert!(result.is_ok());
    let tickets = result.unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].event_id, event_id);
}

#[rstest]
fn test_get_tickets_by_event_empty(event_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_find_by_event_id()
        .with(eq(event_id))
        .times(1)
        .returning(|_| Ok(vec![]));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.get_tickets_by_event(&event_id);
    
    assert!(result.is_ok());
    let tickets = result.unwrap();
    assert!(tickets.is_empty());
}

#[rstest]
fn test_get_tickets_by_event_repository_error(event_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_find_by_event_id()
        .with(eq(event_id))
        .times(1)
        .returning(|_| Err("Database error".to_string()));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.get_tickets_by_event(&event_id);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Database error");
}

// Test update_ticket method
#[rstest]
fn test_update_ticket_success(ticket_id: Uuid, sample_ticket: Ticket) {
    let mut mock_repo = MockTicketRepository::new();
    
    // First fetch the ticket
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(move |_| Ok(Some(sample_ticket.clone())));
        
    // Then update it
    mock_repo.expect_update()
        .with(predicate::function(move |ticket: &Ticket| {
            ticket.id == Some(ticket_id) && 
            ticket.ticket_type == "Premium" &&
            ticket.price == 150.0 &&
            ticket.quota == 40
        }))
        .times(1)
        .returning(|ticket| Ok(ticket.clone()));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.update_ticket(
        &ticket_id, 
        Some("Premium".to_string()), 
        Some(150.0), 
        Some(40)
    );
    
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.ticket_type, "Premium");
    assert_eq!(updated.price, 150.0);
    assert_eq!(updated.quota, 40);
}

#[rstest]
fn test_update_ticket_partial_update(ticket_id: Uuid, sample_ticket: Ticket) {
    let mut mock_repo = MockTicketRepository::new();
    
    // First fetch the ticket
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(move |_| Ok(Some(sample_ticket.clone())));
        
    // Then update only price
    mock_repo.expect_update()
        .with(predicate::function(move |ticket: &Ticket| {
            ticket.id == Some(ticket_id) && 
            ticket.ticket_type == "VIP" &&  // unchanged
            ticket.price == 150.0 &&        // changed
            ticket.quota == 50              // unchanged
        }))
        .times(1)
        .returning(|ticket| Ok(ticket.clone()));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.update_ticket(
        &ticket_id, 
        None,            // Don't update type
        Some(150.0),     // Update price
        None             // Don't update quota
    );
    
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.ticket_type, "VIP"); // unchanged
    assert_eq!(updated.price, 150.0);       // changed
    assert_eq!(updated.quota, 50);          // unchanged
}

#[rstest]
fn test_update_ticket_not_found(ticket_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(|_| Ok(None));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.update_ticket(
        &ticket_id, 
        Some("Premium".to_string()), 
        Some(150.0), 
        Some(40)
    );
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Ticket not found");
}

// Test delete_ticket method
#[rstest]
fn test_delete_ticket_success(ticket_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_delete()
        .with(eq(ticket_id))
        .times(1)
        .returning(|_| Ok(()));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.delete_ticket(&ticket_id);
    
    assert!(result.is_ok());
}

#[rstest]
fn test_delete_ticket_not_found(ticket_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    mock_repo.expect_delete()
        .with(eq(ticket_id))
        .times(1)
        .returning(|_| Err("Ticket not found".to_string()));

    let service = TicketService::new(Box::new(mock_repo));
    let result = service.delete_ticket(&ticket_id);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Ticket not found");
}

// Test allocate_tickets method
#[rstest]
fn test_allocate_tickets_success(ticket_id: Uuid, sample_ticket: Ticket) {
    let mut mock_repo = MockTicketRepository::new();
    
    // Find the ticket
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(move |_| Ok(Some(sample_ticket.clone())));
    
    // Update the quota
    mock_repo.expect_update_quota()
        .with(eq(ticket_id), eq(40))
        .times(1)
        .returning(|_, quota| {
            let mut updated = sample_ticket.clone();
            updated.quota = quota;
            Ok(updated)
        });
    
    let service = TicketService::new(Box::new(mock_repo));
    let result = service.allocate_tickets(&ticket_id, 10);
    
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[rstest]
fn test_allocate_tickets_insufficient(ticket_id: Uuid, sample_ticket: Ticket) {
    let mut mock_repo = MockTicketRepository::new();
    
    // Find the ticket
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(move |_| Ok(Some(sample_ticket.clone())));
    
    // No update_quota call expected because there's not enough tickets
    
    let service = TicketService::new(Box::new(mock_repo));
    let result = service.allocate_tickets(&ticket_id, 60); // More than available
    
    assert!(result.is_ok());
    assert!(!result.unwrap()); // Should return false for insufficient tickets
}

// Test check_availability method
#[rstest]
fn test_check_availability_sufficient(ticket_id: Uuid, sample_ticket: Ticket) {
    let mut mock_repo = MockTicketRepository::new();
    
    // Find the ticket
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(move |_| Ok(Some(sample_ticket.clone())));
    
    let service = TicketService::new(Box::new(mock_repo));
    let result = service.check_availability(&ticket_id, 10);
    
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[rstest]
fn test_check_availability_insufficient(ticket_id: Uuid, sample_ticket: Ticket) {
    let mut mock_repo = MockTicketRepository::new();
    
    // Find the ticket
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(move |_| Ok(Some(sample_ticket.clone())));
    
    let service = TicketService::new(Box::new(mock_repo));
    let result = service.check_availability(&ticket_id, 60); // More than available
    
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[rstest]
fn test_check_availability_ticket_not_found(ticket_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    // Find the ticket (not found)
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(|_| Ok(None));
    
    let service = TicketService::new(Box::new(mock_repo));
    let result = service.check_availability(&ticket_id, 10);
    
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Ticket not found");
}

// Test edge cases for ticket service
#[rstest]
fn test_allocate_zero_tickets(ticket_id: Uuid, sample_ticket: Ticket) {
    let mut mock_repo = MockTicketRepository::new();
    
    // Find the ticket
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(move |_| Ok(Some(sample_ticket.clone())));
    
    // No update_quota call expected for zero allocation
    
    let service = TicketService::new(Box::new(mock_repo));
    let result = service.allocate_tickets(&ticket_id, 0);
    
    assert!(result.is_ok());
    assert!(result.unwrap()); // Should succeed but not change quota
}

#[rstest]
fn test_allocate_last_available_tickets(ticket_id: Uuid, event_id: Uuid) {
    let mut mock_repo = MockTicketRepository::new();
    
    // Create ticket with only 5 available
    let mut ticket = Ticket::new(
        event_id,
        "VIP".to_string(),
        100.0,
        5,
    );
    ticket.id = Some(ticket_id);
    
    // Find the ticket
    mock_repo.expect_find_by_id()
        .with(eq(ticket_id))
        .times(1)
        .returning(move |_| Ok(Some(ticket.clone())));
    
    // Update to zero quota (sold out)
    mock_repo.expect_update_quota()
        .with(eq(ticket_id), eq(0))
        .times(1)
        .returning(|_, quota| {
            let mut updated = ticket.clone();
            updated.quota = quota;
            updated.status = TicketStatus::SOLD_OUT;
            Ok(updated)
        });
    
    let service = TicketService::new(Box::new(mock_repo));
    let result = service.allocate_tickets(&ticket_id, 5);
    
    assert!(result.is_ok());
    assert!(result.unwrap());
}
