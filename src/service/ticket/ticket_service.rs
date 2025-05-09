use std::sync::Arc;
use uuid::Uuid;
use std::error::Error;

use crate::model::ticket::ticket::{Ticket, TicketStatus};
use crate::repository::ticket::TicketRepository;
use crate::events::ticket_events::{TicketEvent, TicketEventManager};
use crate::service::transaction::transaction_service::TransactionService;

pub struct TicketService {
    repository: Arc<dyn TicketRepository>,
}

impl TicketService {
    pub fn new(repository: Arc<dyn TicketRepository>) -> Self {
        Self { repository }
    }

    pub fn create_ticket(
        &self,
        event_id: Uuid,
        name: String,
        description: Option<String>,
        price: i64,
        quantity_available: i32,
        ticket_type: String,
        sale_start_date: Option<String>,
        sale_end_date: Option<String>,
    ) -> Result<Ticket, Box<dyn Error>> {
        if price < 0 {
            return Err("Price cannot be negative".into());
        }
        
        if quantity_available <= 0 {
            return Err("Quantity available must be positive".into());
        }
        
        let ticket = Ticket::new(
            event_id, 
            name, 
            description, 
            price, 
            quantity_available,
            ticket_type,
            sale_start_date,
            sale_end_date
        );
        self.repository.create_ticket(ticket)
    }
    
    pub fn get_ticket(&self, id: Uuid) -> Result<Option<Ticket>, Box<dyn Error>> {
        self.repository.get_ticket(id)
    }
    
    pub fn get_tickets_by_event(&self, event_id: Uuid) -> Result<Vec<Ticket>, Box<dyn Error>> {
        self.repository.get_tickets_by_event(event_id)
    }
    
    pub fn purchase_ticket(&self, id: Uuid, quantity: i32) -> Result<Ticket, Box<dyn Error>> {
        if quantity <= 0 {
            return Err("Quantity must be positive".into());
        }
        
        let ticket_result = self.repository.get_ticket(id)?;
        
        match ticket_result {
            Some(mut ticket) => {
                ticket.sell(quantity)?;
                self.repository.update_ticket(ticket)
            },
            None => Err("Ticket not found".into()),
        }
    }
    
    pub fn validate_ticket(&self, id: Uuid) -> Result<Ticket, Box<dyn Error>> {
        let ticket_result = self.repository.get_ticket(id)?;
        
        match ticket_result {
            Some(mut ticket) => {
                ticket.validate()?;
                self.repository.update_ticket(ticket)
            },
            None => Err("Ticket not found".into()),
        }
    }
    
    pub fn get_ticket_status(&self, id: Uuid) -> Result<TicketStatus, Box<dyn Error>> {
        let ticket_result = self.repository.get_ticket(id)?;
        
        match ticket_result {
            Some(ticket) => Ok(ticket.get_status()),
            None => Err("Ticket not found".into()),
        }
    }
    
    pub fn update_ticket(&self, ticket: Ticket) -> Result<Ticket, Box<dyn Error>> {
        self.repository.update_ticket(ticket)
    }
    
    pub fn delete_ticket(&self, id: Uuid) -> Result<(), Box<dyn Error>> {
        let ticket_result = self.repository.get_ticket(id)?;
        
        match ticket_result {
            Some(ticket) => {
                if ticket.is_purchased() {
                    return Err("Cannot delete purchased tickets".into());
                }
                self.repository.delete_ticket(id)
            },
            None => Err("Ticket not found".into()),
        }
    }
}