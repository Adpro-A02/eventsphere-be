use std::time::Instant;
use uuid::Uuid;
use crate::model::ticket::ticket::Ticket;
use crate::service::ticket::ticket_service::TicketService;

// Define a trait that both the decorator and real service will implement
pub trait TicketServiceTrait {
    fn create_ticket(&self, event_id: Uuid, ticket_type: String, price: f64, quota: u32) -> Result<Ticket, String>;
    fn get_ticket(&self, id: &Uuid) -> Result<Option<Ticket>, String>;
    fn get_tickets_by_event(&self, event_id: &Uuid) -> Result<Vec<Ticket>, String>;
    fn update_ticket(
        &self,
        id: &Uuid,
        ticket_type: Option<String>,
        price: Option<f64>,
        quota: Option<u32>,
    ) -> Result<Ticket, String>;
    fn delete_ticket(&self, id: &Uuid) -> Result<(), String>;
    fn allocate_tickets(&self, ticket_id: &Uuid, quantity: u32) -> Result<bool, String>;
    fn check_availability(&self, ticket_id: &Uuid, quantity: u32) -> Result<bool, String>;
}

// Implement the trait for the actual service
impl TicketServiceTrait for TicketService {
    fn create_ticket(&self, event_id: Uuid, ticket_type: String, price: f64, quota: u32) -> Result<Ticket, String> {
        self.create_ticket(event_id, ticket_type, price, quota)
    }
    
    fn get_ticket(&self, id: &Uuid) -> Result<Option<Ticket>, String> {
        self.get_ticket(id)
    }

    fn get_tickets_by_event(&self, event_id: &Uuid) -> Result<Vec<Ticket>, String> {
        self.get_tickets_by_event(event_id)
    }

    fn update_ticket(
        &self,
        id: &Uuid,
        ticket_type: Option<String>,
        price: Option<f64>,
        quota: Option<u32>,
    ) -> Result<Ticket, String> {
        self.update_ticket(id, ticket_type, price, quota)
    }

    fn delete_ticket(&self, id: &Uuid) -> Result<(), String> {
        self.delete_ticket(id)
    }

    fn allocate_tickets(&self, ticket_id: &Uuid, quantity: u32) -> Result<bool, String> {
        self.allocate_tickets(ticket_id, quantity)
    }

    fn check_availability(&self, ticket_id: &Uuid, quantity: u32) -> Result<bool, String> {
        self.check_availability(ticket_id, quantity)
    }
}

// Create the decorator
pub struct LoggingTicketService<T: TicketServiceTrait> {
    service: T,
    logger: Box<dyn Fn(&str) + Send + Sync>,
}

impl<T: TicketServiceTrait> LoggingTicketService<T> {
    pub fn new(service: T, logger: Box<dyn Fn(&str) + Send + Sync>) -> Self {
        Self { service, logger }
    }
}

// Implement the trait for the decorator
impl<T: TicketServiceTrait> TicketServiceTrait for LoggingTicketService<T> {
    fn create_ticket(&self, event_id: Uuid, ticket_type: String, price: f64, quota: u32) -> Result<Ticket, String> {
        (self.logger)(&format!("Creating ticket: type={}, price={}, quota={}", ticket_type, price, quota));
        let start = Instant::now();
        
        let result = self.service.create_ticket(event_id, ticket_type, price, quota);
        
        let duration = start.elapsed();
        (self.logger)(&format!("Ticket creation took: {:?}", duration));
        
        result
    }
    
    fn get_ticket(&self, id: &Uuid) -> Result<Option<Ticket>, String> {
        (self.logger)(&format!("Getting ticket with ID: {}", id));
        let start = Instant::now();
        
        let result = self.service.get_ticket(id);
        
        let duration = start.elapsed();
        (self.logger)(&format!("Get ticket took: {:?}", duration));
        
        result
    }

    fn get_tickets_by_event(&self, event_id: &Uuid) -> Result<Vec<Ticket>, String> {
        (self.logger)(&format!("Getting tickets for event ID: {}", event_id));
        let start = Instant::now();
        
        let result = self.service.get_tickets_by_event(event_id);
        
        let duration = start.elapsed();
        (self.logger)(&format!("Get tickets by event took: {:?}", duration));
        
        result
    }

    fn update_ticket(
        &self,
        id: &Uuid,
        ticket_type: Option<String>,
        price: Option<f64>,
        quota: Option<u32>,
    ) -> Result<Ticket, String> {
        (self.logger)(&format!("Updating ticket with ID: {}", id));
        let start = Instant::now();
        
        let result = self.service.update_ticket(id, ticket_type, price, quota);
        
        let duration = start.elapsed();
        (self.logger)(&format!("Update ticket took: {:?}", duration));
        
        result
    }

    fn delete_ticket(&self, id: &Uuid) -> Result<(), String> {
        (self.logger)(&format!("Deleting ticket with ID: {}", id));
        let start = Instant::now();
        
        let result = self.service.delete_ticket(id);
        
        let duration = start.elapsed();
        (self.logger)(&format!("Delete ticket took: {:?}", duration));
        
        result
    }

    fn allocate_tickets(&self, ticket_id: &Uuid, quantity: u32) -> Result<bool, String> {
        (self.logger)(&format!("Allocating {} tickets for ticket ID: {}", quantity, ticket_id));
        let start = Instant::now();
        
        let result = self.service.allocate_tickets(ticket_id, quantity);
        
        let duration = start.elapsed();
        (self.logger)(&format!("Allocate tickets took: {:?}", duration));
        
        result
    }

    fn check_availability(&self, ticket_id: &Uuid, quantity: u32) -> Result<bool, String> {
        (self.logger)(&format!("Checking availability for {} tickets for ticket ID: {}", quantity, ticket_id));
        let start = Instant::now();
        
        let result = self.service.check_availability(ticket_id, quantity);
        
        let duration = start.elapsed();
        (self.logger)(&format!("Check availability took: {:?}", duration));
        
        result
    }
}