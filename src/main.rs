#[macro_use] extern crate rocket;

#[get("/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

mod config;
mod controller;
mod events;
mod model;
mod repository;
mod service;

use std::sync::Arc;
use rocket::{self, routes};
use crate::config::DatabaseConfig;
use crate::controller::tiket::ticket_controller;
use crate::events::ticket_events::{TicketEventManager, EmailNotifier};
use crate::repository::factory::{DatabaseType, RepositoryFactory};
use crate::service::decorators::logging_decorator::LoggingTicketService;
use crate::service::tiket::ticket_service::TicketService;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up database configuration
    let db_config = DatabaseConfig::from_env()?;
    
    // Create repository using factory pattern
    let ticket_repo = RepositoryFactory::create_ticket_repository(
        DatabaseType::Postgres,
        &db_config
    );
    
    // Set up event manager with observers
    let event_manager = Arc::new(TicketEventManager::new());
    event_manager.add_observer(Arc::new(EmailNotifier));
    
    // Create service with repository
    let ticket_service = TicketService::new(
        Box::new(ticket_repo),
        event_manager.clone()
    );
    
    // Add logging decorator
    let logging_service = LoggingTicketService::new(
        ticket_service,
        Box::new(|msg| println!("[TICKET SERVICE] {}", msg))
    );
    
    // Start the Rocket server
    rocket::build()
        .mount("/api", ticket_controller::routes())
        .manage(Box::new(logging_service) as Box<dyn TicketServiceTrait + Send + Sync>)
        .launch()
        .await?;
        
    Ok(())
}