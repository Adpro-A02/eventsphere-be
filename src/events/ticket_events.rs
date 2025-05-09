use crate::model::ticket::ticket::Ticket;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use std::fmt::Debug;

/// Defines the possible ticket-related events
#[derive(Clone, Debug)]
pub enum TicketEvent {
    Created(Ticket),
    Updated(Ticket),
    Deleted(Uuid),
    Allocated { ticket_id: Uuid, quantity: u32 },
    Purchased { ticket_id: Uuid, user_id: Uuid, quantity: u32, transaction_id: Uuid },
    SoldOut(Uuid),
    Validated { ticket_id: Uuid, validator_id: Uuid },
}

/// Interface for objects that can handle ticket events
pub trait TicketEventObserver: Send + Sync + Debug {
    fn on_event(&self, event: &TicketEvent);
}

/// Manages ticket events and observers
#[derive(Debug)]
pub struct TicketEventManager {
    observers: Mutex<Vec<Arc<dyn TicketEventObserver>>>,
}

impl TicketEventManager {
    pub fn new() -> Self {
        Self {
            observers: Mutex::new(Vec::new()),
        }
    }

    /// Add an observer that will be notified of ticket events
    pub fn add_observer(&self, observer: Arc<dyn TicketEventObserver>) {
        let mut observers = self.observers.lock().unwrap();
        observers.push(observer);
    }

    /// Notify all registered observers about a ticket event
    pub fn notify_observers(&self, event: TicketEvent) {
        let observers = self.observers.lock().unwrap();
        for observer in observers.iter() {
            observer.on_event(&event);
        }
    }
}

/// Sample observer that sends email notifications
#[derive(Debug)]
pub struct EmailNotifier {}

impl EmailNotifier {
    pub fn new() -> Self {
        Self {}
    }
}

impl TicketEventObserver for EmailNotifier {
    fn on_event(&self, event: &TicketEvent) {
        match event {
            TicketEvent::Created(ticket) => {
                println!("📧 Email: Ticket type '{}' for event {} has been created", 
                    ticket.ticket_type, ticket.event_id);
            },
            TicketEvent::Purchased { ticket_id, user_id, quantity, .. } => {
                println!("📧 Email: User {} has purchased {} tickets (ID: {})", 
                    user_id, quantity, ticket_id);
            },
            TicketEvent::SoldOut(ticket_id) => {
                println!("📧 Email: Ticket {} is now sold out!", ticket_id);
            },
            _ => {} // Other events don't trigger emails
        }
    }
}
