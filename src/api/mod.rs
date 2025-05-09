pub mod v1;
mod middleware;

use rocket::{Build, Rocket};
use crate::error::handlers;
use crate::config::Config;

/// Initializes and configures the REST API
pub fn init(config: &Config) -> Rocket<Build> {
    rocket::build()
        // Mount API versions
        .mount("/api/v1", v1::routes())
        
        // Mount other routes like health checks
        .mount("/health", routes![health_check])
        
        // Register error catchers
        .register("/", catchers![
            handlers::not_found, 
            handlers::unprocessable_entity,
            handlers::server_error,
            handlers::unauthorized,
            handlers::forbidden
        ])
}

/// Simple health check endpoint
#[get("/")]
fn health_check() -> &'static str {
    "OK"
}
