#[macro_use] extern crate rocket;

mod model;
mod repository;
mod service;
mod controller;

use std::sync::Arc;
use rocket::fs::{FileServer, relative};
use rocket::{Build, Rocket};

use repository::event::InMemoryEventRepository;
use service::event::event_service::EventService;
use controller::event::event_controller::{routes, DynEventService, EventServiceTrait};

#[launch]
fn rocket() -> Rocket<Build> {
    
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    
    let repository = Arc::new(InMemoryEventRepository::new());
    
    // Create service
    let service: Arc<EventService<InMemoryEventRepository>> = Arc::new(EventService::new(repository.clone()));
    
    // Create dynamic service (type erasure)
    let dyn_service: DynEventService = service as Arc<dyn EventServiceTrait + Send + Sync>;
    
    // Build Rocket instance
    rocket::build()
        .manage(dyn_service)
        .mount("/api", routes())
}