#[macro_use] extern crate rocket;
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;

use crate::repository::advertisement::ad_repository::PostgresAdvertisementRepository;
use crate::service::advertisement::ad_service_factory::new_advertisement_service;
use crate::controller::advertisement::advertisement_routes;
use crate::error::handlers;

mod common;
mod controller;
mod dto;
mod error;
mod middleware;
mod model;
mod repository;
mod service;

#[get("/<name>/<age>")]
fn hello(name: &str, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[launch]
async fn rocket() -> _ {
    // Initialize logger
    env_logger::init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Set up database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");
    
    // Create repositories
    let ad_repository = Arc::new(PostgresAdvertisementRepository::new(pool.clone()));
    
    // Create services
    let ad_service = Arc::new(new_advertisement_service(ad_repository));
    
    // Build the rocket instance
    rocket::build()
        .mount("/hello", routes![hello])
        .mount("/api/v1", advertisement_routes())
        .manage(ad_service)
        .register("/", catchers![
            handlers::not_found,
            handlers::unprocessable_entity,
            handlers::server_error,
            handlers::unauthorized,
            handlers::forbidden
        ])
}