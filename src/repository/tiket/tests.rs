use crate::model::tiket::ticket::{Ticket, TicketStatus};
use crate::repository::tiket::TicketRepository;
use rstest::*;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// In-memory repository implementation for testing
struct InMemoryTicketRepository {
    tickets: Arc<Mutex<HashMap<Uuid, Ticket>>>,
}

impl InMemoryTicketRepository {
    fn new() -> Self {
        InMemoryTicketRepository {
            tickets: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl TicketRepository for InMemoryTicketRepository {
    fn save(&self, mut ticket: Ticket) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        
        // Generate UUID if not present
        if ticket.id.is_none() {
            ticket.id = Some(Uuid::new_v4());
        }
        
        let id = ticket.id.unwrap();
        tickets.insert(id, ticket.clone());
        
        Ok(ticket)
    }
    
    fn find_by_id(&self, id: &Uuid) -> Result<Option<Ticket>, String> {
        let tickets = self.tickets.lock().unwrap();
        Ok(tickets.get(id).cloned())
    }
    
    fn find_by_event_id(&self, event_id: &Uuid) -> Result<Vec<Ticket>, String> {
        let tickets = self.tickets.lock().unwrap();
        let matching_tickets: Vec<Ticket> = tickets.values()
            .filter(|ticket| ticket.event_id == *event_id)
            .cloned()
            .collect();
            
        Ok(matching_tickets)
    }
    
    fn update(&self, ticket: Ticket) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        
        // Ensure ID exists
        let id = ticket.id.ok_or("Ticket ID is required for update")?;
        
        if !tickets.contains_key(&id) {
            return Err("Ticket not found".to_string());
        }
        
        tickets.insert(id, ticket.clone());
        Ok(ticket)
    }
    
    fn delete(&self, id: &Uuid) -> Result<(), String> {
        let mut tickets = self.tickets.lock().unwrap();
        
        if tickets.remove(id).is_none() {
            return Err("Ticket not found".to_string());
        }
        
        Ok(())
    }
    
    fn update_quota(&self, id: &Uuid, new_quota: u32) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        
        let ticket = tickets.get_mut(id)
            .ok_or_else(|| "Ticket not found".to_string())?;
            
        ticket.update_quota(new_quota);
        
        Ok(ticket.clone())
    }
}

// Fixture for repository
#[fixture]
fn repo() -> impl TicketRepository {
    InMemoryTicketRepository::new()
}

// Fixture for an event ID
#[fixture]
fn event_id() -> Uuid {
    Uuid::new_v4()
}

// Test repository methods
#[rstest]
fn test_save_ticket(repo: impl TicketRepository, event_id: Uuid) {
    // Arrange
    let ticket = Ticket::new(
        event_id,
        "VIP".to_string(),
        100.0,
        50,
    );
    
    // Act
    let saved_ticket = repo.save(ticket).unwrap();
    
    // Assert
    assert!(saved_ticket.id.is_some());
    assert_eq!(saved_ticket.event_id, event_id);
    assert_eq!(saved_ticket.ticket_type, "VIP");
    assert_eq!(saved_ticket.price, 100.0);
    assert_eq!(saved_ticket.quota, 50);
}

#[rstest]
fn test_find_by_id(repo: impl TicketRepository, event_id: Uuid) {
    // Arrange
    let ticket = Ticket::new(
        event_id,
        "VIP".to_string(),
        100.0,
        50,
    );
    let saved_ticket = repo.save(ticket).unwrap();
    
    // Act
    let found_ticket = repo.find_by_id(&saved_ticket.id.unwrap()).unwrap();
    
    // Assert
    assert!(found_ticket.is_some());
    let found = found_ticket.unwrap();
    assert_eq!(found.ticket_type, "VIP");
    assert_eq!(found.event_id, event_id);
}

#[rstest]
fn test_find_nonexistent_ticket(repo: impl TicketRepository) {
    // Arrange
    let nonexistent_id = Uuid::new_v4();
    
    // Act
    let result = repo.find_by_id(&nonexistent_id).unwrap();
    
    // Assert
    assert!(result.is_none());
}

