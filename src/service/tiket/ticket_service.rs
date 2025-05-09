use std::sync::Arc;
use crate::model::ticket::ticket::{Ticket, TicketStatus};
use crate::repository::ticket::TicketRepository;
use crate::events::ticket_events::{TicketEvent, TicketEventManager};
use crate::service::transaction::transaction_service::TransactionService;
use uuid::Uuid;

pub struct TicketService {
    repository: Box<dyn TicketRepository>,
    event_manager: Arc<TicketEventManager>,
    transaction_service: Option<Arc<dyn TransactionService + Send + Sync>>,
}

impl TicketService {
    pub fn new(
        repository: Box<dyn TicketRepository>,
        event_manager: Arc<TicketEventManager>,
        transaction_service: Option<Arc<dyn TransactionService + Send + Sync>>,
    ) -> Self {
        Self { 
            repository, 
            event_manager,
            transaction_service,
        }
    }

    pub fn create_ticket(&self, event_id: Uuid, ticket_type: String, price: f64, quota: u32) -> Result<Ticket, String> {
        // Validate price is positive
        if price < 0.0 {
            return Err("Ticket price cannot be negative".to_string());
        }

        let ticket = Ticket::new(event_id, ticket_type, price, quota);
        let saved_ticket = self.repository.save(ticket)?;
        
        // Notify observers of ticket creation
        self.event_manager.notify_observers(TicketEvent::Created(saved_ticket.clone()));
        
        Ok(saved_ticket)
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
            let updated_ticket = self.repository.update(ticket)?;
            
            // Notify observers of ticket update
            self.event_manager.notify_observers(TicketEvent::Updated(updated_ticket.clone()));
            
            Ok(updated_ticket)
        } else {
            Err("Ticket not found".to_string())
        }
    }

    pub fn delete_ticket(&self, id: &Uuid) -> Result<(), String> {
        // Check if ticket exists
        let ticket_option = self.repository.find_by_id(id)?;
        
        if let Some(ticket) = ticket_option {
            // Check if ticket has been purchased
            if ticket.is_purchased() {
                return Err("Cannot delete tickets that have been purchased".to_string());
            }
            
            // Proceed with deletion
            self.repository.delete(id)?;
            
            // Notify observers of ticket deletion
            self.event_manager.notify_observers(TicketEvent::Deleted(*id));
            
            Ok(())
        } else {
            Err("Ticket not found".to_string())
        }
    }

    pub fn allocate_tickets(&self, ticket_id: &Uuid, quantity: u32) -> Result<bool, String> {
        // Get ticket
        let ticket_option = self.repository.find_by_id(ticket_id)?;
        
        if let Some(ticket) = ticket_option {
            // Check if enough tickets available
            if quantity > 0 && ticket.is_available(quantity) {
                // Update quota
                let new_quota = ticket.quota - quantity;
                let updated_ticket = self.repository.update_quota(ticket_id, new_quota)?;
                
                // Notify of allocation
                self.event_manager.notify_observers(TicketEvent::Allocated { 
                    ticket_id: *ticket_id, 
                    quantity 
                });
                
                // Check if tickets are now sold out
                if updated_ticket.status == TicketStatus::SOLD_OUT {
                    self.event_manager.notify_observers(TicketEvent::SoldOut(*ticket_id));
                }
                
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

    pub fn purchase_ticket(
        &self,
        user_id: Uuid,
        ticket_id: &Uuid,
        quantity: u32,
        payment_method: String,
    ) -> Result<(Ticket, Uuid), String> {
        // Check if ticket exists and is available
        let ticket_option = self.repository.find_by_id(ticket_id)?;
        
        if let Some(mut ticket) = ticket_option {
            // Check if enough tickets are available
            if !ticket.is_available(quantity) {
                return Err("Not enough tickets available".to_string());
            }
            
            // Calculate total price
            let total_price = ticket.price * quantity as f64;
            
            // Create transaction record
            let transaction_service = self.transaction_service
                .as_ref()
                .ok_or("Transaction service not available".to_string())?;
                
            let transaction = transaction_service.create_transaction(
                user_id,
                Some(*ticket_id),
                total_price as i64,
                format!("Purchase of {} x {} tickets", quantity, ticket.ticket_type),
                payment_method,
            ).map_err(|e| e.to_string())?;
            
            // Reduce ticket quota
            self.allocate_tickets(ticket_id, quantity)?;
            
            // Mark the ticket as purchased
            let mut updated_ticket = ticket.clone();
            updated_ticket.mark_as_purchased();
            let saved_ticket = self.repository.update(updated_ticket)?;
            
            // Process payment
            let transaction_id = transaction.id;
            transaction_service.process_payment(transaction_id, None)
                .map_err(|e| e.to_string())?;
            
            // Notify observers about purchase
            self.event_manager.notify_observers(TicketEvent::Purchased { 
                ticket_id: *ticket_id, 
                user_id,
                quantity,
                transaction_id,
            });
            
            Ok((saved_ticket, transaction_id))
        } else {
            Err("Ticket not found".to_string())
        }
    }
    
    pub fn validate_ticket(&self, ticket_id: &Uuid, validator_id: &Uuid, role: &str) -> Result<Ticket, String> {
        // Check if validator has admin or organizer role
        if role != "admin" && role != "organizer" {
            return Err("Unauthorized: Only admin or organizer can validate tickets".to_string());
        }
        
        // Check if ticket exists
        let ticket_option = self.repository.find_by_id(ticket_id)?;
        
        if let Some(mut ticket) = ticket_option {
            // Check if ticket is purchased
            if !ticket.is_purchased() {
                return Err("Ticket has not been purchased".to_string());
            }
            
            // Check if ticket is already used
            if ticket.is_used() {
                return Err("Ticket has already been used".to_string());
            }
            
            // Mark ticket as used
            ticket.mark_as_used()?;
            let updated_ticket = self.repository.update(ticket)?;
            
            // Notify observers
            self.event_manager.notify_observers(TicketEvent::Validated { 
                ticket_id: *ticket_id, 
                validator_id: *validator_id,
            });
            
            Ok(updated_ticket)
        } else {
            Err("Ticket not found".to_string())
        }
    }
}
