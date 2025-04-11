use crate::model::tiket::ticket::{Ticket, TicketStatus};
use crate::repository::tiket::TicketRepository;
use uuid::Uuid;

pub struct TicketService {
    repository: Box<dyn TicketRepository>,
}

impl TicketService {
    pub fn new(repository: Box<dyn TicketRepository>) -> Self {
        Self { repository }
    }

    pub fn create_ticket(&self, event_id: Uuid, ticket_type: String, price: f64, quota: u32) -> Result<Ticket, String> {
        // Validate price is positive
        if price < 0.0 {
            return Err("Ticket price cannot be negative".to_string());
        }

        let ticket = Ticket::new(event_id, ticket_type, price, quota);
        self.repository.save(ticket)
    }

    pub fn get_ticket(&self, id: &Uuid) -> Result<Option<Ticket>, String> {
        self.repository.find_by_id(id)
    }

    pub fn get_tickets_by_event(&self, event_id: &Uuid) -> Result<Vec<Ticket>, String> {
        self.repository.find_by_event_id(event_id)
    }

    pub fn update_ticket(
        &self,
        id: &Uuid,
        ticket_type: Option<String>,
        price: Option<f64>,
        quota: Option<u32>,
    ) -> Result<Ticket, String> {
        // Get existing ticket
        let ticket_option = self.repository.find_by_id(id)?;
        
        if let Some(mut ticket) = ticket_option {
            // Update fields if provided
            if let Some(new_type) = ticket_type {
                ticket.ticket_type = new_type;
            }

            if let Some(new_price) = price {
                ticket.update_price(new_price);
            }

            if let Some(new_quota) = quota {
                ticket.update_quota(new_quota);
            }

            // Save updates
            self.repository.update(ticket)
        } else {
            Err("Ticket not found".to_string())
        }
    }

    pub fn delete_ticket(&self, id: &Uuid) -> Result<(), String> {
        self.repository.delete(id)
    }

    pub fn allocate_tickets(&self, ticket_id: &Uuid, quantity: u32) -> Result<bool, String> {
        // Get ticket
        let ticket_option = self.repository.find_by_id(ticket_id)?;
        
        if let Some(ticket) = ticket_option {
            // Check if enough tickets available
            if quantity > 0 && ticket.is_available(quantity) {
                // Update quota
                let new_quota = ticket.quota - quantity;
                self.repository.update_quota(ticket_id, new_quota)?;
                Ok(true)
            } else if quantity == 0 {
                // Zero allocation is always successful
                Ok(true)
            } else {
                // Not enough tickets
                Ok(false)
            }
        } else {
            Err("Ticket not found".to_string())
        }
    }

    pub fn check_availability(&self, ticket_id: &Uuid, quantity: u32) -> Result<bool, String> {
        // Get ticket
        let ticket_option = self.repository.find_by_id(ticket_id)?;
        
        if let Some(ticket) = ticket_option {
            Ok(ticket.is_available(quantity))
        } else {
            Err("Ticket not found".to_string())
        }
    }
}
