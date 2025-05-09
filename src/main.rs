use actix_web::{App, HttpServer, web, middleware::Logger};
use std::sync::Arc;

mod model;
mod repository;
mod service;
mod controller;

use repository::review::review_repository::InMemoryReviewRepository;  // Ensure you use the correct path
use service::review::review_service::ReviewService;  // Use concrete ReviewService
use controller::review::review_controller::configure_routes;  // Configure the routes
use service::review::notification_service::NotificationService;  // Notification service
use uuid::Uuid;  // Uuid for unique identifiers

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Create the repository and notification service
    let repository = Arc::new(InMemoryReviewRepository::new());
    let notification_service = Arc::new(NotificationService::new());

    // Create the concrete ReviewService
    let service = Arc::new(ReviewService::new(repository.clone(), notification_service.clone()));

    // Start Actix Web server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())  // Enable logging middleware
            .app_data(web::Data::new(service.clone()))  // Add the ReviewService as App Data
            .configure(|cfg| configure_routes::<InMemoryReviewRepository>(cfg))  // Pass InMemoryReviewRepository as R
    })
    .bind("127.0.0.1:8080")?  // Bind to localhost on port 8080
    .run()
    .await

}