#[rstest]
fn test_find_by_event_id(repo: impl TicketRepository, event_id: Uuid) {
    // Arrange
    let ticket1 = Ticket::new(
        event_id,
        "VIP".to_string(),
        100.0,
        50,
    );
    let ticket2 = Ticket::new(
        event_id,
        "Regular".to_string(),
        50.0,
        100,
    );
    let different_event_id = Uuid::new_v4();
    let ticket3 = Ticket::new(
        different_event_id,
        "VIP".to_string(),
        80.0,
        30,
    );
    
    repo.save(ticket1).unwrap();
    repo.save(ticket2).unwrap();
    repo.save(ticket3).unwrap();
    
    // Act
    let tickets = repo.find_by_event_id(&event_id).unwrap();
    
    // Assert
    assert_eq!(tickets.len(), 2);
    assert!(tickets.iter().all(|t| t.event_id == event_id));
}

#[rstest]
fn test_update_ticket(repo: impl TicketRepository, event_id: Uuid) {
    // Arrange
    let ticket = Ticket::new(
        event_id,
        "VIP".to_string(),
        100.0,
        50,
    );
    let saved_ticket = repo.save(ticket).unwrap();
    let mut ticket_to_update = saved_ticket.clone();
    
    // Act
    ticket_to_update.ticket_type = "Premium".to_string();
    ticket_to_update.price = 120.0;
    let updated = repo.update(ticket_to_update).unwrap();
    
    // Assert
    assert_eq!(updated.ticket_type, "Premium");
    assert_eq!(updated.price, 120.0);
    
    // Verify changes persisted
    let retrieved = repo.find_by_id(&saved_ticket.id.unwrap()).unwrap().unwrap();
    assert_eq!(retrieved.ticket_type, "Premium");
    assert_eq!(retrieved.price, 120.0);
}

#[rstest]
fn test_update_nonexistent_ticket(repo: impl TicketRepository, event_id: Uuid) {
    // Arrange
    let mut ticket = Ticket::new(
        event_id,
        "VIP".to_string(),
        100.0,
        50,
    );
    ticket.id = Some(Uuid::new_v4());
    
    // Act
    let result = repo.update(ticket);
    
    // Assert
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Ticket not found");
}

#[rstest]
fn test_delete_ticket(repo: impl TicketRepository, event_id: Uuid) {
    // Arrange
    let ticket = Ticket::new(
        event_id,
        "VIP".to_string(),
        100.0,
        50,
    );
    let saved_ticket = repo.save(ticket).unwrap();
    let id = saved_ticket.id.unwrap();
    
    // Act
    let delete_result = repo.delete(&id);
    let find_result = repo.find_by_id(&id).unwrap();
    
    // Assert
    assert!(delete_result.is_ok());
    assert!(find_result.is_none());
}

#[rstest]
fn test_delete_nonexistent_ticket(repo: impl TicketRepository) {
    // Arrange
    let nonexistent_id = Uuid::new_v4();
    
    // Act
    let result = repo.delete(&nonexistent_id);
    
    // Assert
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Ticket not found");
}

#[rstest]
fn test_update_quota(repo: impl TicketRepository, event_id: Uuid) {
    // Arrange
    let ticket = Ticket::new(
        event_id,
        "VIP".to_string(),
        100.0,
        50,
    );
    let saved_ticket = repo.save(ticket).unwrap();
    let id = saved_ticket.id.unwrap();
    
    // Act
    let updated_ticket = repo.update_quota(&id, 25).unwrap();
    
    // Assert
    assert_eq!(updated_ticket.quota, 25);
    
    // Verify changes persisted
    let retrieved = repo.find_by_id(&id).unwrap().unwrap();
    assert_eq!(retrieved.quota, 25);
    assert_eq!(retrieved.status, TicketStatus::AVAILABLE);
}

#[rstest]
fn test_update_quota_to_zero(repo: impl TicketRepository, event_id: Uuid) {
    // Arrange
    let ticket = Ticket::new(
        event_id,
        "VIP".to_string(),
        100.0,
        50,
    );
    let saved_ticket = repo.save(ticket).unwrap();
    let id = saved_ticket.id.unwrap();
    
    // Act
    let updated_ticket = repo.update_quota(&id, 0).unwrap();
    
    // Assert
    assert_eq!(updated_ticket.quota, 0);
    assert_eq!(updated_ticket.status, TicketStatus::SOLD_OUT);
    
    // Verify changes persisted
    let retrieved = repo.find_by_id(&id).unwrap().unwrap();
    assert_eq!(retrieved.quota, 0);
    assert_eq!(retrieved.status, TicketStatus::SOLD_OUT);
}

#[rstest]
fn test_update_quota_nonexistent(repo: impl TicketRepository) {
    // Arrange
    let nonexistent_id = Uuid::new_v4();
    
    // Act
    let result = repo.update_quota(&nonexistent_id, 10);
    
    // Assert
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Ticket not found");
}
