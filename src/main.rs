#[macro_use] extern crate rocket;

use std::sync::Arc;
use std::collections::HashMap;
use std::sync::Mutex;
use rocket::routes;
use uuid::Uuid;

mod model;
mod repository;
mod service;
mod controller;
mod events;
mod config;
mod common;
mod infrastructure;
mod error;
mod api;

// Fix import paths to use 'ticket' instead of 'tiket'
use crate::model::ticket::ticket::{Ticket, TicketStatus};
use crate::events::ticket_events::{TicketEventManager, EmailNotifier, TicketEvent};

// Simple in-memory repository implementation
struct InMemoryTicketRepository {
    tickets: Mutex<HashMap<Uuid, Ticket>>,
}

impl InMemoryTicketRepository {
    fn new() -> Self {
        Self {
            tickets: Mutex::new(HashMap::new()),
        }
    }
}

// Fix repository implementation
impl repository::ticket::TicketRepository for InMemoryTicketRepository {
    fn save(&self, mut ticket: Ticket) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        
        // Generate UUID if not present
        if ticket.id.is_none() {
            ticket.id = Some(Uuid::new_v4());
        }
        
        let id = ticket.id.unwrap();
        tickets.insert(id, ticket.clone());
        println!("Saved ticket: {:?}", ticket);
        
        Ok(ticket)
    }
    
    fn find_by_id(&self, id: &Uuid) -> Result<Option<Ticket>, String> {
        let tickets = self.tickets.lock().unwrap();
        println!("Finding ticket by ID: {}", id);
        Ok(tickets.get(id).cloned())
    }
    
    fn find_by_event_id(&self, event_id: &Uuid) -> Result<Vec<Ticket>, String> {
        let tickets = self.tickets.lock().unwrap();
        println!("Finding tickets for event: {}", event_id);
        let matching: Vec<Ticket> = tickets.values()
            .filter(|t| t.event_id == *event_id)
            .cloned()
            .collect();
        Ok(matching)
    }
    
    fn update(&self, ticket: Ticket) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        
        let id = ticket.id.ok_or("Ticket ID is required for update")?;
        
        if !tickets.contains_key(&id) {
            return Err("Ticket not found".to_string());
        }
        
        tickets.insert(id, ticket.clone());
        println!("Updated ticket: {:?}", ticket);
        Ok(ticket)
    }
    
    fn delete(&self, id: &Uuid) -> Result<(), String> {
        let mut tickets = self.tickets.lock().unwrap();
        
        if tickets.remove(id).is_none() {
            return Err("Ticket not found".to_string());
        }
        
        println!("Deleted ticket ID: {}", id);
        Ok(())
    }
    
    fn update_quota(&self, id: &Uuid, new_quota: u32) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        
        let ticket = tickets.get_mut(id)
            .ok_or_else(|| "Ticket not found".to_string())?;
            
        ticket.update_quota(new_quota);
        println!("Updated ticket quota: {} -> {}", id, new_quota);
        
        Ok(ticket.clone())
    }
}

// Simple mock transaction service
struct MockTransactionService;

impl service::transaction::TransactionService for MockTransactionService {
    fn create_transaction(
        &self,
        user_id: Uuid,
        ticket_id: Option<Uuid>,
        amount: i64,
        description: String,
        payment_method: String,
    ) -> Result<model::transaction::Transaction, Box<dyn std::error::Error>> {
        println!("Creating transaction: {} for ticket {:?}", amount, ticket_id);
        // Create a mock transaction and return it
        let transaction = model::transaction::Transaction::new(
            user_id, ticket_id, amount, description, payment_method
        );
        Ok(transaction)
    }

    fn process_payment(&self, transaction_id: Uuid, _: Option<String>) -> Result<model::transaction::Transaction, Box<dyn std::error::Error>> {
        println!("Processing payment for transaction: {}", transaction_id);
        // Mock a successful payment
        let mut transaction = model::transaction::Transaction::new(
            Uuid::new_v4(), None, 100, "Mocked transaction".to_string(), "credit_card".to_string()
        );
        transaction.process(true, Some("MOCK-PAY-REF".to_string()));
        Ok(transaction)
    }

    // Minimal implementation of other required methods
    fn validate_payment(&self, _: Uuid) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }
    
    fn refund_transaction(&self, _: Uuid) -> Result<model::transaction::Transaction, Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }
    
    fn get_transaction(&self, _: Uuid) -> Result<Option<model::transaction::Transaction>, Box<dyn std::error::Error>> {
        Ok(None)
    }
    
    fn get_user_transactions(&self, _: Uuid) -> Result<Vec<model::transaction::Transaction>, Box<dyn std::error::Error>> {
        Ok(Vec::new())
    }
    
    fn add_funds_to_balance(&self, _: Uuid, _: i64, _: String) -> Result<(model::transaction::Transaction, i64), Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }
    
    fn withdraw_funds(&self, _: Uuid, _: i64, _: String) -> Result<(model::transaction::Transaction, i64), Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }

    fn delete_transaction(&self, _: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to EventSphere Ticket API!"
}

#[get("/create-sample")]
fn create_sample() -> String {
    // Create an event and ticket
    let event_id = Uuid::new_v4();
    let ticket_repo = Box::new(InMemoryTicketRepository::new());
    let event_manager = Arc::new(TicketEventManager::new());
    
    let service = service::ticket::ticket_service::TicketService::new(
        ticket_repo,
        event_manager,
        Some(Arc::new(MockTransactionService))
    );
    
    let result = service.create_ticket(
        event_id, 
        "VIP Ticket".to_string(), 
        100.0, 
        50
    );
    
    match result {
        Ok(ticket) => format!("Sample ticket created: {:?}", ticket),
        Err(e) => format!("Error creating sample: {}", e),
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    println!("Starting EventSphere API...");
    
    // Create config
    let config = config::Config::default();
    
    // Initialize services and repositories
    let ticket_repo = Box::new(InMemoryTicketRepository::new());
    let event_manager = Arc::new(TicketEventManager::new());
    let ticket_service = service::ticket::ticket_service::TicketService::new(
        ticket_repo,
        event_manager,
        Some(Arc::new(MockTransactionService))
    );
    
    println!("Server starting at http://localhost:8000");
    
    // Build the rocket instance with our API
    api::init(&config)
        // Register services as managed state
        .manage(Box::new(ticket_service) as Box<dyn service::ticket::ticket_service::TicketService + Send + Sync>)
        .manage(Arc::new(MockTransactionService) as Arc<dyn service::transaction::TransactionService + Send + Sync>)
        // Launch the server
        .launch()
        .await
}
