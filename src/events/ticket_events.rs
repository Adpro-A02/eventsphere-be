use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::model::tiket::ticket::Ticket;

#[derive(Clone, Debug)]
pub enum TicketEvent {
    Created(Ticket),
    Updated(Ticket),
    Deleted(Uuid),
    Allocated { ticket_id: Uuid, quantity: u32 },
    SoldOut(Uuid),
}

pub trait TicketEventObserver: Send + Sync {
    fn on_event(&self, event: TicketEvent);
}

pub struct TicketEventManager {
    observers: Arc<Mutex<Vec<Arc<dyn TicketEventObserver>>>>,
}

impl TicketEventManager {
    pub fn new() -> Self {
        Self {
            observers: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn add_observer(&self, observer: Arc<dyn TicketEventObserver>) {
        let mut observers = self.observers.lock().unwrap();
        observers.push(observer);
    }
    
    pub fn notify_observers(&self, event: TicketEvent) {
        let observers = self.observers.lock().unwrap();
        for observer in observers.iter() {
            observer.on_event(event.clone());
        }
    }
}

// Example observer
pub struct EmailNotifier;

impl TicketEventObserver for EmailNotifier {
    fn on_event(&self, event: TicketEvent) {
        match event {
            TicketEvent::SoldOut(ticket_id) => {
                println!("Sending email: Ticket {} is now sold out!", ticket_id);
                // In a real system, this would call an email service
            },
            TicketEvent::Created(ticket) => {
                println!("Sending email: New ticket created for event {}", ticket.event_id);
            },
            _ => {} // Handle other events as needed
        }
    }
}
